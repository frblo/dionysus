use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, patch},
};

use crate::{
    auth::{AuthError, AuthSession, UserId},
    authz::{self, Actor, GlobalRole},
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/users", get(list_users))
        .route("/users/{user_id}/role", patch(set_role))
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("user not found")]
    UserNotFound,

    #[error(transparent)]
    Authz(#[from] authz::Error),

    #[error(transparent)]
    Auth(#[from] AuthError),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Error::UserNotFound => {
                tracing::warn!("admin error: user not found");
                StatusCode::NOT_FOUND.into_response()
            }
            Error::Authz(e) => e.into_response(),
            Error::Auth(e) => e.into_response(),
        }
    }
}

/// The representation of a user for the Admin dashboard.
#[derive(Debug, serde::Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[cfg_attr(test, ts(export, export_to = "admin/User.ts"))]
pub struct User {
    pub id: UserId,
    pub display_name: String,
    pub global_role: GlobalRole,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// The list of every known user, and whether [`set_role`] will work.
///
/// When no [`RoleStore`](crate::authz::RoleStore) is configured that means
/// that an external tools manages these permissions, and [`set_role`] won't
/// work.
#[derive(Debug, serde::Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[cfg_attr(test, ts(export, export_to = "admin/Users.ts"))]
pub struct Users {
    pub users: Vec<User>,
    pub role_management_enabled: bool,
}

async fn list_users(
    AuthSession(session): AuthSession,
    State(state): State<AppState>,
) -> Result<Json<Users>, Error> {
    let actor = Actor::from(&session);
    state.authz.require_admin(&actor).await?;

    let identities = state.auth.list_users().await?;
    let mut users = Vec::with_capacity(identities.len());
    for u in identities {
        let subject = Actor {
            id: u.id,
            display_name: u.display_name.clone(),
        };
        let global_role = state.authz.global_role(&subject).await?;
        users.push(User {
            id: u.id,
            display_name: u.display_name,
            global_role,
            created_at: u.created_at,
        });
    }

    Ok(Json(Users {
        users,
        role_management_enabled: state.authz.role_management_enabled(),
    }))
}

#[derive(serde::Deserialize)]
struct SetRole {
    role: GlobalRole,
}

async fn set_role(
    AuthSession(session): AuthSession,
    State(state): State<AppState>,
    Path(user_id): Path<UserId>,
    Json(payload): Json<SetRole>,
) -> Result<(), Error> {
    let actor = Actor::from(&session);
    state.authz.require_admin(&actor).await?;

    let target = state
        .auth
        .get_user(user_id)
        .await?
        .ok_or(Error::UserNotFound)?;
    let subject = Actor {
        id: target.id,
        display_name: target.display_name,
    };

    state.authz.set_global_role(&subject, payload.role).await?;
    state
        .auth
        .update_session_role(subject.id, payload.role)
        .await;
    Ok(())
}
