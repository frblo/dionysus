use std::collections::HashSet;

use uuid::Uuid;

use crate::auth::{Session, UserId};

/// The actor who is asking about something.
#[derive(Debug, Clone)]
pub struct Actor {
    pub id: UserId,
    pub display_name: String,
}

impl From<&Session> for Actor {
    fn from(session: &Session) -> Self {
        Self {
            id: session.user_id,
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
#[cfg_attr(test, derive(ts_rs::TS))]
#[cfg_attr(test, ts(export, export_to = "RoomRole.ts"))]
pub enum RoomRole {
    /// An owner of a room, can do destructive actions like rename and delete.
    Owner,
    /// An editor of a room, can edit the file.
    Editor,
    /// A viewer of a room, can only view it's contents.
    Viewer,
}

impl RoomRole {
    pub fn as_str(self) -> &'static str {
        match self {
            RoomRole::Owner => "owner",
            RoomRole::Editor => "editor",
            RoomRole::Viewer => "viewer",
        }
    }
}

impl std::str::FromStr for RoomRole {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "owner" => Ok(RoomRole::Owner),
            "editor" => Ok(RoomRole::Editor),
            "viewer" => Ok(RoomRole::Viewer),
            other => Err(format!("unknown room role: {other}")),
        }
    }
}

/// App-wide role, separate from room membership.
///
/// [`Self::Admin`] >= [`Self::User`] >= [`Self::Guest`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
#[cfg_attr(test, derive(ts_rs::TS))]
#[cfg_attr(test, ts(export, export_to = "GlobalRole.ts"))]
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

    pub fn as_str(self) -> &'static str {
        match self {
            GlobalRole::Guest => "guest",
            GlobalRole::User => "user",
            GlobalRole::Admin => "admin",
        }
    }
}

impl std::str::FromStr for GlobalRole {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "guest" => Ok(GlobalRole::Guest),
            "user" => Ok(GlobalRole::User),
            "admin" => Ok(GlobalRole::Admin),
            other => Err(format!("unknown global role: {other}")),
        }
    }
}

/// A member in a room.
#[derive(Debug, Clone, serde::Serialize)]
pub struct RoomMember {
    pub user_id: UserId,
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
        let id = UserId(Uuid::from_u128(1));
        let session = Session::new(id, "Ada".to_string(), GlobalRole::User);
        let actor = Actor::from(&session);
        assert_eq!(actor.id, id);
        assert_eq!(actor.display_name, "Ada");
    }

    #[test]
    fn room_role_round_trips_through_str() {
        for role in [RoomRole::Owner, RoomRole::Editor, RoomRole::Viewer] {
            assert_eq!(role.as_str().parse::<RoomRole>().unwrap(), role);
        }
        assert!("nonsense".parse::<RoomRole>().is_err());
    }

    #[test]
    fn global_role_round_trips_through_str() {
        for role in [GlobalRole::Guest, GlobalRole::User, GlobalRole::Admin] {
            assert_eq!(role.as_str().parse::<GlobalRole>().unwrap(), role);
        }
        assert!("nonsense".parse::<GlobalRole>().is_err());
    }
}
