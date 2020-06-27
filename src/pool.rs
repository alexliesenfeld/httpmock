use std::ptr;
use std::sync::Arc;

use async_std::sync::{Condvar, Mutex};

#[derive(Debug)]
pub struct Pool<T> {
    sync_tuple: Mutex<(Vec<T>, usize)>,
    cvar: Condvar,
    max: usize,
}

impl<T> Pool<T> {
    pub fn new(max: usize) -> Self {
        Self {
            sync_tuple: Mutex::new((Vec::new(), 0)),
            cvar: Condvar::new(),
            max,
        }
    }

    pub async fn put(&self, item: T) {
        let mut lock_guard = (&self.sync_tuple).lock().await;
        (*lock_guard).0.push(item);
        self.cvar.notify_one();
    }

    pub async fn take<F>(&self, create: F) -> T
    where
        F: FnOnce() -> T,
    {
        let mut lock_guard = (&self.sync_tuple).lock().await;

        while (*lock_guard).0.is_empty() && (*lock_guard).1 == self.max {
            lock_guard = self.cvar.wait(lock_guard).await;
        }

        if (*lock_guard).1 < self.max {
            (*lock_guard).0.push(create());
            (*lock_guard).1 += 1;
        }

        return (*lock_guard).0.remove(0);
    }
}
