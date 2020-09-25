use std::sync::Arc;
use std::{
    env,
    future::Future,
    task::{Context, Poll},
};

/// Extension trait for efficiently blocking on a future.
use crossbeam_utils::sync::{Parker, Unparker};
use futures_util::{pin_mut, task::ArcWake};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

// ===============================================================================================
// Retry
// ===============================================================================================
#[doc(hidden)]
pub(crate) async fn with_retry<T, U, F, Fut>(retries: usize, f: F) -> Result<T, U>
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

// ===============================================================================================
// Environment
// ===============================================================================================
#[doc(hidden)]
pub(crate) fn read_env(name: &str, default: &str) -> String {
    match std::env::var(name) {
        Ok(value) => value,
        Err(_) => default.to_string(),
    }
}

// ===============================================================================================
// Futures
// ===============================================================================================
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

// ===============================================================================================
// Files
// ===============================================================================================
pub(crate) fn get_test_resource_file_path(relative_resource_path: &str) -> Result<PathBuf, String> {
    match env::var("CARGO_MANIFEST_DIR") {
        Ok(manifest_path) => Ok(Path::new(&manifest_path).join(relative_resource_path)),
        Err(e) => Err(e.to_string()),
    }
}

pub(crate) fn read_file<P: AsRef<Path>>(absolute_resource_path: P) -> Result<Vec<u8>, String> {
    let mut f = match File::open(&absolute_resource_path) {
        Ok(mut opened_file) => opened_file,
        Err(e) => return Err(e.to_string()),
    };
    let mut buffer = Vec::new();
    match f.read_to_end(&mut buffer) {
        Ok(len) => log::trace!(
            "Read {} bytes from file {:?}",
            &len,
            &absolute_resource_path.as_ref().as_os_str().to_str().expect("Invalid file path")
        ),
        Err(e) => return Err(e.to_string()),
    }

    Ok(buffer)
}

#[cfg(test)]
mod test {
    use crate::util::{with_retry, Join};

    #[test]
    fn with_retry_error_test() {
        let result: Result<(), &str> = with_retry(1, || async {
            return Err("test error");
        })
        .join();

        assert_eq!(result.is_err(), true);
        assert_eq!(result.err().unwrap(), "test error")
    }
}
