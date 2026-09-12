use std::collections::HashSet;

use uuid::Uuid;

use crate::auth::Session;

/// The actor who is asking about something.
#[derive(Debug, Clone)]
pub struct Actor {
    pub external_id: String,
    pub display_name: String,
}

impl From<&Session> for Actor {
    fn from(session: &Session) -> Self {
        Self {
            external_id: session.user_id.clone(),
            display_name: session.display_name.clone(),
        }
    }
}

/// The different actions one can do with a room.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RoomAction {
    View,
    Edit,
    Rename,
    Delete,
    ManageMembers,
}

/// For actions that aren't about any one room.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CollectionAction {
    /// For asking if one is allowed to create rooms.
    CreateRoom,
    /// For asking if one is allowed to view a list of rooms.
    ListRooms,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    Allow,
    Deny,
}

/// A scope of which Rooms something applies to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RoomScope {
    /// Every room, including ones created after this check runs.
    All,
    Only(HashSet<Uuid>),
}

/// A role within one room.
///
/// [`Self::Owner`] >= [`Self::Editor`] >= [`Self::Viewer`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum RoomRole {
    /// An owner of a room, can do destructive actions like rename and delete.
    Owner,
    /// An editor of a room, can edit the file.
    Editor,
    /// A viewer of a room, can only view it's contents.
    Viewer,
}

/// App-wide role, separate from room membership.
///
/// [`Self::Admin`] >= [`Self::User`] >= [`Self::Guest`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum GlobalRole {
    /// The default for an account with no grant. Can view/edit whatever
    /// rooms they're a member of, but can't create a room or hold
    /// [`RoomRole::Owner`] anywhere.
    #[default]
    Guest,
    /// Guest privileges plus creating rooms and managing the ones they own
    /// (rename, delete, membership).
    User,
    /// Every [`RoomRole`] capability, everywhere, plus changing anyone's
    /// roles. Gates admin-only endpoints and is what makes
    /// [`RoomScope::All`] possible.
    Admin,
}

impl GlobalRole {
    pub fn is_admin(self) -> bool {
        matches!(self, GlobalRole::Admin)
    }
}

/// A member in a room.
#[derive(Debug, Clone, serde::Serialize)]
pub struct RoomMember {
    pub external_id: String,
    pub display_name: String,
    pub role: RoomRole,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn global_role_defaults_to_guest() {
        assert_eq!(GlobalRole::default(), GlobalRole::Guest);
    }

    #[test]
    fn global_rol_only_admin_is_admin() {
        assert!(!GlobalRole::Guest.is_admin());
        assert!(!GlobalRole::User.is_admin());
        assert!(GlobalRole::Admin.is_admin());
    }

    #[test]
    fn actor_borrows_identity_from_session() {
        let session = Session::new("google|123".to_string(), "Ada".to_string());
        let actor = Actor::from(&session);
        assert_eq!(actor.external_id, "google|123");
        assert_eq!(actor.display_name, "Ada");
    }
}
