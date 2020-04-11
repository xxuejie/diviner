use crate::{spawn, utils::Signal, ENV};
use std::cmp::{Ordering, Reverse};
use std::task::Waker;
use std::time::{Duration, SystemTime};

pub async fn sleep(dur: Duration) {
    let wake_time = now() + dur;
    let attach = move |waker, waker2| {
        let entry = TimerEntry {
            wake_time,
            wakers: vec![waker, waker2],
        };
        ENV.with(|e| e.borrow_mut().as_mut().unwrap().timers.push(entry));
    };
    spawn(Signal::new(attach))
        .await
        .expect("Task is unexpected canceled!")
        .expect("Sleep task failure!")
}

pub fn now() -> SystemTime {
    ENV.with(|e| e.borrow().as_ref().unwrap().current_time)
}

pub(crate) struct TimerEntry {
    pub(crate) wake_time: SystemTime,
    pub(crate) wakers: Vec<Waker>,
}

impl Ord for TimerEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        Reverse(self.wake_time).cmp(&Reverse(other.wake_time))
    }
}

impl PartialOrd for TimerEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for TimerEntry {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

impl Eq for TimerEntry {}
