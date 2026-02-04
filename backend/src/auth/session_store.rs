use axum::{
    extract::{FromRef, FromRequestParts},
    http::{StatusCode, request::Parts},
};
use tokio::sync::RwLock;

use axum_extra::extract::CookieJar;

use std::{collections::HashMap, sync::Arc};

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

impl FromRef<AppState> for SessionStore {
    fn from_ref(input: &AppState) -> Self {
        input.sessions.clone()
    }
}

#[derive(Clone)]
pub struct Session {
    /// Temp showing how actual data will be placed later
    user_id: String,
}

impl Session {
    pub fn new(user_id: String) -> Self {
        Self { user_id }
    }
}

pub struct AuthSession(pub Session);

impl<S> FromRequestParts<S> for AuthSession
where
    SessionStore: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let jar = CookieJar::from_request_parts(parts, state)
            .await
            .map_err(|_| (StatusCode::UNAUTHORIZED, "missing cookies"))?;

        let cookie = jar
            .get("session")
            .ok_or((StatusCode::UNAUTHORIZED, "missing session"))?;

        let store = SessionStore::from_ref(state);

        let session = store
            .get(cookie.value())
            .await
            .ok_or((StatusCode::UNAUTHORIZED, "invalid session"))?;

        Ok(AuthSession(session))
    }
}
