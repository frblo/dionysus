use axum::{Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::get};

use crate::{
    auth::AuthSession,
    rooms::{self, storage::RoomInfo},
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new().route("/list", get(list_rooms))
}

async fn list_rooms(
    AuthSession(session): AuthSession,
    State(state): State<AppState>,
) -> Result<Json<Vec<RoomInfo>>, rooms::Error> {
    let _ = session;
    Ok(Json(state.rooms.list_rooms().await?))
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
