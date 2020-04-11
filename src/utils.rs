use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};

struct SignalState {
    completed: bool,
    // TODO: this is ugly, I will need to find a way to wrap an existing waker
    // with additional function. Right now moving a Waker inside a closure will
    // make that closure FnOnce, while async_task::waker_fn takes Fn.
    attacher: Option<Box<dyn FnOnce(Waker, Waker) + Send + 'static>>,
}

// This is a minimal future that will complete as soon as waker is called.
// It is diviner specific since it is tailored for the single threaded
// running environment of diviner.
pub(crate) struct Signal {
    state: Arc<Mutex<SignalState>>,
}

impl Signal {
    pub fn new<F>(attacher: F) -> Self
    where
        F: FnOnce(Waker, Waker) + Send + 'static,
    {
        Signal {
            state: Arc::new(Mutex::new(SignalState {
                completed: false,
                attacher: Some(Box::new(attacher)),
            })),
        }
    }
}

impl Future for Signal {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.state.lock().unwrap().completed {
            Poll::Ready(())
        } else {
            if let Some(attacher) = self.state.lock().unwrap().attacher.take() {
                let state = Arc::clone(&self.state);
                attacher(
                    async_task::waker_fn(move || {
                        state.lock().unwrap().completed = true;
                    }),
                    cx.waker().clone(),
                );
            }
            Poll::Pending
        }
    }
}
