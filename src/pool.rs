use std::ptr;
use async_std::sync::{Mutex, Condvar};
use std::sync::Arc;

#[derive(Debug)]
struct ItemPoolState<T> {
    pub free: Vec<Arc<T>>,
    pub taken: Vec<Arc<T>>,
}

impl<T> ItemPoolState<T> {
    fn new() -> Self {
        Self {
            taken: Vec::new(),
            free: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct ItemPool<T> {
    sync_pair: Arc<(Arc<Mutex<ItemPoolState<T>>>, Condvar)>,
    max: usize,
}

impl<T> ItemPool<T> {
    pub fn new(max: usize) -> ItemPool<T> {
        ItemPool {
            sync_pair: Arc::new((Arc::new(Mutex::new(ItemPoolState::new())), Condvar::new())),
            max,
        }
    }

    pub async fn put_back(&self, item: Arc<T>) {
        let &(ref lock, ref cvar) = &*self.sync_pair.clone();
        let mut state = lock.lock().await;

        if let Some(idx) = (*state)
            .taken
            .iter()
            .position(|e| ptr::eq(e.as_ref(), item.as_ref()))
        {
            (*state).taken.remove(idx);
        }

        (*state).free.push(item);

        cvar.notify_one();
    }

    pub async fn get_or_create_from<F>(&self, create: F) -> Arc<T>
        where
            F: FnOnce() -> T,
    {
        let &(ref lock, ref cvar) = &*self.sync_pair.clone();
        let mut state = lock.lock().await;

        while (*state).free.len() == 0 && (*state).taken.len() >= self.max {
            state = cvar.wait(state).await;
        }

        if ((*state).free.len() + (*state).taken.len()) < self.max {
            (*state).free.push(Arc::new(create()));
        }

        let free = (*state).free.pop().unwrap();
        (*state).taken.push(free.clone());

        return free;
    }
}
