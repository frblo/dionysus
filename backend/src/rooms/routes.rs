use std::{collections::HashMap, convert::Infallible};

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
    authz::{self, Actor, AuthzManager, CollectionAction, RoomAction, RoomRole, RoomScope},
    rooms::{self, storage::RoomInfo as StorageRoomInfo},
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
    Added(StorageRoomInfo),
    Updated(StorageRoomInfo),
    Removed(Uuid),
    Resync,
}

/// The room shape sent to clients.
///
/// Hides some storage fields and includes the callers's own role.
#[derive(Debug, Clone, serde::Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[cfg_attr(test, ts(export, export_to = "RoomInfo.ts"))]
pub struct RoomInfo {
    pub room_id: Uuid,
    pub room_name: String,
    pub role: Option<RoomRole>,
}

impl RoomInfo {
    fn new(info: StorageRoomInfo, role: Option<RoomRole>) -> Self {
        Self {
            room_id: info.room_id,
            room_name: info.room_name,
            role,
        }
    }
}

/// Per SEE conection calculation of if [`RoomDelta`] update can be seen and
/// which role this `actor` has.
async fn viewer_room_info(
    authz: &AuthzManager,
    actor: &Actor,
    info: StorageRoomInfo,
) -> Option<RoomInfo> {
    match authz
        .authorize_room(actor, RoomAction::View, info.room_id)
        .await
    {
        Ok(authz::Decision::Allow) => {}
        Ok(authz::Decision::Deny) => return None,
        Err(e) => {
            tracing::warn!(
                error = ?e,
                room_id = %info.room_id,
                "authz backend error filtering SSE delta"
            );
            return None;
        }
    }

    let role = match authz.role_in_room(info.room_id, actor).await {
        Ok(role) => role,
        Err(e) => {
            tracing::warn!(
                error = ?e,
                room_id = %info.room_id,
                "authz backend error resolving SSE role"
            );
            None
        }
    };

    Some(RoomInfo::new(info, role))
}

async fn sse_handler(
    AuthSession(session): AuthSession,
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let actor = Actor::from(&session);
    let rx_stream = BroadcastStream::new(state.rooms.gallery_tx.subscribe());

    let stream = rx_stream
        .then(move |res| {
            let actor = actor.clone();
            let authz = state.authz.clone();
            async move {
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
                    RoomDelta::Added(info) => {
                        let room_info = viewer_room_info(&authz, &actor, info).await?;

                        let Some(data) = serde_json::to_string(&room_info)
                            .map_err(
                                |e| tracing::error!(error = ?e, "failed to serialize room-added"),
                            )
                            .ok()
                        else {
                            return None;
                        };
                        Event::default().event("room-added").data(data)
                    }
                    RoomDelta::Updated(info) => {
                        let room_info = viewer_room_info(&authz, &actor, info).await?;

                        let Some(data) = serde_json::to_string(&room_info)
                            .map_err(
                                |e| tracing::error!(error = ?e, "failed to serialize room-updated"),
                            )
                            .ok()
                        else {
                            return None;
                        };
                        Event::default().event("room-updated").data(data)
                    }
                    // Not gated, since:
                    // a) authz against a room that doesn't exist.
                    // b) only carries a bare Uuid of a room that doesn't exist.
                    RoomDelta::Removed(room_id) => Event::default()
                        .event("room-removed")
                        .data(room_id.to_string()),
                    RoomDelta::Resync => Event::default().event("resync"),
                };

                Some(Ok(event))
            }
        })
        .filter_map(|x| x);

    Sse::new(stream).keep_alive(KeepAlive::default())
}

async fn list_rooms(
    AuthSession(session): AuthSession,
    State(state): State<AppState>,
) -> Result<Json<Vec<RoomInfo>>, rooms::Error> {
    let actor = Actor::from(&session);
    let scope = state.authz.rooms_matching(&actor, RoomAction::View).await?;
    let membership: HashMap<Uuid, RoomRole> = state
        .authz
        .member_rooms(&actor)
        .await?
        .into_iter()
        .collect();

    let rooms = state.rooms.list_rooms().await?;
    let rooms = match scope {
        RoomScope::All => rooms,
        RoomScope::Only(ids) => rooms
            .into_iter()
            .filter(|r| ids.contains(&r.room_id))
            .collect(),
    };

    let rooms = rooms
        .into_iter()
        .map(|info| {
            let role = membership.get(&info.room_id).copied();
            RoomInfo::new(info, role)
        })
        .collect();

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

    let role = state.authz.role_in_room(room_id, &actor).await?;
    match state.rooms.room_info(room_id).await? {
        Some(info) => Ok(Json(RoomInfo::new(info, role))),
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
