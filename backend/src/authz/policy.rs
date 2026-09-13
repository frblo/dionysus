use crate::authz::model::{CollectionAction, GlobalRole, RoomAction, RoomRole};

pub fn room_role_allows(role: RoomRole, action: RoomAction) -> bool {
    use RoomAction::*;
    use RoomRole::*;

    match action {
        View => matches!(role, Viewer | Editor | Owner),
        Edit => matches!(role, Editor | Owner),
        Rename | Delete | ManageMembers => matches!(role, Owner),
    }
}

pub fn global_role_allows(role: GlobalRole, action: CollectionAction) -> bool {
    use CollectionAction::*;
    use GlobalRole::*;

    match action {
        CreateRoom => matches!(role, User | Admin),
        ListRooms => true,
    }
}

/// A [`GlobalRole::Guest`] can't be [`RoomRole::Owner`] of any
/// room.
pub fn may_hold_room_role(global: GlobalRole, room: RoomRole) -> bool {
    !matches!((global, room), (GlobalRole::Guest, RoomRole::Owner))
}

#[cfg(test)]
mod tests {
    use super::*;

    const ROOM_ACTIONS: [RoomAction; 5] = [
        RoomAction::View,
        RoomAction::Edit,
        RoomAction::Rename,
        RoomAction::Delete,
        RoomAction::ManageMembers,
    ];

    #[test]
    fn viewer_can_only_view() {
        assert!(room_role_allows(RoomRole::Viewer, RoomAction::View));
        assert!(!room_role_allows(RoomRole::Viewer, RoomAction::Edit));
        assert!(!room_role_allows(RoomRole::Viewer, RoomAction::Rename));
        assert!(!room_role_allows(RoomRole::Viewer, RoomAction::Delete));
        assert!(!room_role_allows(
            RoomRole::Viewer,
            RoomAction::ManageMembers
        ));
    }

    #[test]
    fn editor_can_view_and_edit_only() {
        assert!(room_role_allows(RoomRole::Editor, RoomAction::View));
        assert!(room_role_allows(RoomRole::Editor, RoomAction::Edit));
        assert!(!room_role_allows(RoomRole::Editor, RoomAction::Rename));
        assert!(!room_role_allows(RoomRole::Editor, RoomAction::Delete));
        assert!(!room_role_allows(
            RoomRole::Editor,
            RoomAction::ManageMembers
        ));
    }

    #[test]
    fn owner_can_do_every_room_action() {
        for action in ROOM_ACTIONS {
            assert!(
                room_role_allows(RoomRole::Owner, action),
                "owner denied {action:?}"
            );
        }
    }

    #[test]
    fn room_roles_are_additive() {
        for action in ROOM_ACTIONS {
            if room_role_allows(RoomRole::Viewer, action) {
                assert!(
                    room_role_allows(RoomRole::Editor, action),
                    "editor < viewer for {action:?}"
                );
            }
            if room_role_allows(RoomRole::Editor, action) {
                assert!(
                    room_role_allows(RoomRole::Owner, action),
                    "owner < editor for {action:?}"
                );
            }
        }
    }

    #[test]
    fn only_guest_creating_a_room_is_denied() {
        assert!(!global_role_allows(
            GlobalRole::Guest,
            CollectionAction::CreateRoom
        ));
        assert!(global_role_allows(
            GlobalRole::User,
            CollectionAction::CreateRoom
        ));
        assert!(global_role_allows(
            GlobalRole::Admin,
            CollectionAction::CreateRoom
        ));
    }

    #[test]
    fn everyone_may_list_rooms() {
        for role in [GlobalRole::Guest, GlobalRole::User, GlobalRole::Admin] {
            assert!(global_role_allows(role, CollectionAction::ListRooms));
        }
    }

    #[test]
    fn only_guest_owner_pairing_is_forbidden() {
        assert!(!may_hold_room_role(GlobalRole::Guest, RoomRole::Owner));
        assert!(may_hold_room_role(GlobalRole::Guest, RoomRole::Editor));
        assert!(may_hold_room_role(GlobalRole::Guest, RoomRole::Viewer));
        for room in [RoomRole::Owner, RoomRole::Editor, RoomRole::Viewer] {
            assert!(may_hold_room_role(GlobalRole::User, room));
            assert!(may_hold_room_role(GlobalRole::Admin, room));
        }
    }
}
