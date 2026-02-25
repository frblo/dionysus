use axum::{
    extract::{FromRef, FromRequestParts},
    http::{StatusCode, request::Parts},
};
use axum_extra::extract::CookieJar;

use crate::auth::AuthManager;

#[derive(Clone)]
pub struct Session {
    pub user_id: String,
}

impl Session {
    pub fn new(user_id: String) -> Self {
        Self { user_id }
    }
}

pub struct AuthSession(pub Session);

impl<S> FromRequestParts<S> for AuthSession
where
    AuthManager: FromRef<S>,
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

        let auth = AuthManager::from_ref(state);

        let session = auth
            .get_session(cookie.value())
            .await
            .ok_or((StatusCode::UNAUTHORIZED, "invalid session"))?;

        Ok(AuthSession(session))
    }
}
