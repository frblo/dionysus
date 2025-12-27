use axum::{
    extract::{Path, State, ws::WebSocketUpgrade},
    http::StatusCode,
    response::IntoResponse,
};

use crate::state::AppState;
use crate::ws;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Path(room_id): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    println!("Request for {room_id} handler!");
    let bcast = match state.rooms.connect(&room_id).await {
        Ok(bc) => bc,
        Err(e) => {
            println!("{:?}", e);
            return StatusCode::NOT_FOUND.into_response();
        }
    };

    ws.on_upgrade(move |socket| ws::peer::peer(socket, bcast, room_id))
}
