use axum::{Router, routing::get};
use tower_http::services::{ServeDir, ServeFile};

use crate::{auth, state::AppState, ws};

pub fn router(state: AppState, serve_frontend: bool) -> Router {
    let mut router = Router::new()
        .nest("/auth", auth::router())
        .route("/rooms/ws/{room_id}", get(ws::handler::ws_handler))
        .with_state(state);

    if serve_frontend {
        let serve_dir =
            ServeDir::new("./build").not_found_service(ServeFile::new("./build/index.html"));

        router = router.fallback_service(serve_dir);
    }

    router
}
