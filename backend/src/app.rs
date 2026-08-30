use axum::{Router, body::Body, http::Request, routing::get};
use tower::ServiceBuilder;
use tower_http::{
    ServiceBuilderExt,
    request_id::{MakeRequestUuid, RequestId},
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};
use tracing::Span;

use crate::{auth, rooms, state::AppState, ws};

pub fn router(state: AppState) -> Router {
    let serve_dir =
        ServeDir::new("./build").not_found_service(ServeFile::new("./build/index.html"));

    let app = Router::new()
        .nest("/auth", auth::router())
        .nest("/api/rooms", rooms::router())
        .route("/rooms/ws/{room_id}", get(ws::handler::ws_handler))
        .fallback_service(serve_dir)
        .with_state(state);

    with_observability(app)
}

/// Layer request-id assignment plus a tracing span onto every HTTP request.
///
/// Composed as a single `ServiceBuilder` so the ordering is explicit: the first
/// `.layer()` is outermost and sees the request first. `SetRequestId` must run
/// before `TraceLayer` so the span can read the `RequestId` extension it
/// inserts; `PropagateRequestId` copies the id onto the response on the way
/// back out.
fn with_observability(router: Router) -> Router {
    router.layer(
        ServiceBuilder::new()
            .set_x_request_id(MakeRequestUuid)
            .layer(TraceLayer::new_for_http().make_span_with(make_http_span))
            .propagate_x_request_id(),
    )
}

fn make_http_span(req: &Request<Body>) -> Span {
    let request_id = req
        .extensions()
        .get::<RequestId>()
        .and_then(|id| id.header_value().to_str().ok())
        .unwrap_or("unknown");
    tracing::info_span!(
        "http_request",
        method = %req.method(),
        uri = %req.uri(),
        request_id = %request_id,
    )
}
