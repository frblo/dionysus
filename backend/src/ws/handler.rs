use axum::{
    extract::{Extension, Path, State, ws::WebSocketUpgrade},
    response::IntoResponse,
};
use tower_http::request_id::RequestId;
use uuid::Uuid;

use crate::ws;
use crate::{auth::AuthSession, state::AppState};
use crate::{
    authz::{self, Actor, Decision, RoomAction},
    rooms,
};

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

    let actor = Actor::from(&session);

    // Deny manually mapped to not found as to not reveal that a room exists
    // to someone who can't view it.
    match state
        .authz
        .require_room(&actor, RoomAction::View, room_id)
        .await
    {
        Ok(()) => {}
        Err(authz::Error::Denied) => return rooms::Error::NotFound.into_response(),
        Err(e) => return e.into_response(),
    }

    let room = match state.rooms.connect(room_id).await {
        Ok(r) => r,
        Err(e) => return e.into_response(),
    };

    let can_edit = match state
        .authz
        .authorize_room(&actor, RoomAction::Edit, room_id)
        .await
    {
        Ok(Decision::Allow) => true,
        Ok(Decision::Deny) => false,
        Err(e) => {
            tracing::warn!(error = ?e, %room_id, "authz error checking edit access");
            false
        }
    };

    let rooms = state.rooms.clone();
    let bcast = room.bcast.clone();

    ws.on_upgrade(move |socket| ws::peer::peer(socket, rooms, bcast, room_id, request_id, can_edit))
}
