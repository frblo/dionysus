//! A module for interacting with OIDC login and authentication flows.
use std::{collections::HashMap, sync::Arc, time::Duration};

use openidconnect::{
    ClientId, ClientSecret, IssuerUrl, Nonce, PkceCodeVerifier, RedirectUrl, Scope,
    core::{CoreClient, CoreProviderMetadata},
    reqwest::async_http_client,
    url,
};
use thiserror::Error;
use tokio::{sync::RwLock, time::Instant};

use crate::config;

/// Errors that can happen during an OIDC login process
#[derive(Debug, Error)]
pub enum OidcError {
    #[error("invalid issuer url for provider `{provider}`: {source}")]
    InvalidIssuer {
        provider: String,
        #[source]
        source: url::ParseError,
    },

    #[error("OIDC discovery failed for provider `{provider}`: {source}")]
    Discovery {
        provider: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("No OIDC scopes given for provider `{provider}`")]
    NoScopes { provider: String },

    #[error("unknown OIDC provider `{0}`")]
    UnknownProvider(String),
}

/// A PendingLoginStore is used to store information about an ongoing OIDC
/// login process.
///
/// Currently implemented as a in memory map.
#[derive(Debug, Clone)]
pub struct PendingLoginStore {
    ttl: Duration,
    inner: Arc<RwLock<HashMap<String, PendingLogin>>>,
}

/// Contains the neccessary state for an OIDC login to be able to safely
/// finish.
#[derive(Debug)]
pub struct PendingLogin {
    pub provier_id: String,
    pub nonce: Nonce,
    pub pkce_verifier: PkceCodeVerifier,
    pub redirect_url: RedirectUrl,
    pub created_at: Instant,
}

impl PendingLoginStore {
    /// Create a [`PendingLoginStore`] with a set `ttl` [`Duration`].
    pub fn new(ttl: Duration) -> Self {
        Self {
            ttl,
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Insert a [`PendingLogin`] into the store
    pub async fn insert(&self, state: String, pl: PendingLogin) {
        self.inner.write().await.insert(state, pl);
    }

    /// Take a [`PendingLogin`] out of the store
    ///
    /// If the login time has elapsed return [`None`] as if it didn't exist.
    pub async fn take(&self, state: &str) -> Option<PendingLogin> {
        let mut map = self.inner.write().await;
        let pl = map.remove(state)?;
        if pl.created_at.elapsed() > self.ttl {
            return None;
        }
        Some(pl)
    }

    /// Garbage collect the [`PendingLoginStore`] removing any unfinished
    /// [`PendingLogin`]s that have passed the allocated time.
    pub async fn gc(&self) {
        self.inner
            .write()
            .await
            .retain(|_, v| v.created_at.elapsed() < self.ttl);
    }
}

/// A static registry of the [`OidcProvider`]s available.
pub struct OidcRegistry {
    providers: HashMap<String, Arc<OidcProvider>>,
}

/// A client and scopes for a specific OIDC provider.
pub struct OidcProvider {
    pub client: CoreClient,
    pub scopes: Vec<Scope>,
}

impl OidcRegistry {
    /// Create a new registry from the config file.
    ///
    /// Will error when it fails to instantiate all the providers,
    /// the assumption is that this indicates an error in the configuration.
    pub async fn new(cfg: &config::Oidc) -> Result<Self, OidcError> {
        let mut providers = HashMap::new();

        for (id, p) in &cfg.providers {
            let issuer =
                IssuerUrl::new(p.issuer.clone()).map_err(|e| OidcError::InvalidIssuer {
                    provider: id.clone(),
                    source: e,
                })?;

            let meta = CoreProviderMetadata::discover_async(issuer, async_http_client)
                .await
                .map_err(|e| OidcError::Discovery {
                    provider: id.clone(),
                    source: Box::new(e),
                })?;

            let client = CoreClient::from_provider_metadata(
                meta,
                ClientId::new(p.client_id.clone()),
                Some(ClientSecret::new(p.client_secret.clone())),
            );

            if p.scopes.is_empty() {
                return Err(OidcError::NoScopes {
                    provider: id.clone(),
                });
            }

            let scopes = p.scopes.iter().cloned().map(Scope::new).collect();

            providers.insert(id.clone(), Arc::new(OidcProvider { client, scopes }));
        }

        Ok(Self { providers })
    }

    pub fn get(&self, id: &str) -> Result<Arc<OidcProvider>, OidcError> {
        self.providers
            .get(id)
            .cloned()
            .ok_or_else(|| OidcError::UnknownProvider(id.to_string()))
    }
}
