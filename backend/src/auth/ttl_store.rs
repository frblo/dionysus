//! A generic TTL-bounded map used to track short-lived, single-use state
//! across a round trip through an external system (e.g. an OIDC provider
//! redirect).
//!
//! Currently implemented as an in-memory map.
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::RwLock;
use tokio::time::Instant;

#[derive(Debug)]
struct Entry<T> {
    created_at: Instant,
    value: T,
}

#[derive(Debug)]
pub struct TtlStore<T> {
    ttl: Duration,
    inner: Arc<RwLock<HashMap<String, Entry<T>>>>,
}

//  Written instaed of `#[derive(Clone)]` since the derive would require
//  `T: Clone` but `T` is inside an `Arc`, which is `Clone` regardless of `T`.
impl<T> Clone for TtlStore<T> {
    fn clone(&self) -> Self {
        Self {
            ttl: self.ttl,
            inner: self.inner.clone(),
        }
    }
}

impl<T> TtlStore<T> {
    pub fn new(ttl: Duration) -> Self {
        Self {
            ttl,
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Insert a value under `key`.
    pub async fn insert(&self, key: String, value: T) {
        self.inner.write().await.insert(
            key,
            Entry {
                created_at: Instant::now(),
                value,
            },
        );
    }

    /// Remove and return the value for `key`.
    ///
    /// If the entry's ttl has elapsed, returns [`None`] as if it didn't
    /// exist.
    pub async fn take(&self, key: &str) -> Option<T> {
        let mut map = self.inner.write().await;
        let entry = map.remove(key)?;
        if entry.created_at.elapsed() > self.ttl {
            return None;
        }
        Some(entry.value)
    }

    /// Garbage collect the store, removing any entries whose ttl has
    /// elapsed.
    pub async fn gc(&self) {
        self.inner
            .write()
            .await
            .retain(|_, e| e.created_at.elapsed() < self.ttl);
    }
}

impl<T: Clone> TtlStore<T> {
    /// Return a clone of the value for `key`, without removing it.
    ///
    /// If the entry's ttl has elapsed, returns [`None`] as if it didn't
    /// exist.
    pub async fn get(&self, key: &str) -> Option<T> {
        let map = self.inner.read().await;
        let entry = map.get(key)?;
        if entry.created_at.elapsed() > self.ttl {
            return None;
        }
        Some(entry.value.clone())
    }
}
