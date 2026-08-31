use axum::{
    extract::{Extension, Path, State, ws::WebSocketUpgrade},
    http::StatusCode,
    response::IntoResponse,
};
use tower_http::request_id::RequestId;
use uuid::Uuid;

use crate::ws;
use crate::{auth::AuthSession, state::AppState};

#[tracing::instrument(skip_all, fields(room_id = %room_id, user_id = %session.user_id))]
pub async fn ws_handler(
    AuthSession(session): AuthSession,
    ws: WebSocketUpgrade,
    Path(room_id): Path<Uuid>,
    State(state): State<AppState>,
    Extension(request_id): Extension<RequestId>,
) -> impl IntoResponse {
    let request_id = request_id
        .header_value()
        .to_str()
        .unwrap_or("invalid")
        .to_owned();

    tracing::info!("handling websocket upgrade request");

    let room = match state.rooms.connect(room_id).await {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!(error = %e, "failed to connect to room");
            return StatusCode::NOT_FOUND.into_response();
        }
    };

    let rooms = state.rooms.clone();
    let bcast = room.bcast.clone();

    ws.on_upgrade(move |socket| ws::peer::peer(socket, rooms, bcast, room_id, request_id))
}
