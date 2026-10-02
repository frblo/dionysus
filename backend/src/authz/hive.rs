//! [`RoleSource`] backed by Hive's permissions API.
use std::time::Duration;

use async_trait::async_trait;
use reqwest::Url;
use serde::Deserialize;

use crate::auth::{IdentityStore, UserEvents, UserId};
use crate::authz::model::Actor;
use crate::authz::{Error, GlobalRole, RoleSource};
use crate::config::Hive;
use crate::ttl_store::TtlStore;

/// Resolves [`GlobalRole`] from the permissions Hive holds for the user's
/// identity at the configured provider.
///
/// Answers are cached per user. A Hive failure with no fresh cached answer is
/// an error.
///
/// Roles change in Hive without Dionysus being involved, so a refetch that
/// finds a different role than before announces it on `events`.
pub struct HiveRoleSource {
    http: reqwest::Client,
    base_url: Url,
    token: String,
    provider_id: String,
    identities: IdentityStore,
    cache: TtlStore<UserId, GlobalRole>,
    events: UserEvents,
}

impl HiveRoleSource {
    pub fn new(
        config: &Hive,
        identities: IdentityStore,
        events: UserEvents,
    ) -> Result<Self, anyhow::Error> {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()?;

        Ok(Self {
            http,
            base_url: Url::parse(&config.base_url)?,
            token: config.token.clone(),
            provider_id: config.provider_id.clone(),
            identities,
            cache: TtlStore::new(Duration::from_secs(config.cache_ttl_secs)),
            events,
        })
    }

    fn permissions_url(&self, subject: &str) -> Url {
        let mut url = self.base_url.clone();
        if let Ok(mut segments) = url.path_segments_mut() {
            segments
                .pop_if_empty()
                .extend(["api", "v1", "user", subject, "permissions"]);
        }
        url
    }

    async fn fetch(&self, subject: &str) -> Result<Vec<Permission>, reqwest::Error> {
        self.http
            .get(self.permissions_url(subject))
            .bearer_auth(&self.token)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }
}

#[async_trait]
impl RoleSource for HiveRoleSource {
    async fn global_role(&self, actor: &Actor) -> Result<GlobalRole, Error> {
        if let Some(role) = self.cache.get(&actor.id).await {
            return Ok(role);
        }

        let Some(subject) = self
            .identities
            .subject_for(actor.id, &self.provider_id)
            .await
            .map_err(|e| Error::Backend { source: e.into() })?
        else {
            return Ok(GlobalRole::Guest);
        };

        let permissions = self.fetch(&subject).await.map_err(|e| {
            tracing::warn!(error = %e, "hive permissions lookup failed");
            Error::Backend { source: e.into() }
        })?;
        let role = map_permissions(&permissions);

        let previous = self.cache.insert(actor.id, role).await;
        if previous.is_some_and(|previous| previous != role) {
            self.events.role_changed(actor.id, role);
        }

        Ok(role)
    }
}

/// One entry of Hive's `/api/v1/user/{username}/permissions` response.
#[derive(Debug, Deserialize)]
pub struct Permission {
    pub id: String,
    pub scope: Option<String>,
}

/// The highest [`GlobalRole`] granted by Hive's `admin` / `user` permissions.
///
/// Only unscoped (`null`) and wildcard (`*`) assignments count.
pub fn map_permissions(permissions: &[Permission]) -> GlobalRole {
    permissions
        .iter()
        .filter(|p| matches!(p.scope.as_deref(), None | Some("*")))
        .filter_map(|p| match p.id.as_str() {
            "admin" => Some(GlobalRole::Admin),
            "user" => Some(GlobalRole::User),
            _ => None,
        })
        .max()
        .unwrap_or(GlobalRole::Guest)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn perm(id: &str, scope: Option<&str>) -> Permission {
        Permission {
            id: id.to_string(),
            scope: scope.map(str::to_string),
        }
    }

    #[test]
    fn no_permissions_is_guest() {
        assert_eq!(map_permissions(&[]), GlobalRole::Guest);
    }

    #[test]
    fn user_permission_is_user() {
        assert_eq!(map_permissions(&[perm("user", None)]), GlobalRole::User);
    }

    #[test]
    fn admin_permission_is_admin() {
        assert_eq!(map_permissions(&[perm("admin", None)]), GlobalRole::Admin);
    }

    #[test]
    fn highest_role_wins() {
        let perms = [perm("user", None), perm("admin", None)];

        assert_eq!(map_permissions(&perms), GlobalRole::Admin);
    }

    #[test]
    fn wildcard_scope_counts() {
        assert_eq!(
            map_permissions(&[perm("admin", Some("*"))]),
            GlobalRole::Admin
        );
    }

    #[test]
    fn other_scopes_are_ignored() {
        assert_eq!(
            map_permissions(&[perm("admin", Some("somewhere"))]),
            GlobalRole::Guest
        );
    }

    #[test]
    fn unknown_ids_are_ignored() {
        assert_eq!(map_permissions(&[perm("attest", None)]), GlobalRole::Guest);
    }
}
