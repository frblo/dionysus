use std::convert::Infallible;

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::{
        IntoResponse, Sse,
        sse::{Event, KeepAlive},
    },
    routing::{delete, get, post},
};
use futures_util::Stream;
use tokio_stream::{
    StreamExt,
    wrappers::{BroadcastStream, errors::BroadcastStreamRecvError},
};
use uuid::Uuid;

use crate::{
    auth::AuthSession,
    authz::{self, Actor, CollectionAction, RoomAction, RoomScope},
    rooms::{self, storage::RoomInfo},
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/list", get(list_rooms))
        .route("/room_info/{room_id}", get(room_info))
        .route("/rename/{room_id}", post(rename))
        .route("/create/{room_name}", post(create))
        .route("/delete/{room_id}", delete(remove))
        .route("/sse", get(sse_handler))
}

#[derive(Debug, Clone)]
pub enum RoomDelta {
    Added(RoomInfo),
    Updated(RoomInfo),
    Removed(Uuid),
    Resync,
}

async fn sse_handler(
    AuthSession(_session): AuthSession,
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx_stream = BroadcastStream::new(state.rooms.gallery_tx.subscribe());

    let stream = rx_stream.filter_map(|res| {
        let delta = match res {
            Ok(rd) => rd,
            Err(BroadcastStreamRecvError::Lagged(n)) => {
                tracing::warn!(
                    missed_messages = n,
                    "SSE recv lagged; requesting room list resync"
                );
                RoomDelta::Resync
            }
        };

        let event = match delta {
            RoomDelta::Added(room_info) => {
                let Some(data) = serde_json::to_string(&room_info)
                    .map_err(|e| tracing::error!(error = ?e, "failed to serialize room-added"))
                    .ok()
                else {
                    return None;
                };
                Event::default().event("room-added").data(data)
            }
            RoomDelta::Updated(room_info) => {
                let Some(data) = serde_json::to_string(&room_info)
                    .map_err(|e| tracing::error!(error = ?e, "failed to serialize room-updated"))
                    .ok()
                else {
                    return None;
                };
                Event::default().event("room-updated").data(data)
            }
            RoomDelta::Removed(room_id) => Event::default()
                .event("room-removed")
                .data(room_id.to_string()),
            RoomDelta::Resync => Event::default().event("resync"),
        };

        Some(Ok(event))
    });

    Sse::new(stream).keep_alive(KeepAlive::default())
}

async fn list_rooms(
    AuthSession(session): AuthSession,
    State(state): State<AppState>,
) -> Result<Json<Vec<RoomInfo>>, rooms::Error> {
    let actor = Actor::from(&session);
    let scope = state.authz.rooms_matching(&actor, RoomAction::View).await?;

    let rooms = state.rooms.list_rooms().await?;
    let rooms = match scope {
        RoomScope::All => rooms,
        RoomScope::Only(ids) => rooms
            .into_iter()
            .filter(|r| ids.contains(&r.room_id))
            .collect(),
    };

    Ok(Json(rooms))
}

async fn room_info(
    AuthSession(session): AuthSession,
    State(state): State<AppState>,
    Path(room_id): Path<Uuid>,
) -> Result<Json<RoomInfo>, rooms::Error> {
    let actor = Actor::from(&session);

    // Deny manually mapped to not found as to not reveal that a room exists
    // to someone who can't view it.
    match state
        .authz
        .require_room(&actor, RoomAction::View, room_id)
        .await
    {
        Ok(()) => {}
        Err(authz::Error::Denied) => return Err(rooms::Error::NotFound),
        Err(e) => return Err(e.into()),
    }

    match state.rooms.room_info(room_id).await? {
        Some(info) => Ok(Json(info)),
        None => Err(rooms::Error::NotFound),
    }
}

#[derive(serde::Deserialize)]
struct RenameRoom {
    name: String,
}

async fn rename(
    AuthSession(session): AuthSession,
    State(state): State<AppState>,
    Path(room_id): Path<Uuid>,
    Json(payload): Json<RenameRoom>,
) -> Result<(), rooms::Error> {
    let actor = Actor::from(&session);
    state
        .authz
        .require_room(&actor, RoomAction::Rename, room_id)
        .await?;

    let info = state.rooms.rename_room(room_id, &payload.name).await?;
    let _ = state.rooms.gallery_tx.send(RoomDelta::Updated(info));
    Ok(())
}

async fn create(
    AuthSession(session): AuthSession,
    State(state): State<AppState>,
    Path(room_name): Path<String>,
) -> Result<Json<Uuid>, rooms::Error> {
    let actor = Actor::from(&session);
    state
        .authz
        .require_collection(&actor, CollectionAction::CreateRoom)
        .await?;

    let info = state.rooms.create_room(&room_name).await?;

    if let Err(e) = state.authz.on_room_created(&actor, info.room_id).await {
        if let Err(cleanup_err) = state.rooms.delete_room(info.room_id).await {
            tracing::error!(
                room_id = %info.room_id,
                error = ?cleanup_err,
                "failed to clean up room after on_room_created failure"
            );
        }
        return Err(e.into());
    }

    let _ = state.rooms.gallery_tx.send(RoomDelta::Added(info.clone()));
    Ok(Json(info.room_id))
}

async fn remove(
    AuthSession(session): AuthSession,
    State(state): State<AppState>,
    Path(room_id): Path<Uuid>,
) -> Result<(), rooms::Error> {
    let actor = Actor::from(&session);
    state
        .authz
        .require_room(&actor, RoomAction::Delete, room_id)
        .await?;

    state.rooms.delete_room(room_id).await?;
    let _ = state.rooms.gallery_tx.send(RoomDelta::Removed(room_id));
    Ok(())
}

impl From<authz::Error> for rooms::Error {
    fn from(value: authz::Error) -> Self {
        match value {
            authz::Error::Denied => rooms::Error::Forbidden,
            authz::Error::Invalid(msg) => rooms::Error::InvalidArgument(msg),
            authz::Error::Unsupported => rooms::Error::Forbidden,
            authz::Error::Backend { source } => rooms::Error::Backend { source },
        }
    }
}

impl IntoResponse for rooms::Error {
    fn into_response(self) -> axum::response::Response {
        tracing::warn!(error = ?self, "rooms error");

        let status = match &self {
            rooms::Error::NotFound => StatusCode::NOT_FOUND,
            rooms::Error::Forbidden => StatusCode::FORBIDDEN,
            rooms::Error::InvalidArgument(_) => StatusCode::BAD_REQUEST,
            rooms::Error::Decoding(_) | rooms::Error::Backend { .. } => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };

        (status, self.to_string()).into_response()
    }
}
