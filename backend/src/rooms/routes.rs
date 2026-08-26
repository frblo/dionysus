use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
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
    Path(room_id): Path<Uuid>,
    Path(room_name): Path<String>,
) -> Result<(), rooms::Error> {
    Ok(state.rooms.rename_room(room_id, &room_name).await?)
}

async fn create(
    AuthSession(_session): AuthSession,
    State(state): State<AppState>,
    Path(room_name): Path<String>,
) -> Result<Json<Uuid>, rooms::Error> {
    Ok(Json(state.rooms.create_room(&room_name).await?))
}

async fn delete(
    AuthSession(_session): AuthSession,
    State(state): State<AppState>,
    Path(room_id): Path<Uuid>,
) -> Result<(), rooms::Error> {
    Ok(state.rooms.delete_room(room_id).await?)
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
