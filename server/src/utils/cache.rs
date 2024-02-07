use crate::{CacheError, Identity};
use cached::{Cached, TimedSizedCache};
use futures_util::stream::TryChunks;
use mongodb::Cursor;
use std::sync::{Arc, Mutex};

pub struct MyCache {
    pub store: Arc<Mutex<TimedSizedCache<String, TryChunks<Cursor<Identity>>>>>,
}

impl Default for MyCache {
    fn default() -> Self {
        MyCache {
            store: Arc::new(Mutex::new(
                TimedSizedCache::with_size_and_lifespan_and_refresh(1000, 20, true),
            )),
        }
    }
}

impl MyCache {
    pub fn get(&self, key: String) -> Result<Option<TryChunks<Cursor<Identity>>>, CacheError> {
        let mut guard_cache = self.store.lock().map_err(|err| {
            tracing::error!("Program is too incompetent to get a mutex: {err:?}");
            CacheError::Mutex
        })?;
        let old_cursor = guard_cache.cache_remove(&key);
        Ok(old_cursor)
    }

    pub fn set(
        &self,
        key: String,
        value: TryChunks<Cursor<Identity>>,
    ) -> Result<Option<TryChunks<Cursor<Identity>>>, CacheError> {
        let mut guard_cache = self.store.lock().map_err(|err| {
            tracing::error!("Program is too incompetent to get a mutex: {err:?}");
            CacheError::Mutex
        })?;
        let old_cursor = guard_cache.cache_set(key, value);
        Ok(old_cursor)
    }
}
