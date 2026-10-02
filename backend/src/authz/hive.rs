use serde::Deserialize;

use crate::authz::GlobalRole;

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
