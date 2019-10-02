use std::{thread, time};

#[doc(hidden)]
/// Executes the provided function a given number of times with the given interval between
/// the retries. This function swallows all results and only returns the last result.
pub fn with_retry<T, U>(
    retries: usize,
    interval: u64,
    f: impl Fn() -> Result<T, U>,
) -> Result<T, U> {
    let mut result = (f)();
    for _ in 1..=retries {
        if result.is_ok() {
            return result;
        }
        thread::sleep(time::Duration::from_millis(interval));
        result = (f)();
    }
    result
}
