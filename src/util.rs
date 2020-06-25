use std::sync::{Arc, Condvar, Mutex};
use std::{thread, time};

#[doc(hidden)]
/// Executes the provided function a given number of times with the given interval between
/// the retries. This function swallows all results and only returns the last result.
pub fn with_retry<T, U>(retries: usize, delay: u64, f: impl Fn() -> Result<T, U>) -> Result<T, U> {
    let mut result = (f)();
    for _ in 1..=retries {
        if result.is_ok() {
            return result;
        }
        thread::sleep(time::Duration::from_millis(delay));
        result = (f)();
    }
    result
}

#[derive(Debug, Clone)]
pub struct MaxPassLatch {
    pair: Arc<(Arc<Mutex<usize>>, Condvar)>,
    max: usize,
}

impl MaxPassLatch {
    pub fn new(max: usize) -> MaxPassLatch {
        MaxPassLatch {
            pair: Arc::new((Arc::new(Mutex::new(0)), Condvar::new())),
            max,
        }
    }

    pub fn leave(&self) {
        let &(ref lock, ref cvar) = &*self.pair.clone();
        let mut started = lock.lock().unwrap();
        if *started > 0 {
            *started -= 1;
        }
        cvar.notify_one();
    }

    pub fn enter(&self) {
        let &(ref lock, ref cvar) = &*self.pair.clone();
        let mut started = lock.lock().unwrap();
        while *started >= self.max {
            let result = cvar.wait(started);

            if result.is_err() {
                started = result.err().unwrap().into_inner();
            } else {
                started = result.unwrap();
            }
        }
        *started += 1;
    }
}

pub fn read_env(name: &str, default: &str) -> String {
    return match std::env::var(name) {
        Ok(value) => value,
        Err(_) => default.to_string(),
    };
}



/// Extension trait for efficiently blocking on a future.
use crossbeam_utils::sync::{Parker, Unparker};
use futures_util::{pin_mut, task::ArcWake};
use std::{
    future::Future,
    net::{UdpSocket},
    task::{Context, Poll, Waker},
};

pub(crate) trait Join: Future {
    fn join(self) -> <Self as Future>::Output;
}

impl<F: Future> Join for F {
    fn join(self) -> <Self as Future>::Output {
        struct ThreadWaker(Unparker);

        impl ArcWake for ThreadWaker {
            fn wake_by_ref(arc_self: &Arc<Self>) {
                arc_self.0.unpark();
            }
        }

        let parker = Parker::new();
        let waker = futures_util::task::waker(Arc::new(ThreadWaker(parker.unparker().clone())));
        let mut context = Context::from_waker(&waker);

        let future = self;
        pin_mut!(future);

        loop {
            match future.as_mut().poll(&mut context) {
                Poll::Ready(output) => return output,
                Poll::Pending => parker.park(),
            }
        }
    }
}
