use axum::{
    extract::{FromRef, FromRequestParts},
    http::{StatusCode, request::Parts},
};
use axum_extra::extract::CookieJar;

use crate::auth::{AuthManager, UserId};
use crate::authz::GlobalRole;

#[derive(Clone)]
pub struct Session {
    pub user_id: UserId,
    pub display_name: String,
    pub global_role: GlobalRole,
}

impl Session {
    pub fn new(user_id: UserId, display_name: String, global_role: GlobalRole) -> Self {
        Self {
            user_id,
            display_name,
            global_role,
        }
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
            .map_err(|_| {
                tracing::debug!("rejected request: missing cookies");
                (StatusCode::UNAUTHORIZED, "missing cookies")
            })?;

        let cookie = jar.get("session").ok_or_else(|| {
            tracing::debug!("rejected request: missing session cookie");
            (StatusCode::UNAUTHORIZED, "missing session")
        })?;

        let auth = AuthManager::from_ref(state);

        let session = auth.get_session(cookie.value()).await.ok_or_else(|| {
            tracing::debug!("rejected request: invalid or expired session");
            (StatusCode::UNAUTHORIZED, "invalid session")
        })?;

        Ok(AuthSession(session))
    }
}
