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
        println!("7");
        let mut lock_guard = (&self.sync_tuple).lock().await;
        (*lock_guard).0.push(item);
        println!("8");
        self.cvar.notify_one();
    }

    pub async fn take<F>(&self, create: F) -> T
        where
            F: FnOnce() -> T,
    {
        println!("1");
        let mut lock_guard = (&self.sync_tuple).lock().await;

        println!("2");
        while (*lock_guard).0.is_empty() && (*lock_guard).1 == self.max {
            println!("3");
            lock_guard = self.cvar.wait(lock_guard).await;
        }

        println!("4");
        if (*lock_guard).1 < self.max {
            println!("5");
            (*lock_guard).0.push(create());
            (*lock_guard).1 += 1;
        }

        println!("6");
        return (*lock_guard).0.remove(0);
    }
}
