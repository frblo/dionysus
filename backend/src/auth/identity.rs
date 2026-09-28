//! Persistent user identity, separate from any one OIDC provider.
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
    /// (e.g. an initial role).
    ///
    /// `display_name` is captured once, at account creation, and never
    /// updated on repeat logins.
    ///
    /// `email` is only ever stored when the caller already knows it's verified
    /// (e.g. not `None`). It's captured the first time a verified email is
    /// provided, and never updated on repeat logins.
    pub async fn resolve_or_create(
        &self,
        provider_id: &str,
        subject: &str,
        display_name: &str,
        email: Option<&str>,
    ) -> Result<(UserId, bool), Error> {
        let mut tx = self.db.pool().begin().await?;

        sqlx::query!(
            "SELECT pg_advisory_xact_lock(hashtextextended($1 || ':' || $2, 0))",
            provider_id,
            subject,
        )
        .execute(&mut *tx)
        .await?;

        let existing = sqlx::query!(
            r#"SELECT user_id AS "user_id!: UserId" FROM user_identities WHERE provider_id = $1 AND subject = $2"#,
            provider_id,
            subject,
        )
        .fetch_optional(&mut *tx)
        .await?;

        let (user_id, is_new) = if let Some(row) = existing {
            sqlx::query!(
                r#"
                UPDATE users
                SET
                    last_login_at = now(),
                    email = COALESCE(email, $2),
                    updated_at = CASE
                        WHEN email IS NULL AND $2 IS NOT NULL THEN now()
                        ELSE updated_at
                    END
                WHERE user_id = $1
                "#,
                row.user_id.0,
                email,
            )
            .execute(&mut *tx)
            .await?;
            (row.user_id, false)
        } else {
            let row = sqlx::query!(
                r#"INSERT INTO users (display_name, email) VALUES ($1, $2) RETURNING user_id AS "user_id!: UserId""#,
                display_name,
                email,
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

    /// Find the user, if any, with the given `(provider_id, subject)` identity.
    pub async fn find_user_id_by_identity(
        &self,
        provider_id: &str,
        subject: &str,
    ) -> Result<Option<UserId>, Error> {
        let row = sqlx::query!(
            r#"SELECT user_id AS "user_id!: UserId" FROM user_identities WHERE provider_id = $1 AND subject = $2"#,
            provider_id,
            subject,
        )
        .fetch_optional(self.db.pool())
        .await?;

        Ok(row.map(|r| r.user_id))
    }

    /// Find the user, if any, whose stored `email` matches.
    ///
    /// The caller's own claim must be verified before relying
    /// on this for account-linking decisions.
    pub async fn find_user_id_by_email(&self, email: &str) -> Result<Option<UserId>, Error> {
        let row = sqlx::query!(
            r#"SELECT user_id AS "user_id!: UserId" FROM users WHERE email = $1"#,
            email
        )
        .fetch_optional(self.db.pool())
        .await?;

        Ok(row.map(|r| r.user_id))
    }

    /// Every provider a user already has a linked identity with.
    pub async fn provider_ids_for_user(&self, user_id: UserId) -> Result<Vec<String>, Error> {
        let rows = sqlx::query!(
            "SELECT DISTINCT provider_id FROM user_identities WHERE user_id = $1",
            user_id.0
        )
        .fetch_all(self.db.pool())
        .await?;

        Ok(rows.into_iter().map(|r| r.provider_id).collect())
    }

    /// Attach a new `(provider_id, subject)` identity to an existing user.
    ///
    /// Only meant to be called once ownership of both sides has already been
    /// proven.
    pub async fn link_identity(
        &self,
        user_id: UserId,
        provider_id: &str,
        subject: &str,
    ) -> Result<(), Error> {
        sqlx::query!(
            "INSERT INTO user_identities (provider_id, subject, user_id) VALUES ($1, $2, $3)",
            provider_id,
            subject,
            user_id.0,
        )
        .execute(self.db.pool())
        .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[sqlx::test]
    async fn first_login_creates_a_user(pool: sqlx::PgPool) {
        let store = IdentityStore::new(Db::new(pool));

        let (_, is_new) = store
            .resolve_or_create("google", "abc", "Ada", None)
            .await
            .unwrap();

        assert!(is_new);
    }

    #[sqlx::test]
    async fn repeat_login_reuses_user_id_and_does_not_refresh_name(pool: sqlx::PgPool) {
        let store = IdentityStore::new(Db::new(pool.clone()));

        let (first, first_is_new) = store
            .resolve_or_create("google", "abc", "Ada", None)
            .await
            .unwrap();
        let (second, second_is_new) = store
            .resolve_or_create("google", "abc", "Ada Lovelace", None)
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
        assert_eq!(name, "Ada");
    }

    #[sqlx::test]
    async fn repeat_login_bumps_last_login_at(pool: sqlx::PgPool) {
        let store = IdentityStore::new(Db::new(pool.clone()));

        let (id, _) = store
            .resolve_or_create("google", "abc", "Ada", None)
            .await
            .unwrap();
        let created_login =
            sqlx::query_scalar!("SELECT last_login_at FROM users WHERE user_id = $1", id.0)
                .fetch_one(&pool)
                .await
                .unwrap();

        store
            .resolve_or_create("google", "abc", "Ada", None)
            .await
            .unwrap();
        let second_login =
            sqlx::query_scalar!("SELECT last_login_at FROM users WHERE user_id = $1", id.0)
                .fetch_one(&pool)
                .await
                .unwrap();

        assert!(second_login >= created_login);
    }

    #[sqlx::test]
    async fn email_is_stored_on_creation(pool: sqlx::PgPool) {
        let store = IdentityStore::new(Db::new(pool.clone()));

        let (id, _) = store
            .resolve_or_create("google", "abc", "Ada", Some("ada@example.com"))
            .await
            .unwrap();

        let email = sqlx::query_scalar!("SELECT email FROM users WHERE user_id = $1", id.0)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(email.as_deref(), Some("ada@example.com"));
    }

    #[sqlx::test]
    async fn missing_email_is_backfilled_on_repeat_login(pool: sqlx::PgPool) {
        let store = IdentityStore::new(Db::new(pool.clone()));

        let (id, _) = store
            .resolve_or_create("google", "abc", "Ada", None)
            .await
            .unwrap();
        store
            .resolve_or_create("google", "abc", "Ada", Some("ada@example.com"))
            .await
            .unwrap();

        let email = sqlx::query_scalar!("SELECT email FROM users WHERE user_id = $1", id.0)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(email.as_deref(), Some("ada@example.com"));
    }

    #[sqlx::test]
    async fn set_email_is_not_overwritten_by_a_later_login(pool: sqlx::PgPool) {
        let store = IdentityStore::new(Db::new(pool.clone()));

        let (id, _) = store
            .resolve_or_create("google", "abc", "Ada", Some("ada@example.com"))
            .await
            .unwrap();
        store
            .resolve_or_create("google", "abc", "Ada", Some("ada.lovelace@example.com"))
            .await
            .unwrap();

        let email = sqlx::query_scalar!("SELECT email FROM users WHERE user_id = $1", id.0)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(email.as_deref(), Some("ada@example.com"));
    }

    #[sqlx::test]
    async fn different_identities_get_different_users(pool: sqlx::PgPool) {
        let store = IdentityStore::new(Db::new(pool));

        let (first, _) = store
            .resolve_or_create("google", "first", "Ada", None)
            .await
            .unwrap();
        let (second, _) = store
            .resolve_or_create("google", "second", "Bob", None)
            .await
            .unwrap();

        assert_ne!(first, second);
    }

    #[sqlx::test]
    async fn list_users_returns_everyone_oldest_first(pool: sqlx::PgPool) {
        let store = IdentityStore::new(Db::new(pool));

        let (first, _) = store
            .resolve_or_create("google", "first", "Ada", None)
            .await
            .unwrap();
        let (second, _) = store
            .resolve_or_create("google", "second", "Bob", None)
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
            .resolve_or_create("google", "a", "Ada Lovelace", None)
            .await
            .unwrap();
        store
            .resolve_or_create("google", "b", "Bob", None)
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
            .resolve_or_create("google", "a", "A", None)
            .await
            .unwrap();

        let results = store.search_by_display_name("a").await.unwrap();

        assert!(results.is_empty());
    }

    #[sqlx::test]
    async fn find_user_id_by_identity_finds_a_known_identity(pool: sqlx::PgPool) {
        let store = IdentityStore::new(Db::new(pool));

        let (id, _) = store
            .resolve_or_create("google", "abc", "Ada", None)
            .await
            .unwrap();

        let found = store
            .find_user_id_by_identity("google", "abc")
            .await
            .unwrap();

        assert_eq!(found, Some(id));
    }

    #[sqlx::test]
    async fn find_user_id_by_identity_returns_none_for_unknown_identity(pool: sqlx::PgPool) {
        let store = IdentityStore::new(Db::new(pool));

        let found = store
            .find_user_id_by_identity("google", "abc")
            .await
            .unwrap();

        assert_eq!(found, None);
    }

    #[sqlx::test]
    async fn find_user_id_by_email_finds_a_verified_email(pool: sqlx::PgPool) {
        let store = IdentityStore::new(Db::new(pool));

        let (id, _) = store
            .resolve_or_create("google", "abc", "Ada", Some("ada@example.com"))
            .await
            .unwrap();

        let found = store
            .find_user_id_by_email("ada@example.com")
            .await
            .unwrap();

        assert_eq!(found, Some(id));
    }

    #[sqlx::test]
    async fn find_user_id_by_email_returns_none_for_unknown_email(pool: sqlx::PgPool) {
        let store = IdentityStore::new(Db::new(pool));

        let found = store
            .find_user_id_by_email("nobody@example.com")
            .await
            .unwrap();

        assert_eq!(found, None);
    }

    #[sqlx::test]
    async fn provider_ids_for_user_lists_linked_providers(pool: sqlx::PgPool) {
        let store = IdentityStore::new(Db::new(pool));

        let (id, _) = store
            .resolve_or_create("google", "abc", "Ada", None)
            .await
            .unwrap();
        store.link_identity(id, "github", "xyz").await.unwrap();

        let mut providers = store.provider_ids_for_user(id).await.unwrap();
        providers.sort();

        assert_eq!(providers, vec!["github".to_string(), "google".to_string()]);
    }

    #[sqlx::test]
    async fn link_identity_attaches_to_the_given_user(pool: sqlx::PgPool) {
        let store = IdentityStore::new(Db::new(pool));

        let (id, _) = store
            .resolve_or_create("google", "abc", "Ada", None)
            .await
            .unwrap();
        store.link_identity(id, "github", "xyz").await.unwrap();

        let (resolved, is_new) = store
            .resolve_or_create("github", "xyz", "Ada", None)
            .await
            .unwrap();

        assert_eq!(resolved, id);
        assert!(!is_new);
    }
}
