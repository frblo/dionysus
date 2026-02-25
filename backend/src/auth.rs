mod oidc;
mod routes;
mod session;
mod session_store;

use std::sync::Arc;
use std::time::Duration;

use axum::extract::FromRef;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use openidconnect::core::CoreResponseType;
use openidconnect::reqwest::async_http_client;
use openidconnect::{
    AuthenticationFlow, AuthorizationCode, CsrfToken, Nonce, PkceCodeChallenge, PkceCodeVerifier,
    RedirectUrl,
};
use rand::{Rng, distr::Alphanumeric};
use serde::Serialize;
use thiserror::Error;

use crate::auth::oidc::{OidcRegistry, PendingLogin, PendingLoginStore};
use crate::config::Config;
use crate::state::AppState;

pub use session::AuthSession;
pub use session::Session;
pub use session_store::SessionStore;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("unknown provider")]
    UnknownProvider,

    #[error("invalid or expired login state")]
    InvalidState,

    #[error("provider mismatch")]
    ProviderMismatch,

    #[error("token exchange failed")]
    TokenExchange,

    #[error("id token verification failed")]
    IdTokenVerification,

    #[error("error initilizing auth: {source}")]
    Initilization {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

#[derive(Clone)]
pub struct AuthManager {
    oidc: Arc<OidcRegistry>,
    pending: PendingLoginStore,
    sessions: SessionStore,
    external_base_url: String,
}

impl AuthManager {
    pub async fn new(cfg: &Config) -> Result<Self, AuthError> {
        let oidc = OidcRegistry::new(&cfg.oidc)
            .await
            .map_err(|e| AuthError::Initilization {
                source: Box::new(e),
            })?;

        let pending = PendingLoginStore::new(Duration::from_mins(1));

        let sessions = SessionStore::new();

        Ok(Self {
            oidc: Arc::new(oidc),
            pending,
            sessions,
            external_base_url: cfg.oidc.base_external_id.clone(),
        })
    }

    pub async fn get_session(&self, session_id: &str) -> Option<Session> {
        self.sessions.get(session_id).await
    }

    pub async fn logout(&self, session_id: &str) {
        self.sessions.remove(session_id).await
    }

    pub fn provider_ids(&self) -> Vec<String> {
        self.oidc.provider_ids()
    }

    pub async fn start_login(&self, provider_id: &str) -> Result<String, AuthError> {
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

        let (challenge, verifier) = PkceCodeChallenge::new_random_sha256();

        let mut req = client.authorize_url(
            AuthenticationFlow::<CoreResponseType>::AuthorizationCode,
            CsrfToken::new_random,
            Nonce::new_random,
        );

        for s in &provider.scopes {
            req = req.add_scope(s.clone());
        }

        let (auth_url, csrf, nonce) = req.set_pkce_challenge(challenge).url();

        self.pending
            .insert(
                csrf.secret().to_string(),
                PendingLogin {
                    provier_id: provider_id.to_string(),
                    nonce,
                    pkce_verifier: verifier,
                    redirect_url,
                    created_at: tokio::time::Instant::now(),
                },
            )
            .await;

        println!("{auth_url}");
        Ok(auth_url.to_string())
    }

    pub async fn finish_login(
        &self,
        provider_id: &str,
        code: String,
        state: String,
    ) -> Result<String, AuthError> {
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
            .set_pkce_verifier(pl.pkce_verifier)
            .request_async(async_http_client)
            .await
            .map_err(|_| AuthError::TokenExchange)?;

        let id_token = token
            .extra_fields()
            .id_token()
            .ok_or(AuthError::TokenExchange)?;

        let claims = id_token
            .claims(&client.id_token_verifier(), &pl.nonce)
            .map_err(|_| AuthError::IdTokenVerification)?;

        let user_id = format!("{}|{}", claims.issuer().as_str(), claims.subject().as_str());

        let session_id = rand_str(64);
        self.sessions
            .insert(session_id.clone(), Session::new(user_id))
            .await;

        Ok(session_id)
    }
}

impl FromRef<AppState> for AuthManager {
    fn from_ref(input: &AppState) -> Self {
        input.auth.clone()
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
        .route("/providers", get(routes::providers))
        .route("/login", get(routes::login))
        .route("/callback/{provider}", get(routes::oidc_callback))
}

#[derive(Serialize)]
struct ErrorBody {
    error: &'static str,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> axum::response::Response {
        tracing::warn!(error = ?self, "authentication error");

        let (status, message) = match self {
            AuthError::UnknownProvider => (StatusCode::BAD_REQUEST, "unknown_provider"),
            AuthError::InvalidState => (StatusCode::BAD_REQUEST, "invalid_or_expired_state"),
            AuthError::ProviderMismatch => (StatusCode::BAD_REQUEST, "provider_mismatch"),
            AuthError::TokenExchange => (StatusCode::UNAUTHORIZED, "token_exchange_failed"),
            AuthError::IdTokenVerification => {
                (StatusCode::UNAUTHORIZED, "id_token_verification_failed")
            }
            AuthError::Initilization { source: _ } => {
                unreachable!("Should never call this from an Axum thing")
            }
        };

        (status, Json(ErrorBody { error: message })).into_response()
    }
}
