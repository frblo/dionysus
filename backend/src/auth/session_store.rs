use std::{collections::HashMap, sync::Arc};

use axum::extract::FromRef;

use tokio::sync::RwLock;

use crate::auth::Session;
use crate::state::AppState;

/// Stores current user sessions
///
/// Currently implemented with an in memory hashmap
#[derive(Clone)]
pub struct SessionStore {
    store: Arc<RwLock<HashMap<String, Session>>>,
}

impl SessionStore {
    pub fn new() -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn get(&self, session_id: &str) -> Option<Session> {
        self.store.read().await.get(session_id).cloned()
    }

    pub async fn insert(&self, session_id: String, session: Session) {
        self.store.write().await.insert(session_id, session);
    }

    pub async fn remove(&self, session_id: &str) {
        self.store.write().await.remove(session_id);
    }
}
