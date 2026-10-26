use std::convert::Infallible;

use axum::extract::{Path, Query};
use axum::response::sse::{Event, KeepAlive};
use axum::response::{IntoResponse, Redirect, Sse};
use axum::{Json, extract::State};
use axum_extra::extract::{CookieJar, cookie::Cookie};
use futures_util::Stream;
use serde::Deserialize;
use serde::Serialize;
use tokio_stream::StreamExt;
use tokio_stream::wrappers::{BroadcastStream, errors::BroadcastStreamRecvError};

use crate::auth::{AuthError, LoginOutcome, PendingLinkInfo, Profile, UserEvent, UserId};
use crate::authz::{Actor, GlobalRole};
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

/// Payload of the `role-changed` SSE event.
#[derive(Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[cfg_attr(test, ts(export, export_to = "UserEvents.ts"))]
pub struct RoleChangedEvent {
    global_role: GlobalRole,
}

/// Payload of the `display-name-changed` SSE event.
#[derive(Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[cfg_attr(test, ts(export, export_to = "UserEvents.ts"))]
pub struct DisplayNameChangedEvent {
    display_name: String,
}

pub async fn me(
    AuthSession(session): AuthSession,
    State(state): State<AppState>,
) -> Result<Json<Me>, AuthError> {
    let user = state
        .auth
        .get_user(session.user_id)
        .await?
        .ok_or(AuthError::UserNotFound)?;
    let global_role = state.authz.global_role(&Actor::from(&session)).await?;

    Ok(Json(Me {
        user: MeUser {
            id: user.id,
            display_name: user.display_name,
        },
        global_role,
    }))
}

pub async fn profile(
    AuthSession(session): AuthSession,
    State(state): State<AppState>,
) -> Result<Json<Profile>, AuthError> {
    Ok(Json(state.auth.profile(session.user_id).await?))
}

#[derive(Deserialize)]
pub struct UpdateDisplayName {
    pub display_name: String,
}

pub async fn update_display_name(
    AuthSession(session): AuthSession,
    State(state): State<AppState>,
    Json(body): Json<UpdateDisplayName>,
) -> Result<Json<Profile>, AuthError> {
    state
        .auth
        .update_display_name(session.user_id, &body.display_name)
        .await?;
    Ok(Json(state.auth.profile(session.user_id).await?))
}

/// Informs the user when something has changed with their session.
pub async fn sse(
    AuthSession(session): AuthSession,
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let my_user_id = session.user_id;
    let rx_stream = BroadcastStream::new(state.auth.subscribe_events());

    let stream = rx_stream.filter_map(move |res| {
        let event = match res {
            Ok(UserEvent::RoleChanged { user_id, role }) if user_id == my_user_id => {
                Event::default()
                    .event("role-changed")
                    .json_data(RoleChangedEvent { global_role: role })
            }
            Ok(UserEvent::DisplayNameChanged {
                user_id,
                display_name,
            }) if user_id == my_user_id => Event::default()
                .event("display-name-changed")
                .json_data(DisplayNameChangedEvent { display_name }),
            Ok(_) => return None,
            Err(BroadcastStreamRecvError::Lagged(n)) => {
                tracing::warn!(missed_messages = n, "SSE recv lagged; requesting resync");
                Ok(Event::default().event("resync").data("resync"))
            }
        };

        match event {
            Ok(event) => Some(Ok(event)),
            Err(e) => {
                tracing::error!(error = %e, "failed to serialize user event");
                None
            }
        }
    });

    Sse::new(stream).keep_alive(KeepAlive::default())
}

#[derive(Deserialize)]
pub struct LoginQuery {
    pub provider: String,
    pub link_token: Option<String>,
}

pub async fn login(
    State(state): State<AppState>,
    Query(q): Query<LoginQuery>,
) -> Result<Redirect, AuthError> {
    let url = state.auth.start_login(&q.provider, q.link_token).await?;
    Ok(Redirect::to(&url))
}

pub async fn link_info(
    State(state): State<AppState>,
    Path(token): Path<String>,
) -> Result<Json<PendingLinkInfo>, AuthError> {
    Ok(Json(state.auth.pending_link_info(&token).await?))
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
    let outcome = state
        .auth
        .finish_login(&provider, q.code, q.state, &state.authz)
        .await?;

    match outcome {
        LoginOutcome::LoggedIn { session_id } => {
            let cookie = Cookie::build(("session", session_id))
                .path("/")
                .http_only(true)
                .secure(true)
                .same_site(axum_extra::extract::cookie::SameSite::Lax)
                .build();

            Ok((jar.add(cookie), Redirect::to("/")))
        }
        LoginOutcome::LinkRequired { token, .. } => {
            Ok((jar, Redirect::to(&format!("/link?token={token}"))))
        }
    }
}
