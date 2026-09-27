use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get},
};
use uuid::Uuid;

use crate::{
    auth::{AuthError, AuthSession, UserId},
    authz::{self, Actor, RoomAction, RoomMember, RoomRole},
    rooms::{self, routes::RoomDelta},
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/members/{room_id}", get(list_members).post(add_member))
        .route("/members/{room_id}/{user_id}", delete(remove_member))
        .route("/members_search/{room_id}", get(search_candidates))
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("user not found")]
    UserNotFound,

    #[error("room not found")]
    RoomNotFound,

    #[error(transparent)]
    Room(#[from] rooms::Error),

    #[error(transparent)]
    Authz(#[from] authz::Error),

    #[error(transparent)]
    Auth(#[from] AuthError),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Error::UserNotFound => {
                tracing::warn!("member error: user not found");
                StatusCode::NOT_FOUND.into_response()
            }
            Error::RoomNotFound => {
                tracing::error!("member error: couldn't find room just modified");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
            Error::Room(e) => e.into_response(),
            Error::Authz(e) => e.into_response(),
            Error::Auth(e) => e.into_response(),
        }
    }
}

async fn list_members(
    AuthSession(session): AuthSession,
    State(state): State<AppState>,
    Path(room_id): Path<Uuid>,
) -> Result<Json<Vec<RoomMember>>, Error> {
    let actor = Actor::from(&session);
    state
        .authz
        .require_room(&actor, RoomAction::ManageMembers, room_id)
        .await?;

    Ok(Json(state.authz.list_members(room_id).await?))
}

#[derive(serde::Deserialize)]
struct GrantMember {
    user_id: UserId,
    role: RoomRole,
}

async fn add_member(
    AuthSession(session): AuthSession,
    State(state): State<AppState>,
    Path(room_id): Path<Uuid>,
    Json(payload): Json<GrantMember>,
) -> Result<(), Error> {
    let actor = Actor::from(&session);
    state
        .authz
        .require_room(&actor, RoomAction::ManageMembers, room_id)
        .await?;

    let target = state
        .auth
        .get_user(payload.user_id)
        .await?
        .ok_or(Error::UserNotFound)?;
    let subject = Actor {
        id: target.id,
        display_name: target.display_name,
    };

    state
        .authz
        .grant_role(room_id, &subject, payload.role)
        .await?;

    let info = state
        .rooms
        .room_info(room_id)
        .await?
        .ok_or(Error::RoomNotFound)?;
    let _ = state.rooms.gallery_tx.send(RoomDelta::Updated(info));
    Ok(())
}

async fn remove_member(
    AuthSession(session): AuthSession,
    State(state): State<AppState>,
    Path((room_id, user_id)): Path<(Uuid, UserId)>,
) -> Result<(), Error> {
    let actor = Actor::from(&session);
    state
        .authz
        .require_room(&actor, RoomAction::ManageMembers, room_id)
        .await?;

    let target = state
        .auth
        .get_user(user_id)
        .await?
        .ok_or(Error::UserNotFound)?;
    let subject = Actor {
        id: target.id,
        display_name: target.display_name,
    };

    state.authz.revoke_role(room_id, &subject).await?;

    let info = state
        .rooms
        .room_info(room_id)
        .await?
        .ok_or(Error::RoomNotFound)?;
    let _ = state.rooms.gallery_tx.send(RoomDelta::Updated(info));
    Ok(())
}

/// The shape returned by [`search_candidates`]. Contains only the information
/// necessary for the member search.
#[derive(Debug, serde::Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[cfg_attr(test, ts(export, export_to = "rooms/members/Candidate.ts"))]
pub struct Candidate {
    pub id: UserId,
    pub display_name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(serde::Deserialize)]
struct SearchQuery {
    q: String,
}

async fn search_candidates(
    AuthSession(session): AuthSession,
    State(state): State<AppState>,
    Path(room_id): Path<Uuid>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<Vec<Candidate>>, Error> {
    let actor = Actor::from(&session);
    state
        .authz
        .require_room(&actor, RoomAction::ManageMembers, room_id)
        .await?;

    let candidates = state.auth.search_users(&query.q).await?;
    Ok(Json(
        candidates
            .into_iter()
            .map(|u| Candidate {
                id: u.id,
                display_name: u.display_name,
                created_at: u.created_at,
            })
            .collect(),
    ))
}
