mod oidc;
mod routes;
mod session;
mod session_store;

use std::sync::Arc;

use axum::Router;
use axum::routing::{get, post};

use crate::auth::oidc::{OidcRegistry, PendingLoginStore};
use crate::state::AppState;

pub use session::AuthSession;
pub use session::Session;
pub use session_store::SessionStore;

#[derive(Clone)]
pub struct AuthManager {
    oidc: Arc<OidcRegistry>,
    pending: PendingLoginStore,
    sessions: SessionStore,
    external_base_url: String,
}

impl AuthManager {}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/me", get(routes::me))
        .route("/login", post(routes::login))
}
