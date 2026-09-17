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

/// A person's identity.
#[derive(Debug, Clone, Serialize)]
pub struct User {
    pub id: UserId,
    pub display_name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Stable id for one person, independent of which OIDC provider they log in with.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[cfg_attr(test, ts(export, export_to = "UserId.ts"))]
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

    /// Every known user, oldest first.
    pub async fn list_users(&self) -> Result<Vec<User>, Error> {
        let rows = sqlx::query!(
            r#"SELECT user_id AS "user_id!: UserId", display_name, created_at FROM users ORDER BY created_at"#
        )
        .fetch_all(self.db.pool())
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| User {
                id: r.user_id,
                display_name: r.display_name,
                created_at: r.created_at,
            })
            .collect())
    }

    /// Case-insensitive substring match on `display_name`, capped at 20
    /// results.
    ///
    /// Placeholder user lookup for room sharing, used until a real solution,
    /// e.g. email search or invite links, is implemented.
    ///
    /// Not a longterm solution since display names aren't unique.
    ///
    /// Requires 2+ characters in the query so it can't be used as a way to
    /// browse every user's join date.
    pub async fn search_by_display_name(&self, query: &str) -> Result<Vec<User>, Error> {
        if query.trim().chars().count() < 2 {
            return Ok(Vec::new());
        }

        let rows = sqlx::query!(
            r#"SELECT user_id AS "user_id!: UserId", display_name, created_at
               FROM users
               WHERE display_name ILIKE '%' || $1 || '%'
               ORDER BY display_name
               LIMIT 20"#,
            query
        )
        .fetch_all(self.db.pool())
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| User {
                id: r.user_id,
                display_name: r.display_name,
                created_at: r.created_at,
            })
            .collect())
    }

    pub async fn get_user(&self, id: UserId) -> Result<Option<User>, Error> {
        let row = sqlx::query!(
            r#"SELECT user_id AS "user_id!: UserId", display_name, created_at FROM users WHERE user_id = $1"#,
            id.0
        )
        .fetch_optional(self.db.pool())
        .await?;

        Ok(row.map(|r| User {
            id: r.user_id,
            display_name: r.display_name,
            created_at: r.created_at,
        }))
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

    #[sqlx::test]
    async fn list_users_returns_everyone_oldest_first(pool: sqlx::PgPool) {
        let store = IdentityStore::new(Db::new(pool));

        let (first, _) = store
            .resolve_or_create("google", "first", "Ada")
            .await
            .unwrap();
        let (second, _) = store
            .resolve_or_create("google", "second", "Bob")
            .await
            .unwrap();

        let users = store.list_users().await.unwrap();
        assert_eq!(
            users.iter().map(|u| u.id).collect::<Vec<_>>(),
            [first, second]
        );
    }

    #[sqlx::test]
    async fn get_user_returns_none_for_unknown_id(pool: sqlx::PgPool) {
        let store = IdentityStore::new(Db::new(pool));

        let user = store.get_user(UserId(Uuid::new_v4())).await.unwrap();

        assert!(user.is_none());
    }

    #[sqlx::test]
    async fn search_matches_substring_case_insensitively(pool: sqlx::PgPool) {
        let store = IdentityStore::new(Db::new(pool));

        store
            .resolve_or_create("google", "a", "Ada Lovelace")
            .await
            .unwrap();
        store
            .resolve_or_create("google", "b", "Bob")
            .await
            .unwrap();

        let results = store.search_by_display_name("lovelace").await.unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].display_name, "Ada Lovelace");
    }

    #[sqlx::test]
    async fn search_rejects_short_queries(pool: sqlx::PgPool) {
        let store = IdentityStore::new(Db::new(pool));

        store
            .resolve_or_create("google", "a", "A")
            .await
            .unwrap();

        let results = store.search_by_display_name("a").await.unwrap();

        assert!(results.is_empty());
    }
}
