use std::convert::Infallible;

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::{
        IntoResponse, Sse,
        sse::{Event, KeepAlive},
    },
    routing::{get, post},
};
use futures_util::Stream;
use tokio_stream::{StreamExt, wrappers::BroadcastStream};
use uuid::Uuid;

use crate::{
    auth::AuthSession,
    rooms::{self, storage::RoomInfo},
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/list", get(list_rooms))
        .route("/room_info/{room_id}", get(room_info))
        .route("/rename/{room_id}/{room_name}", post(rename))
        .route("/create/{room_name}", post(create))
        .route("/delete/{room_id}", post(delete))
        .route("/sse", get(sse_handler))
}

#[derive(Debug, Clone)]
pub enum RoomDelta {
    Added(RoomInfo),
    Updated(RoomInfo),
    Removed(Uuid),
}

async fn sse_handler(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx_stream = BroadcastStream::new(state.gallary_tx.subscribe());

    let stream = rx_stream.filter_map(|res| {
        let delta = res.ok()?;

        let event = match delta {
            RoomDelta::Added(room_info) => Event::default()
                .event("room-added")
                .data(serde_json::to_string(&room_info).unwrap()),
            RoomDelta::Updated(room_info) => Event::default()
                .event("room-updated")
                .data(serde_json::to_string(&room_info).unwrap()),
            RoomDelta::Removed(room_id) => Event::default()
                .event("room-removed")
                .data(room_id.to_string()),
        };

        Some(Ok(event))
    });

    Sse::new(stream).keep_alive(KeepAlive::default())
}

async fn list_rooms(
    AuthSession(_session): AuthSession,
    State(state): State<AppState>,
) -> Result<Json<Vec<RoomInfo>>, rooms::Error> {
    Ok(Json(state.rooms.list_rooms().await?))
}

async fn room_info(
    AuthSession(_session): AuthSession,
    State(state): State<AppState>,
    Path(room_id): Path<Uuid>,
) -> Result<Json<RoomInfo>, rooms::Error> {
    match state.rooms.room_info(room_id).await? {
        Some(info) => Ok(Json(info)),
        None => Err(rooms::Error::NotFound),
    }
}

async fn rename(
    AuthSession(_session): AuthSession,
    State(state): State<AppState>,
    Path((room_id, room_name)): Path<(Uuid, String)>,
) -> Result<(), rooms::Error> {
    let info = state.rooms.rename_room(room_id, &room_name).await?;
    let _ = state.gallary_tx.send(RoomDelta::Updated(info));
    Ok(())
}

async fn create(
    AuthSession(_session): AuthSession,
    State(state): State<AppState>,
    Path(room_name): Path<String>,
) -> Result<Json<Uuid>, rooms::Error> {
    let info = state.rooms.create_room(&room_name).await?;
    let _ = state.gallary_tx.send(RoomDelta::Added(info.clone()));
    Ok(Json(info.room_id))
}

async fn delete(
    AuthSession(_session): AuthSession,
    State(state): State<AppState>,
    Path(room_id): Path<Uuid>,
) -> Result<(), rooms::Error> {
    state.rooms.delete_room(room_id).await?;
    let _ = state.gallary_tx.send(RoomDelta::Removed(room_id));
    Ok(())
}

impl IntoResponse for rooms::Error {
    fn into_response(self) -> axum::response::Response {
        tracing::warn!(error = ?self, "rooms error");

        let status = match &self {
            rooms::Error::NotFound => StatusCode::NOT_FOUND,
            rooms::Error::AlreadyExists => StatusCode::CONFLICT,
            rooms::Error::InvalidArgument(_) => StatusCode::BAD_REQUEST,
            rooms::Error::Decoding(_) | rooms::Error::Backend { .. } => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };

        (status, self.to_string()).into_response()
    }
}
