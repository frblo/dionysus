use axum::{Router, routing::get};
use tower_http::services::{ServeDir, ServeFile};

use crate::{auth, state::AppState, ws};

pub fn router(state: AppState) -> Router {
    let serve_dir =
        ServeDir::new("./build").not_found_service(ServeFile::new("./build/index.html"));
    let router = Router::new()
        .nest("/auth", auth::router())
        .route("/rooms/ws/{room_id}", get(ws::handler::ws_handler))
        .fallback_service(serve_dir)
        .with_state(state);

    router
}
