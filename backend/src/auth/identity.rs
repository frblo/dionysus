//! Persistent user identity, separate from any one OIDC provider.
//!
//! Note that currently there is no mechanism for linking different providers
//! to the same user. This is deliberately left until a later moment.
use std::fmt;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::Db;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("identity backend error")]
    Backend {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

impl From<sqlx::Error> for Error {
    fn from(value: sqlx::Error) -> Self {
        Error::Backend {
            source: Box::new(value),
        }
    }
}

/// Stable id for one person, independent of which OIDC provider they log in with.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
pub struct UserId(pub Uuid);

impl fmt::Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

#[derive(Clone)]
pub struct IdentityStore {
    db: Db,
}

impl IdentityStore {
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    /// Resolves `(provider_id, subject)` to a [`UserId`], creating the user
    /// on a first login.
    ///
    /// The returned `bool` is `true` when a new user is created, so a caller
    /// can provision anything that only needs to happen once per person
    /// (e.g. an initial role). Refreshes `display_name` on repeat
    /// logins.
    pub async fn resolve_or_create(
        &self,
        provider_id: &str,
        subject: &str,
        display_name: &str,
    ) -> Result<(UserId, bool), Error> {
        let mut tx = self.db.pool().begin().await?;

        let existing = sqlx::query!(
            r#"SELECT user_id AS "user_id!: UserId" FROM user_identities WHERE provider_id = $1 AND subject = $2"#,
            provider_id,
            subject,
        )
        .fetch_optional(&mut *tx)
        .await?;

        let (user_id, is_new) = if let Some(row) = existing {
            sqlx::query!(
                "UPDATE users SET display_name = $2, updated_at = now() WHERE user_id = $1",
                row.user_id.0,
                display_name,
            )
            .execute(&mut *tx)
            .await?;
            (row.user_id, false)
        } else {
            let row = sqlx::query!(
                r#"INSERT INTO users (display_name) VALUES ($1) RETURNING user_id AS "user_id!: UserId""#,
                display_name,
            )
            .fetch_one(&mut *tx)
            .await?;

            sqlx::query!(
                "INSERT INTO user_identities (provider_id, subject, user_id) VALUES ($1, $2, $3)",
                provider_id,
                subject,
                row.user_id.0,
            )
            .execute(&mut *tx)
            .await?;

            (row.user_id, true)
        };

        tx.commit().await?;
        Ok((user_id, is_new))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[sqlx::test]
    async fn first_login_creates_a_user(pool: sqlx::PgPool) {
        let store = IdentityStore::new(Db::new(pool));

        let (_, is_new) = store
            .resolve_or_create("google", "abc", "Ada")
            .await
            .unwrap();

        assert!(is_new);
    }

    #[sqlx::test]
    async fn repeat_login_reuses_user_id_and_refreshes_name(pool: sqlx::PgPool) {
        let store = IdentityStore::new(Db::new(pool.clone()));

        let (first, first_is_new) = store
            .resolve_or_create("google", "abc", "Ada")
            .await
            .unwrap();
        let (second, second_is_new) = store
            .resolve_or_create("google", "abc", "Ada Lovelace")
            .await
            .unwrap();

        assert!(first_is_new);
        assert!(!second_is_new);
        assert_eq!(first, second);

        let name =
            sqlx::query_scalar!("SELECT display_name FROM users WHERE user_id = $1", first.0)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(name, "Ada Lovelace");
    }

    #[sqlx::test]
    async fn different_identities_get_different_users(pool: sqlx::PgPool) {
        let store = IdentityStore::new(Db::new(pool));

        let (first, _) = store
            .resolve_or_create("google", "first", "Ada")
            .await
            .unwrap();
        let (second, _) = store
            .resolve_or_create("google", "second", "Bob")
            .await
            .unwrap();

        assert_ne!(first, second);
    }
}
