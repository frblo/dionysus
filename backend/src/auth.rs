mod routes;
mod session_store;

use axum::Router;
use axum::routing::{get, post};

use crate::state::AppState;

pub use session_store::SessionStore;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/me", get(routes::me))
        .route("/login", post(routes::login))
}
