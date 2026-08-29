use axum::{
    extract::{Path, State, ws::WebSocketUpgrade},
    http::StatusCode,
    response::IntoResponse,
};

use crate::ws;
use crate::{auth::AuthSession, state::AppState};

#[tracing::instrument(skip_all, fields(room_id = %room_id, user_id = %session.user_id))]
pub async fn ws_handler(
    AuthSession(session): AuthSession,
    ws: WebSocketUpgrade,
    Path(room_id): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    tracing::info!("handling websocket upgrade request");

    let room = match state.rooms.connect(&room_id).await {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!(error = %e, "failed to connect to room");
            return StatusCode::NOT_FOUND.into_response();
        }
    };

    let rooms = state.rooms.clone();
    let bcast = room.bcast.clone();

    ws.on_upgrade(move |socket| ws::peer::peer(socket, rooms, bcast, room_id))
}
