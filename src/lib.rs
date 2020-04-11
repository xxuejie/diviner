pub mod time;
mod utils;

use crate::time::{now, TimerEntry};
use futures::future::FutureExt;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::cell::RefCell;
use std::collections::binary_heap::BinaryHeap;
use std::future::Future;
use std::panic::AssertUnwindSafe;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use std::thread::Result;
use std::time::SystemTime;

type Task = async_task::Task<()>;
type JoinHandle<T> = async_task::JoinHandle<T, ()>;

pub struct Environment {
    rng: ChaCha8Rng,
    current_time: SystemTime,
    tasks: Vec<Task>,
    timers: BinaryHeap<TimerEntry>,
}

thread_local! {
    pub(crate) static ENV: RefCell<Option<Environment>> = RefCell::new(None);
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            rng: ChaCha8Rng::from_entropy(),
            current_time: SystemTime::now(),
            tasks: vec![],
            timers: BinaryHeap::new(),
        }
    }

    pub fn new_with_seed(seed: u64) -> Self {
        Environment {
            rng: ChaCha8Rng::seed_from_u64(seed),
            current_time: SystemTime::now(),
            tasks: vec![],
            timers: BinaryHeap::new(),
        }
    }

    pub fn block_on<F, R>(self, future: F) -> Result<R>
    where
        F: Future<Output = R> + 'static,
        R: Send + 'static,
    {
        let root_future = AssertUnwindSafe(future).catch_unwind();
        pin_utils::pin_mut!(root_future);
        ENV.with(|e| {
            assert!(
                e.borrow().is_none(),
                "Current thread should not have an environment when calling block_on!"
            );
            e.replace(Some(self));
        });
        let root_runnable_flag = Arc::new(Mutex::new(true));
        let waker = {
            let flag2 = Arc::clone(&root_runnable_flag);
            async_task::waker_fn(move || *flag2.lock().expect("root waker") = true)
        };
        let root_cx = &mut Context::from_waker(&waker);
        let result = loop {
            let root_runnable = *root_runnable_flag.lock().expect("polling root");
            let mut num = ENV.with(|e| e.borrow().as_ref().unwrap().tasks.len());
            if root_runnable {
                num += 1;
            }
            if num > 0 {
                let i = ENV.with(|e| e.borrow_mut().as_mut().unwrap().rng.gen_range(0, num));
                if root_runnable && i == 0 {
                    *root_runnable_flag.lock().expect("suspending root") = false;
                    if let Poll::Ready(output) = root_future.as_mut().poll(root_cx) {
                        break output;
                    }
                } else {
                    let index = if root_runnable { i - 1 } else { i };
                    let task = ENV.with(|e| e.borrow_mut().as_mut().unwrap().tasks.remove(index));
                    task.run();
                }
                continue;
            }
            if let Some(entry) = ENV.with(|e| e.borrow_mut().as_mut().unwrap().timers.pop()) {
                if entry.wake_time >= now() {
                    ENV.with(|e| {
                        e.borrow_mut().as_mut().unwrap().current_time = entry.wake_time;
                    });
                    for waker in entry.wakers {
                        waker.wake();
                    }
                    continue;
                }
            }
            break Err(Box::new("No task is runnable!"));
        };
        ENV.with(|e| {
            e.replace(None);
        });
        result
    }
}

pub fn spawn<F, R>(future: F) -> JoinHandle<Result<R>>
where
    F: Future<Output = R> + 'static,
    R: Send + 'static,
{
    let future = AssertUnwindSafe(future).catch_unwind();
    let schedule = |t| {
        ENV.with(|e| {
            e.borrow_mut().as_mut().unwrap().tasks.push(t);
        })
    };
    let (task, handle) = async_task::spawn_local(future, schedule, ());
    task.schedule();

    handle
}
