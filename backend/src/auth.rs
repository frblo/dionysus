mod identity;
mod linking;
mod oidc;
mod routes;
mod session;
mod session_store;
mod ttl_store;

use std::sync::Arc;
use std::time::Duration;

use axum::extract::FromRef;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use openidconnect::core::CoreResponseType;
use openidconnect::reqwest::async_http_client;
use openidconnect::{AuthenticationFlow, AuthorizationCode, CsrfToken, Nonce, RedirectUrl};
use rand::{Rng, distr::Alphanumeric};
use serde::Serialize;
use thiserror::Error;

use crate::auth::linking::{PendingLink, PendingLinkStore};
use crate::auth::oidc::{OidcRegistry, PendingLogin, PendingLoginStore};
use crate::authz::{Actor, AuthzManager};
use crate::config::Config;
use crate::db::Db;
use crate::state::AppState;

pub use identity::{IdentityStore, User, UserId};
pub use session::AuthSession;
pub use session::Session;
pub use session_store::{SessionDelta, SessionStore};

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("unknown provider")]
    UnknownProvider,

    #[error("invalid or expired login state")]
    InvalidState,

    #[error("provider mismatch")]
    ProviderMismatch,

    #[error("could not confirm account link")]
    LinkMismatch,

    #[error("display name must not be empty")]
    InvalidDisplayName,

    #[error("user not found")]
    UserNotFound,

    #[error("token exchange failed")]
    TokenExchange,

    #[error("id token verification failed")]
    IdTokenVerification,

    #[error("identity resolution failed: {0}")]
    Identity(#[from] identity::Error),

    #[error("authorization backend error: {0}")]
    Authz(#[from] crate::authz::Error),

    #[error("error initializing auth: {source}")]
    Initialization {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

/// The result of a completed OIDC callback.
#[derive(Debug)]
pub enum LoginOutcome {
    /// A normal login (or a confirmed account link).
    /// Contains the session id to set on the response.
    LoggedIn { session_id: String },
    /// The login's verified email collided with a different, existing
    /// account. Nothing was created. `token` identifies the pending link so
    /// the caller can guide the user through confirming it's them, by
    /// logging in again via one of the `target_providers`.
    LinkRequired {
        token: String,
        target_providers: Vec<String>,
        masked_email: String,
    },
}

/// What to show on the "an account already exists" link-confirmation
/// prompt.
#[derive(Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[cfg_attr(test, ts(export, export_to = "PendingLinkInfo.ts"))]
pub struct PendingLinkInfo {
    pub target_providers: Vec<String>,
    pub masked_email: String,
}

/// A user's own profile.
/// Their display name, email, and which providers they can log in with.
#[derive(Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[cfg_attr(test, ts(export, export_to = "Profile.ts"))]
pub struct Profile {
    pub display_name: String,
    pub email: Option<String>,
    pub providers: Vec<String>,
}

#[derive(Clone)]
pub struct AuthManager {
    oidc: Arc<OidcRegistry>,
    pending: PendingLoginStore,
    links: PendingLinkStore,
    sessions: SessionStore,
    identity: IdentityStore,
    external_base_url: String,
}

impl AuthManager {
    pub async fn new(cfg: &Config, db: Db) -> Result<Self, AuthError> {
        let oidc = OidcRegistry::new(&cfg.oidc)
            .await
            .map_err(|e| AuthError::Initialization {
                source: Box::new(e),
            })?;

        let pending = PendingLoginStore::new(Duration::from_mins(1));
        let links = PendingLinkStore::new(Duration::from_mins(5));

        let sessions = SessionStore::new();

        let manager = Self {
            oidc: Arc::new(oidc),
            pending: pending.clone(),
            links: links.clone(),
            sessions,
            identity: IdentityStore::new(db),
            external_base_url: cfg.oidc.external_base_url.clone(),
        };

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_mins(1));
            loop {
                interval.tick().await;
                pending.gc().await;
                links.gc().await;
            }
        });

        Ok(manager)
    }

    pub async fn get_session(&self, session_id: &str) -> Option<Session> {
        self.sessions.get(session_id).await
    }

    pub fn subscribe_sessions(&self) -> tokio::sync::broadcast::Receiver<SessionDelta> {
        self.sessions.subscribe()
    }

    #[tracing::instrument(skip_all)]
    pub async fn logout(&self, session_id: &str) {
        self.sessions.remove(session_id).await;
        tracing::info!("session logged out");
    }

    pub fn provider_ids(&self) -> Vec<String> {
        self.oidc.provider_ids()
    }

    pub async fn list_users(&self) -> Result<Vec<User>, AuthError> {
        Ok(self.identity.list_users().await?)
    }

    pub async fn get_user(&self, id: UserId) -> Result<Option<User>, AuthError> {
        Ok(self.identity.get_user(id).await?)
    }

    pub async fn search_users(&self, query: &str) -> Result<Vec<User>, AuthError> {
        Ok(self.identity.search_by_display_name(query).await?)
    }

    pub async fn profile(&self, user_id: UserId) -> Result<Profile, AuthError> {
        let user = self
            .identity
            .get_user(user_id)
            .await?
            .ok_or(AuthError::UserNotFound)?;
        let providers = self.identity.provider_ids_for_user(user_id).await?;

        Ok(Profile {
            display_name: user.display_name,
            email: user.email,
            providers,
        })
    }

    pub async fn update_display_name(
        &self,
        user_id: UserId,
        display_name: &str,
    ) -> Result<(), AuthError> {
        if display_name.trim().is_empty() {
            return Err(AuthError::InvalidDisplayName);
        }

        let display_name = display_name.trim();
        self.identity
            .update_display_name(user_id, display_name)
            .await?;
        self.sessions
            .update_display_name_for_user(user_id, display_name)
            .await;
        Ok(())
    }

    /// Used to keep the session cached [`GlobalRole`](crate::authz::GlobalRole)
    /// in sync.
    pub async fn update_session_role(&self, user_id: UserId, role: crate::authz::GlobalRole) {
        self.sessions.update_role_for_user(user_id, role).await;
    }

    /// Starts a login oidc round trip.
    ///
    /// When `link_token` is given, this round trip confirms a pending
    /// account link rather than a plain login. The `provider_id` must
    /// therefore be one of the target account's own already-linked providers.
    #[tracing::instrument(skip_all, fields(provider_id = %provider_id))]
    pub async fn start_login(
        &self,
        provider_id: &str,
        link_token: Option<String>,
    ) -> Result<String, AuthError> {
        if let Some(token) = &link_token {
            let link = self.links.get(token).await.ok_or(AuthError::LinkMismatch)?;
            let target_providers = self
                .identity
                .provider_ids_for_user(link.target_user_id)
                .await?;
            if !target_providers.iter().any(|p| p == provider_id) {
                return Err(AuthError::LinkMismatch);
            }
        }

        let provider = self
            .oidc
            .get(provider_id)
            .map_err(|_| AuthError::UnknownProvider)?;

        let redirect_url = format!("{}/auth/callback/{}", self.external_base_url, provider_id);

        let redirect_url =
            RedirectUrl::new(redirect_url).map_err(|_| AuthError::UnknownProvider)?;

        let client = provider
            .client
            .clone()
            .set_redirect_uri(redirect_url.clone());

        let mut req = client.authorize_url(
            AuthenticationFlow::<CoreResponseType>::AuthorizationCode,
            CsrfToken::new_random,
            Nonce::new_random,
        );

        for s in &provider.scopes {
            req = req.add_scope(s.clone());
        }

        let (auth_url, csrf, nonce) = req.url();

        self.pending
            .insert(
                csrf.secret().clone(),
                PendingLogin {
                    provier_id: provider_id.to_string(),
                    nonce,
                    redirect_url,
                    linking: link_token,
                },
            )
            .await;

        tracing::info!("login redirect issued");
        Ok(auth_url.to_string())
    }

    /// Resolves identity, runs `authz`'s new-user bootstrap when this login
    /// created the account, and resolves the [`GlobalRole`](crate::authz::GlobalRole)
    /// to stash on the [`Session`].
    ///
    /// When the login's verified email collides with a *different* existing
    /// account, no session is issued and no account is created, instead
    /// returns [`LoginOutcome::LinkRequired`].
    #[tracing::instrument(skip_all, fields(provider_id = %provider_id))]
    pub async fn finish_login(
        &self,
        provider_id: &str,
        code: String,
        state: String,
        authz: &AuthzManager,
    ) -> Result<LoginOutcome, AuthError> {
        let pl = self
            .pending
            .take(&state)
            .await
            .ok_or(AuthError::InvalidState)?;

        if pl.provier_id != provider_id {
            return Err(AuthError::ProviderMismatch);
        }

        let provider = self
            .oidc
            .get(&pl.provier_id)
            .map_err(|_| AuthError::UnknownProvider)?;

        let client = provider.client.clone().set_redirect_uri(pl.redirect_url);

        let token = client
            .exchange_code(AuthorizationCode::new(code))
            .request_async(async_http_client)
            .await
            .map_err(|e| {
                tracing::warn!(error = %e, "OIDC token exchange failed");
                AuthError::TokenExchange
            })?;

        let id_token = token
            .extra_fields()
            .id_token()
            .ok_or(AuthError::TokenExchange)?;

        let claims = id_token
            .claims(&client.id_token_verifier(), &pl.nonce)
            .map_err(|_| AuthError::IdTokenVerification)?;

        let subject = claims.subject().as_str();
        let display_name = claims
            .preferred_username()
            .map(|s| s.to_string())
            .or(claims.email().map(|s| s.to_string()))
            .or(claims
                .name()
                .and_then(|s| s.get(None))
                .map(|s| s.to_string()))
            .unwrap_or_else(|| subject.to_string());

        // Only care about and store email if the provider says it's verified.
        let email = claims
            .email_verified()
            .unwrap_or(false)
            .then(|| claims.email().map(|s| s.to_string()))
            .flatten();

        let already_known = self
            .identity
            .find_user_id_by_identity(provider_id, subject)
            .await?;

        let confirmed_link = if let Some(link_token) = &pl.linking {
            Some(self.confirm_pending_link(link_token, already_known).await?)
        } else if already_known.is_none()
            && let Some(outcome) = self
                .check_email_collision(provider_id, subject, email.as_deref())
                .await?
        {
            return Ok(outcome);
        } else {
            None
        };

        let (user_id, is_new) = self
            .identity
            .resolve_or_create(provider_id, subject, &display_name, email.as_deref())
            .await?;

        if let Some(link) = confirmed_link {
            self.identity
                .link_identity(user_id, &link.new_provider_id, &link.subject)
                .await?;
            tracing::info!(user_id = %user_id, linked_provider_id = %link.new_provider_id, "account link confirmed");
        }

        let actor = Actor {
            id: user_id,
            display_name: display_name.clone(),
        };
        if is_new {
            authz.on_user_created(&actor).await?;
        }
        let global_role = authz.global_role(&actor).await?;

        let session_id = rand_str(64);
        self.sessions
            .insert(
                session_id.clone(),
                Session::new(user_id, display_name, global_role),
            )
            .await;

        tracing::info!(user_id = %user_id, "login succeeded");
        Ok(LoginOutcome::LoggedIn { session_id })
    }

    /// Validates a login round trip that's confirming an account link.
    ///
    /// Must resolve to an identity that's already attached to *some*
    /// account. Never creates one as a side effect of a link
    /// confirmation, as this is meant to only be offered for linking already
    /// existing accounts.
    async fn confirm_pending_link(
        &self,
        link_token: &str,
        already_known: Option<UserId>,
    ) -> Result<PendingLink, AuthError> {
        let existing_user_id = already_known.ok_or(AuthError::LinkMismatch)?;

        let link = self
            .links
            .take(link_token)
            .await
            .ok_or(AuthError::LinkMismatch)?;
        if link.target_user_id != existing_user_id {
            return Err(AuthError::LinkMismatch);
        }

        Ok(link)
    }

    /// Checks whether a login that would otherwise create a new account has
    /// a verified email already belonging to a different, existing one.
    ///
    /// On a collision, stashes a [`PendingLink`] and returns the
    /// [`LoginOutcome`] for the login. `Ok(None)` means there's no collision
    /// and login should proceed normally.
    async fn check_email_collision(
        &self,
        provider_id: &str,
        subject: &str,
        email: Option<&str>,
    ) -> Result<Option<LoginOutcome>, AuthError> {
        let Some(email) = email else {
            return Ok(None);
        };
        let Some(colliding_user_id) = self.identity.find_user_id_by_email(email).await? else {
            return Ok(None);
        };

        let target_providers = self
            .identity
            .provider_ids_for_user(colliding_user_id)
            .await?;
        let link_token = rand_str(32);
        self.links
            .insert(
                link_token.clone(),
                PendingLink {
                    new_provider_id: provider_id.to_string(),
                    subject: subject.to_string(),
                    email: email.to_string(),
                    target_user_id: colliding_user_id,
                },
            )
            .await;

        tracing::info!(
            target_user_id = %colliding_user_id,
            "login collided with an existing verified email"
        );
        Ok(Some(LoginOutcome::LinkRequired {
            token: link_token,
            target_providers,
            masked_email: mask_email(email),
        }))
    }

    /// What to show on the link-confirmation prompt for a pending link.
    pub async fn pending_link_info(&self, token: &str) -> Result<PendingLinkInfo, AuthError> {
        let link = self.links.get(token).await.ok_or(AuthError::LinkMismatch)?;
        let target_providers = self
            .identity
            .provider_ids_for_user(link.target_user_id)
            .await?;

        Ok(PendingLinkInfo {
            target_providers,
            masked_email: mask_email(&link.email),
        })
    }
}

impl FromRef<AppState> for AuthManager {
    fn from_ref(input: &AppState) -> Self {
        input.auth.clone()
    }
}

/// Masks an email's local part down to its first character, e.g.
/// `ada@example.com` -> `a***@example.com`.
fn mask_email(email: &str) -> String {
    match email.split_once('@') {
        Some((local, domain)) => {
            let first = local.chars().next().map(String::from).unwrap_or_default();
            format!("{first}***@{domain}")
        }
        None => "***".to_string(),
    }
}

fn rand_str(n: usize) -> String {
    rand::rng()
        .sample_iter(&Alphanumeric)
        .take(n)
        .map(char::from)
        .collect()
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/me", get(routes::me))
        .route("/sse", get(routes::sse))
        .route("/providers", get(routes::providers))
        .route("/login", get(routes::login))
        .route("/callback/{provider}", get(routes::oidc_callback))
        .route("/link/{token}", get(routes::link_info))
        .route(
            "/profile",
            get(routes::profile).patch(routes::update_display_name),
        )
}

#[derive(Serialize)]
struct ErrorBody {
    error: &'static str,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> axum::response::Response {
        tracing::warn!(error = %self, "authentication error");

        let (status, message) = match self {
            AuthError::UnknownProvider => (StatusCode::BAD_REQUEST, "unknown_provider"),
            AuthError::InvalidState => (StatusCode::BAD_REQUEST, "invalid_or_expired_state"),
            AuthError::ProviderMismatch => (StatusCode::BAD_REQUEST, "provider_mismatch"),
            AuthError::LinkMismatch => (StatusCode::BAD_REQUEST, "link_mismatch"),
            AuthError::InvalidDisplayName => (StatusCode::BAD_REQUEST, "invalid_display_name"),
            AuthError::UserNotFound => (StatusCode::INTERNAL_SERVER_ERROR, "user_not_found"),
            AuthError::TokenExchange => (StatusCode::UNAUTHORIZED, "token_exchange_failed"),
            AuthError::IdTokenVerification => {
                (StatusCode::UNAUTHORIZED, "id_token_verification_failed")
            }
            AuthError::Identity(_) => (StatusCode::INTERNAL_SERVER_ERROR, "identity_backend_error"),
            AuthError::Authz(_) => (StatusCode::INTERNAL_SERVER_ERROR, "authz_backend_error"),
            AuthError::Initialization { source: _ } => {
                unreachable!("Should never call this from an Axum thing")
            }
        };

        (status, Json(ErrorBody { error: message })).into_response()
    }
}
