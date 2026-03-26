//! TwitterFlux role definitions — admin + user.

use openerp_core::rbac::{PermissionMap, RolePermissions};

pub enum TwitterRole {
    Admin,
    User,
}

impl RolePermissions for TwitterRole {
    fn role_name(&self) -> &str {
        match self {
            Self::Admin => "admin",
            Self::User => "user",
        }
    }

    fn granted_permissions(&self) -> &[&str] {
        match self {
            Self::Admin => &["*:*:*"],
            Self::User => &[
                "twitter:tweet:create",
                "twitter:tweet:read",
                "twitter:tweet:list",
                "twitter:like:create",
                "twitter:like:delete",
                "twitter:follow:create",
                "twitter:follow:delete",
                "twitter:user:read",
                "twitter:message:read",
                "twitter:message:update",
            ],
        }
    }
}

pub fn twitter_permission_map() -> PermissionMap {
    PermissionMap::from_roles(&[&TwitterRole::Admin, &TwitterRole::User])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admin_has_all_permissions() {
        let map = twitter_permission_map();
        assert!(map.is_allowed("admin", "twitter:user:delete"));
        assert!(map.is_allowed("admin", "twitter:tweet:create"));
        assert!(map.is_allowed("admin", "twitter:message:read"));
    }

    #[test]
    fn user_can_create_tweet() {
        let map = twitter_permission_map();
        assert!(map.is_allowed("user", "twitter:tweet:create"));
    }

    #[test]
    fn user_cannot_delete_tweet() {
        let map = twitter_permission_map();
        assert!(!map.is_allowed("user", "twitter:tweet:delete"));
    }

    #[test]
    fn user_cannot_delete_other_users() {
        let map = twitter_permission_map();
        assert!(!map.is_allowed("user", "twitter:user:delete"));
        assert!(!map.is_allowed("user", "twitter:user:create"));
    }
}
