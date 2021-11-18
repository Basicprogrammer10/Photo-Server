use std::collections::hash_map::HashMap;
use std::hash::Hash;
use std::time::Duration;
use std::time::SystemTime;

pub struct Cache<T, M>
where
    T: Hash + Eq,
    M: Hash + Clone,
{
    cache_items: HashMap<T, CacheItem<M>>,
    timeout: Duration,
}

struct CacheItem<M> {
    data: M,
    last_update: SystemTime,
}

impl<T, M> Cache<T, M>
where
    T: Hash + Eq,
    M: Hash + Clone,
{
    pub fn new(timeout: Duration) -> Self {
        Cache::<T, M> {
            cache_items: HashMap::new(),
            timeout,
        }
    }

    // pub fn set_update(self, fun: Box<dyn Fn() -> M>) -> Self {
    //     Cache {
    //         update: Some(fun),
    //         ..self
    //     }
    // }

    pub fn get(&self, key: T) -> Option<M> {
        let item = self.cache_items.get(&key)?.to_owned();
        let elapsed = item.last_update.elapsed().ok()?;

        if elapsed > self.timeout {
            
        }

        Some(item.data.clone())
    }
}
