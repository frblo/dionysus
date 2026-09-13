//! Postgres-backed implementation of the authz ports.
use async_trait::async_trait;
use uuid::Uuid;

use crate::auth::UserId;
use crate::db::Db;

use crate::authz::model::{Actor, GlobalRole, RoomMember, RoomRole};
use crate::authz::{Error, RoleSource, RoleStore, RoomStore};

impl From<sqlx::Error> for Error {
    fn from(value: sqlx::Error) -> Self {
        Error::Backend {
            source: Box::new(value),
        }
    }
}

fn parse_room_role(role: String) -> Result<RoomRole, Error> {
    role.parse()
        .map_err(|e: String| Error::Backend { source: e.into() })
}

#[derive(Clone)]
pub struct PgAuthz {
    db: Db,
}

impl PgAuthz {
    pub fn new(db: Db) -> Self {
        Self { db }
    }
}

#[async_trait]
impl RoleSource for PgAuthz {
    async fn global_role(&self, actor: &Actor) -> Result<GlobalRole, Error> {
        let row = sqlx::query!(
            "SELECT global_role FROM user_roles WHERE user_id = $1",
            actor.id.0
        )
        .fetch_optional(self.db.pool())
        .await?;

        Ok(match row {
            Some(row) => row
                .global_role
                .parse()
                .map_err(|e: String| Error::Backend { source: e.into() })?,
            None => GlobalRole::Guest,
        })
    }
}

#[async_trait]
impl RoleStore for PgAuthz {
    async fn set_global_role(&self, subject: &Actor, role: GlobalRole) -> Result<(), Error> {
        sqlx::query!(
            r#"
            INSERT INTO user_roles (user_id, global_role)
            VALUES ($1, $2)
            ON CONFLICT (user_id) DO UPDATE SET global_role = $2, updated_at = now()
            "#,
            subject.id.0,
            role.as_str(),
        )
        .execute(self.db.pool())
        .await?;

        Ok(())
    }

    async fn on_user_created(&self, actor: &Actor) -> Result<(), Error> {
        let mut tx = self.db.pool().begin().await?;

        // Serializes against every other concurrent first-login so exactly
        // one person can land the "no rows yet" branch below as admin.
        sqlx::query!(r#"SELECT pg_advisory_xact_lock(hashtext('dionysus:first_admin'))"#)
            .execute(&mut *tx)
            .await?;

        sqlx::query!(
            r#"
            INSERT INTO user_roles (user_id, global_role)
            VALUES ($1, CASE WHEN EXISTS (SELECT 1 FROM user_roles) THEN 'guest' ELSE 'admin' END)
            "#,
            actor.id.0,
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }
}

#[async_trait]
impl RoomStore for PgAuthz {
    async fn role_in_room(&self, room_id: Uuid, actor: &Actor) -> Result<Option<RoomRole>, Error> {
        let row = sqlx::query!(
            "SELECT role FROM room_members WHERE room_id = $1 AND user_id = $2",
            room_id,
            actor.id.0,
        )
        .fetch_optional(self.db.pool())
        .await?;

        row.map(|row| parse_room_role(row.role)).transpose()
    }

    async fn member_rooms(&self, actor: &Actor) -> Result<Vec<(Uuid, RoomRole)>, Error> {
        let rows = sqlx::query!(
            "SELECT room_id, role FROM room_members WHERE user_id = $1",
            actor.id.0,
        )
        .fetch_all(self.db.pool())
        .await?;

        rows.into_iter()
            .map(|row| parse_room_role(row.role).map(|role| (row.room_id, role)))
            .collect()
    }

    async fn on_room_created(&self, creator: &Actor, room_id: Uuid) -> Result<(), Error> {
        sqlx::query!(
            r#"
            INSERT INTO room_members (room_id, user_id, role)
            VALUES ($1, $2, 'owner')
            ON CONFLICT (room_id, user_id) DO UPDATE SET role = 'owner'
            "#,
            room_id,
            creator.id.0,
        )
        .execute(self.db.pool())
        .await?;

        Ok(())
    }

    async fn on_room_deleted(&self, room_id: Uuid) -> Result<(), Error> {
        sqlx::query!("DELETE FROM room_members WHERE room_id = $1", room_id)
            .execute(self.db.pool())
            .await?;

        Ok(())
    }

    async fn grant_role(
        &self,
        room_id: Uuid,
        subject: &Actor,
        role: RoomRole,
    ) -> Result<(), Error> {
        sqlx::query!(
            r#"
            INSERT INTO room_members (room_id, user_id, role)
            VALUES ($1, $2, $3)
            ON CONFLICT (room_id, user_id) DO UPDATE SET role = $3
            "#,
            room_id,
            subject.id.0,
            role.as_str(),
        )
        .execute(self.db.pool())
        .await?;

        Ok(())
    }

    async fn revoke_role(&self, room_id: Uuid, subject: &Actor) -> Result<(), Error> {
        sqlx::query!(
            "DELETE FROM room_members WHERE room_id = $1 AND user_id = $2",
            room_id,
            subject.id.0,
        )
        .execute(self.db.pool())
        .await?;

        Ok(())
    }

    async fn list_members(&self, room_id: Uuid) -> Result<Vec<RoomMember>, Error> {
        let rows = sqlx::query!(
            r#"
            SELECT rm.user_id, u.display_name, rm.role
            FROM room_members rm
            JOIN users u ON u.user_id = rm.user_id
            WHERE rm.room_id = $1
            "#,
            room_id,
        )
        .fetch_all(self.db.pool())
        .await?;

        rows.into_iter()
            .map(|row| {
                parse_room_role(row.role).map(|role| RoomMember {
                    user_id: UserId(row.user_id),
                    display_name: row.display_name,
                    role,
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use crate::auth::UserId;

    use super::*;

    async fn make_actor(pool: &sqlx::PgPool, display_name: &str) -> Actor {
        let id = sqlx::query!(
            "INSERT INTO users (display_name) VALUES ($1) RETURNING user_id",
            display_name,
        )
        .fetch_one(pool)
        .await
        .unwrap()
        .user_id;

        Actor {
            id: UserId(id),
            display_name: display_name.to_string(),
        }
    }

    async fn make_room(pool: &sqlx::PgPool, room_name: &str) -> Uuid {
        sqlx::query!(
            "INSERT INTO rooms (room_name) VALUES ($1) RETURNING room_id",
            room_name,
        )
        .fetch_one(pool)
        .await
        .unwrap()
        .room_id
    }

    #[sqlx::test]
    async fn global_role_defaults_to_guest(pool: sqlx::PgPool) {
        let authz = PgAuthz::new(Db::new(pool));
        let actor = Actor {
            id: UserId(Uuid::new_v4()),
            display_name: "Nobody".to_string(),
        };

        assert_eq!(authz.global_role(&actor).await.unwrap(), GlobalRole::Guest);
    }

    #[sqlx::test]
    async fn set_global_role_upserts(pool: sqlx::PgPool) {
        let actor = make_actor(&pool, "Ada").await;
        let authz = PgAuthz::new(Db::new(pool));

        authz
            .set_global_role(&actor, GlobalRole::User)
            .await
            .unwrap();
        assert_eq!(authz.global_role(&actor).await.unwrap(), GlobalRole::User);

        authz
            .set_global_role(&actor, GlobalRole::Admin)
            .await
            .unwrap();
        assert_eq!(authz.global_role(&actor).await.unwrap(), GlobalRole::Admin);
    }

    #[sqlx::test]
    async fn first_user_created_becomes_admin(pool: sqlx::PgPool) {
        let actor = make_actor(&pool, "Ada").await;
        let authz = PgAuthz::new(Db::new(pool));

        authz.on_user_created(&actor).await.unwrap();

        assert_eq!(authz.global_role(&actor).await.unwrap(), GlobalRole::Admin);
    }

    #[sqlx::test]
    async fn second_user_created_becomes_guest(pool: sqlx::PgPool) {
        let first = make_actor(&pool, "Ada").await;
        let second = make_actor(&pool, "Bob").await;
        let authz = PgAuthz::new(Db::new(pool));

        authz.on_user_created(&first).await.unwrap();
        authz.on_user_created(&second).await.unwrap();

        assert_eq!(authz.global_role(&first).await.unwrap(), GlobalRole::Admin);
        assert_eq!(authz.global_role(&second).await.unwrap(), GlobalRole::Guest);
    }

    #[sqlx::test]
    async fn on_room_created_makes_creator_owner(pool: sqlx::PgPool) {
        let creator = make_actor(&pool, "Ada").await;
        let room_id = make_room(&pool, "Act One").await;
        let authz = PgAuthz::new(Db::new(pool));

        authz.on_room_created(&creator, room_id).await.unwrap();

        assert_eq!(
            authz.role_in_room(room_id, &creator).await.unwrap(),
            Some(RoomRole::Owner)
        );
    }

    #[sqlx::test]
    async fn role_in_room_is_none_for_non_member(pool: sqlx::PgPool) {
        let actor = make_actor(&pool, "Ada").await;
        let room_id = make_room(&pool, "Act One").await;
        let authz = PgAuthz::new(Db::new(pool));

        assert_eq!(authz.role_in_room(room_id, &actor).await.unwrap(), None);
    }

    #[sqlx::test]
    async fn member_rooms_lists_only_that_actors_rooms(pool: sqlx::PgPool) {
        let alice = make_actor(&pool, "Alice").await;
        let bob = make_actor(&pool, "Bob").await;
        let alices_room = make_room(&pool, "Alice's Room").await;
        let bobs_room = make_room(&pool, "Bob's Room").await;
        let authz = PgAuthz::new(Db::new(pool));

        authz.on_room_created(&alice, alices_room).await.unwrap();
        authz.on_room_created(&bob, bobs_room).await.unwrap();

        assert_eq!(
            authz.member_rooms(&alice).await.unwrap(),
            vec![(alices_room, RoomRole::Owner)]
        );
    }

    #[sqlx::test]
    async fn grant_role_upserts(pool: sqlx::PgPool) {
        let owner = make_actor(&pool, "Ada").await;
        let member = make_actor(&pool, "Bob").await;
        let room_id = make_room(&pool, "Act One").await;
        let authz = PgAuthz::new(Db::new(pool));
        authz.on_room_created(&owner, room_id).await.unwrap();

        authz
            .grant_role(room_id, &member, RoomRole::Viewer)
            .await
            .unwrap();
        assert_eq!(
            authz.role_in_room(room_id, &member).await.unwrap(),
            Some(RoomRole::Viewer)
        );

        authz
            .grant_role(room_id, &member, RoomRole::Editor)
            .await
            .unwrap();
        assert_eq!(
            authz.role_in_room(room_id, &member).await.unwrap(),
            Some(RoomRole::Editor)
        );
    }

    #[sqlx::test]
    async fn revoke_role_removes_membership(pool: sqlx::PgPool) {
        let owner = make_actor(&pool, "Ada").await;
        let member = make_actor(&pool, "Bob").await;
        let room_id = make_room(&pool, "Act One").await;
        let authz = PgAuthz::new(Db::new(pool));
        authz.on_room_created(&owner, room_id).await.unwrap();
        authz
            .grant_role(room_id, &member, RoomRole::Editor)
            .await
            .unwrap();

        authz.revoke_role(room_id, &member).await.unwrap();

        assert_eq!(authz.role_in_room(room_id, &member).await.unwrap(), None);
    }

    #[sqlx::test]
    async fn revoke_role_on_a_non_member_is_a_no_op(pool: sqlx::PgPool) {
        let member = make_actor(&pool, "Bob").await;
        let room_id = make_room(&pool, "Act One").await;
        let authz = PgAuthz::new(Db::new(pool));

        authz.revoke_role(room_id, &member).await.unwrap();
    }

    #[sqlx::test]
    async fn on_room_deleted_clears_members(pool: sqlx::PgPool) {
        let owner = make_actor(&pool, "Ada").await;
        let room_id = make_room(&pool, "Act One").await;
        let authz = PgAuthz::new(Db::new(pool));
        authz.on_room_created(&owner, room_id).await.unwrap();

        authz.on_room_deleted(room_id).await.unwrap();

        assert_eq!(authz.role_in_room(room_id, &owner).await.unwrap(), None);
    }

    #[sqlx::test]
    async fn list_members_returns_every_member(pool: sqlx::PgPool) {
        let owner = make_actor(&pool, "Ada").await;
        let editor = make_actor(&pool, "Bob").await;
        let room_id = make_room(&pool, "Act One").await;
        let authz = PgAuthz::new(Db::new(pool));
        authz.on_room_created(&owner, room_id).await.unwrap();
        authz
            .grant_role(room_id, &editor, RoomRole::Editor)
            .await
            .unwrap();

        let mut members = authz.list_members(room_id).await.unwrap();
        members.sort_by_key(|m| m.display_name.clone());

        assert_eq!(members.len(), 2);
        assert_eq!(members[0].user_id, owner.id);
        assert_eq!(members[0].display_name, "Ada");
        assert_eq!(members[0].role, RoomRole::Owner);
        assert_eq!(members[1].user_id, editor.id);
        assert_eq!(members[1].display_name, "Bob");
        assert_eq!(members[1].role, RoomRole::Editor);
    }
}
