//! A generic TTL-bounded map for short-lived state, such as single-use state
//! tracked across a round trip through an external system (e.g. an OIDC
//! provider redirect) or a cached answer from one.
//!
//! Currently implemented as an in-memory map.
use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::RwLock;
use tokio::time::Instant;

#[derive(Debug)]
struct Entry<V> {
    created_at: Instant,
    value: V,
}

#[derive(Debug)]
pub struct TtlStore<K, V> {
    ttl: Duration,
    inner: Arc<RwLock<HashMap<K, Entry<V>>>>,
}

//  Written instaed of `#[derive(Clone)]` since the derive would require
//  `K: Clone` and `V: Clone` but they are inside an `Arc`, which is `Clone`
//  regardless.
impl<K, V> Clone for TtlStore<K, V> {
    fn clone(&self) -> Self {
        Self {
            ttl: self.ttl,
            inner: self.inner.clone(),
        }
    }
}

impl<K: Eq + Hash, V> TtlStore<K, V> {
    pub fn new(ttl: Duration) -> Self {
        Self {
            ttl,
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Insert a value under `key`.
    ///
    /// Returns the value it replaced, even if that one's ttl had already
    /// elapsed (and it hadn't been garbage collected yet).
    pub async fn insert(&self, key: K, value: V) -> Option<V> {
        self.inner
            .write()
            .await
            .insert(
                key,
                Entry {
                    created_at: Instant::now(),
                    value,
                },
            )
            .map(|e| e.value)
    }

    /// Remove and return the value for `key`.
    ///
    /// If the entry's ttl has elapsed, returns [`None`] as if it didn't
    /// exist.
    pub async fn take<Q>(&self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Eq + Hash + ?Sized,
    {
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

impl<K: Eq + Hash, V: Clone> TtlStore<K, V> {
    /// Return a clone of the value for `key`, without removing it.
    ///
    /// If the entry's ttl has elapsed, returns [`None`] as if it didn't
    /// exist.
    pub async fn get<Q>(&self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Eq + Hash + ?Sized,
    {
        let map = self.inner.read().await;
        let entry = map.get(key)?;
        if entry.created_at.elapsed() > self.ttl {
            return None;
        }
        Some(entry.value.clone())
    }
}
