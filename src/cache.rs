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

    pub fn update(&mut self, key: T, value: M) -> Option<()> {
        self.cache_items.insert(
            key,
            CacheItem {
                data: value,
                last_update: SystemTime::now(),
            },
        )?;

        Some(())
    }

    pub fn get(&self, key: T) -> Option<M> {
        let item = self.cache_items.get(&key)?.to_owned();
        let elapsed = item.last_update.elapsed().ok()?;

        if elapsed > self.timeout {
            return None;
        }

        Some(item.data.clone())
    }
}
