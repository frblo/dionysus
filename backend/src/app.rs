use axum::{Router, routing::get};

use crate::{auth, state::AppState, ws};

pub fn router(state: AppState) -> Router {
    Router::new()
        .nest("/auth", auth::router())
        .route("/rooms/ws/{room_id}", get(ws::handler::ws_handler))
        .with_state(state)
}
