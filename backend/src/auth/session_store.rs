use std::{collections::HashMap, sync::Arc};

use tokio::sync::{RwLock, broadcast};

use crate::auth::{Session, UserId};
use crate::authz::GlobalRole;

#[derive(Debug, Clone)]
pub enum SessionDelta {
    RoleChanged {
        user_id: UserId,
        role: GlobalRole,
    },
    DisplayNameChanged {
        user_id: UserId,
        display_name: String,
    },
}

/// Stores current user sessions
///
/// Currently implemented with an in memory hashmap
#[derive(Clone)]
pub struct SessionStore {
    store: Arc<RwLock<HashMap<String, Session>>>,
    tx: broadcast::Sender<SessionDelta>,
}

impl SessionStore {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(16);
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
            tx,
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<SessionDelta> {
        self.tx.subscribe()
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
        {
            let mut store = self.store.write().await;
            for session in store.values_mut() {
                if session.user_id == user_id {
                    session.global_role = role;
                }
            }
        }
        let _ = self.tx.send(SessionDelta::RoleChanged { user_id, role });
    }

    /// Patches the cached `display_name` on every live session belonging to
    /// `user_id`. There can be multiple sessions for the same `user_id`.
    ///
    /// This ensures that someone whose display name is updated doesn't need
    /// to log out and back in to see it reflected in the frontend.
    pub async fn update_display_name_for_user(&self, user_id: UserId, display_name: &str) {
        {
            let mut store = self.store.write().await;
            for session in store.values_mut() {
                if session.user_id == user_id {
                    session.display_name = display_name.to_string();
                }
            }
        }
        let _ = self.tx.send(SessionDelta::DisplayNameChanged {
            user_id,
            display_name: display_name.to_string(),
        });
    }
}
