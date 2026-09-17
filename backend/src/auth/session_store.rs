use std::{collections::HashMap, sync::Arc};

use tokio::sync::RwLock;

use crate::auth::{Session, UserId};
use crate::authz::GlobalRole;

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

    /// Patches the cached [`GlobalRole`] on every live session belonging to
    /// `user_id`. There can be multiple sessions for the same `user_id`.
    ///
    /// This ensures that someone whose role is updated doesn't need to log out
    /// and back in to see their updated permission in the frontend.
    pub async fn update_role_for_user(&self, user_id: UserId, role: GlobalRole) {
        let mut store = self.store.write().await;
        for session in store.values_mut() {
            if session.user_id == user_id {
                session.global_role = role;
            }
        }
    }
}
