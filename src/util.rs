use std::sync::Arc;
use std::{
    future::Future,
    task::{Context, Poll},
};

/// Extension trait for efficiently blocking on a future.
use crossbeam_utils::sync::{Parker, Unparker};
use futures_util::{pin_mut, task::ArcWake};

#[doc(hidden)]
pub async fn with_retry<T, U, F, Fut>(retries: usize, f: F) -> Result<T, U>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, U>>,
{
    let mut result = (f)().await;
    for _ in 1..=retries {
        if result.is_ok() {
            return result;
        }
        result = (f)().await;
    }
    result
}

#[doc(hidden)]
pub fn read_env(name: &str, default: &str) -> String {
    match std::env::var(name) {
        Ok(value) => value,
        Err(_) => default.to_string(),
    }
}

#[doc(hidden)]
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
