use axum::extract::{Path, Query};
use axum::response::{IntoResponse, Redirect};
use axum::{Json, extract::State};
use axum_extra::extract::{CookieJar, cookie::Cookie};
use serde::Deserialize;
use serde::Serialize;

use crate::auth::{AuthError, UserId};
use crate::authz::GlobalRole;
use crate::{auth::session::AuthSession, state::AppState};

#[derive(Serialize)]
pub struct ProviderList {
    pub providers: Vec<String>,
}

pub async fn providers(State(state): State<AppState>) -> Json<ProviderList> {
    let providers = state.auth.provider_ids();
    Json(ProviderList { providers })
}

#[derive(Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[cfg_attr(test, ts(export, export_to = "Me.ts"))]
pub struct Me {
    user: MeUser,
    global_role: GlobalRole,
}

#[derive(Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[cfg_attr(test, ts(export, export_to = "Me.ts"))]
pub struct MeUser {
    id: UserId,
    display_name: String,
}

pub async fn me(AuthSession(session): AuthSession) -> Json<Me> {
    Json(Me {
        user: MeUser {
            id: session.user_id,
            display_name: session.display_name,
        },
        global_role: session.global_role,
    })
}

#[derive(Deserialize)]
pub struct LoginQuery {
    pub provider: String,
}

pub async fn login(
    State(state): State<AppState>,
    Query(q): Query<LoginQuery>,
) -> Result<Redirect, AuthError> {
    let url = state.auth.start_login(&q.provider).await?;
    Ok(Redirect::to(&url))
}

#[derive(Deserialize)]
pub struct CallbackQuery {
    pub code: String,
    pub state: String,
}

pub async fn oidc_callback(
    State(state): State<AppState>,
    Path(provider): Path<String>,
    Query(q): Query<CallbackQuery>,
    jar: CookieJar,
) -> Result<impl IntoResponse, AuthError> {
    let session_id = state
        .auth
        .finish_login(&provider, q.code, q.state, &state.authz)
        .await?;

    let cookie = Cookie::build(("session", session_id))
        .path("/")
        .http_only(true)
        .secure(true)
        .same_site(axum_extra::extract::cookie::SameSite::Lax)
        .build();

    Ok((jar.add(cookie), Redirect::to("/")))
}
