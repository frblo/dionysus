use axum::{Json, extract::State, http::StatusCode, response::Redirect};
use axum_extra::extract::{CookieJar, cookie::Cookie};
use rand::{Rng, distr::Alphanumeric};
use serde::Deserialize;
use serde::Serialize;

use crate::{
    auth::session_store::{AuthSession, Session},
    state::AppState,
};

#[derive(Serialize)]
pub struct Me {
    user_id: String
}

pub async fn me(AuthSession(session): AuthSession) -> Json<Me> {
    Json(Me { user_id: session.user_id })
}

#[derive(Deserialize)]
pub struct LoginBody {
    pub password: String,
}

pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(body): Json<LoginBody>,
) -> Result<(CookieJar, StatusCode), (StatusCode, &'static str)> {
    if !(body.password == "manus27") {
        return Err((StatusCode::UNAUTHORIZED, "invalid credentials"));
    }

    let session_id: String = rand::rng()
        .sample_iter(&Alphanumeric)
        .take(48)
        .map(char::from)
        .collect();

    state
        .sessions
        .insert(session_id.clone(), Session::new("shared".to_string()))
        .await;

    let cookie = Cookie::build(("session", session_id))
        .path("/")
        .http_only(true)
        .secure(true)
        .same_site(axum_extra::extract::cookie::SameSite::Lax)
        .build();

    let jar = jar.add(cookie);

    Ok((jar, StatusCode::NO_CONTENT))
}
