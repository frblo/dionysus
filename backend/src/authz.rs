mod error;
mod model;
mod pg;
mod policy;

use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

pub use error::Error;
pub use model::{
    Actor, CollectionAction, Decision, GlobalRole, RoomAction, RoomMember, RoomRole, RoomScope,
};
pub use pg::PgAuthz;
use policy::{global_role_allows, may_hold_room_role, room_role_allows};

/// Resolves an actor's global role.
#[async_trait]
pub trait RoleSource: Send + Sync + 'static {
    async fn global_role(&self, actor: &Actor) -> Result<GlobalRole, Error>;
}

/// If the [`RoleSource`] can be updated from `dionysus` it should also
/// implement this trait.
#[async_trait]
pub trait RoleStore: RoleSource {
    async fn set_global_role(&self, subject: &Actor, role: GlobalRole) -> Result<(), Error>;

    /// Called once, right after `actor` is created as a brand-new user.
    /// Lets a store apply its own user-bootstrap rule.
    ///
    /// Should be a no-op for a store with nothing to do.
    async fn on_user_created(&self, actor: &Actor) -> Result<(), Error>;
}

/// Room membership storage
#[async_trait]
pub trait RoomStore: Send + Sync + 'static {
    async fn role_in_room(&self, room_id: Uuid, actor: &Actor) -> Result<Option<RoomRole>, Error>;

    async fn member_rooms(&self, actor: &Actor) -> Result<Vec<(Uuid, RoomRole)>, Error>;

    async fn on_room_created(&self, creator: &Actor, room_id: Uuid) -> Result<(), Error>;

    async fn on_room_deleted(&self, room_id: Uuid) -> Result<(), Error>;

    async fn grant_role(&self, room_id: Uuid, subject: &Actor, role: RoomRole)
    -> Result<(), Error>;

    async fn revoke_role(&self, room_id: Uuid, subject: &Actor) -> Result<(), Error>;

    async fn list_members(&self, room_id: Uuid) -> Result<Vec<RoomMember>, Error>;
}

/// `roles` and `role_store` should refer to the same vendor if both exist,
/// if `role_store` is [`None`] that means global roles are managed in an
/// external handler
#[derive(Clone)]
pub struct AuthzManager {
    roles: Arc<dyn RoleSource>,
    role_store: Option<Arc<dyn RoleStore>>,
    rooms: Arc<dyn RoomStore>,
}

impl AuthzManager {
    pub fn new(
        roles: Arc<dyn RoleSource>,
        role_store: Option<Arc<dyn RoleStore>>,
        rooms: Arc<dyn RoomStore>,
    ) -> Self {
        Self {
            roles,
            role_store,
            rooms,
        }
    }

    pub async fn global_role(&self, actor: &Actor) -> Result<GlobalRole, Error> {
        self.roles.global_role(actor).await
    }

    pub async fn authorize_room(
        &self,
        actor: &Actor,
        action: RoomAction,
        room_id: Uuid,
    ) -> Result<Decision, Error> {
        if self.global_role(actor).await?.is_admin() {
            return Ok(Decision::Allow);
        }
        let role = self.rooms.role_in_room(room_id, actor).await?;
        Ok(match role {
            Some(role) if room_role_allows(role, action) => Decision::Allow,
            _ => Decision::Deny,
        })
    }

    pub async fn authorize_collection(
        &self,
        actor: &Actor,
        action: CollectionAction,
    ) -> Result<Decision, Error> {
        let role = self.global_role(actor).await?;
        Ok(if global_role_allows(role, action) {
            Decision::Allow
        } else {
            Decision::Deny
        })
    }

    /// Returns every room `actor` may perform `action` on.
    pub async fn rooms_matching(
        &self,
        actor: &Actor,
        action: RoomAction,
    ) -> Result<RoomScope, Error> {
        if self.global_role(actor).await?.is_admin() {
            return Ok(RoomScope::All);
        }
        let matching = self
            .rooms
            .member_rooms(actor)
            .await?
            .into_iter()
            .filter(|(_, role)| room_role_allows(*role, action))
            .map(|(room_id, _)| room_id)
            .collect();
        Ok(RoomScope::Only(matching))
    }

    /// `actor`'s role in `room_id`, or [`None`] if they aren't a member.
    ///
    /// Their actual role, might not match [`Self::authorize_room`] for
    /// [`GlobalRole::Admin`].
    pub async fn role_in_room(&self, room_id: Uuid, actor: &Actor) -> Result<Option<RoomRole>, Error> {
        self.rooms.role_in_room(room_id, actor).await
    }

    /// Every room `actor` is a member of, with their role in each.
    pub async fn member_rooms(&self, actor: &Actor) -> Result<Vec<(Uuid, RoomRole)>, Error> {
        self.rooms.member_rooms(actor).await
    }

    /// [`authorize_room`](Self::authorize_room), collapsed to a plain
    /// [`Result`] so a handler can `?` it instead of matching on [`Decision`].
    pub async fn require_room(
        &self,
        actor: &Actor,
        action: RoomAction,
        room_id: Uuid,
    ) -> Result<(), Error> {
        enforce(self.authorize_room(actor, action, room_id).await?)
    }

    /// [`authorize_collection`](Self::authorize_collection), collapsed to a
    /// plain [`Result`] so a handler can `?` it instead of matching on [`Decision`].
    pub async fn require_collection(
        &self,
        actor: &Actor,
        action: CollectionAction,
    ) -> Result<(), Error> {
        enforce(self.authorize_collection(actor, action).await?)
    }

    /// For admin-only endpoints
    ///
    /// Anything but [`GlobalRole::Admin`] gets [`Error::Denied`].
    pub async fn require_admin(&self, actor: &Actor) -> Result<(), Error> {
        match self.global_role(actor).await? {
            GlobalRole::Admin => Ok(()),
            GlobalRole::Guest | GlobalRole::User => Err(Error::Denied),
        }
    }

    pub async fn on_room_created(&self, creator: &Actor, room_id: Uuid) -> Result<(), Error> {
        self.rooms.on_room_created(creator, room_id).await
    }

    pub async fn on_room_deleted(&self, room_id: Uuid) -> Result<(), Error> {
        self.rooms.on_room_deleted(room_id).await
    }

    /// Grant a `role` to `subject`.
    ///
    /// # Errors
    ///
    /// [`Error::Invalid`] when attempting to grant [`RoomRole::Owner`] to
    /// a [`GlobalRole::Guest`].
    pub async fn grant_role(
        &self,
        room_id: Uuid,
        subject: &Actor,
        role: RoomRole,
    ) -> Result<(), Error> {
        if !may_hold_room_role(self.global_role(subject).await?, role) {
            return Err(Error::Invalid("a guest cannot hold room ownership".into()));
        }
        self.rooms.grant_role(room_id, subject, role).await
    }

    pub async fn revoke_role(&self, room_id: Uuid, subject: &Actor) -> Result<(), Error> {
        self.rooms.revoke_role(room_id, subject).await
    }

    pub async fn list_members(&self, room_id: Uuid) -> Result<Vec<RoomMember>, Error> {
        self.rooms.list_members(room_id).await
    }

    /// Sets the [`GlobalRole`] of `subject` to `role`
    ///
    /// # Errors
    ///
    /// [`Error::Unsupported`] when no [`RoleStore`] is configured.
    pub async fn set_global_role(&self, subject: &Actor, role: GlobalRole) -> Result<(), Error> {
        match &self.role_store {
            Some(store) => store.set_global_role(subject, role).await,
            None => Err(Error::Unsupported),
        }
    }

    /// A missing [`RoleStore`] is just treated like there is nothing to do.
    pub async fn on_user_created(&self, actor: &Actor) -> Result<(), Error> {
        match &self.role_store {
            Some(store) => store.on_user_created(actor).await,
            None => Ok(()),
        }
    }
}

/// Lift [`Decision`] into a [`Result`].
fn enforce(decision: Decision) -> Result<(), Error> {
    match decision {
        Decision::Allow => Ok(()),
        Decision::Deny => Err(Error::Denied),
    }
}
