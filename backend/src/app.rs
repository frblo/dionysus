use axum::{Router, routing::get};

use crate::{state::AppState, ws};

pub fn router(state: AppState) -> Router {
    Router::new()
        // single handler supports arbitrary rooms
        .route("/rooms/ws/{room_id}", get(ws::handler::ws_handler))
        .with_state(state)
}
