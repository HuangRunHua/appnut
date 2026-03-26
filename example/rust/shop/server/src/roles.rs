//! ShopFlux role definitions — admin + seller + buyer.

use openerp_core::rbac::{PermissionMap, RolePermissions};

pub enum ShopRole {
    Admin,
    Seller,
    Buyer,
}

impl RolePermissions for ShopRole {
    fn role_name(&self) -> &str {
        match self {
            Self::Admin => "admin",
            Self::Seller => "seller",
            Self::Buyer => "buyer",
        }
    }

    fn granted_permissions(&self) -> &[&str] {
        match self {
            Self::Admin => &["*:*:*"],
            Self::Seller => &[
                "shop:product:create",
                "shop:product:read",
                "shop:product:update",
                "shop:product:list",
                "shop:shop_entity:create",
                "shop:shop_entity:read",
                "shop:shop_entity:update",
                "shop:shop_entity:list",
                "shop:order:read",
                "shop:order:list",
                "shop:order:update",
                "shop:order_item:read",
                "shop:order_item:list",
                "shop:category:read",
                "shop:category:list",
                "shop:review:read",
                "shop:review:list",
                "shop:user:read",
            ],
            Self::Buyer => &[
                "shop:product:read",
                "shop:product:list",
                "shop:category:read",
                "shop:category:list",
                "shop:cart_item:create",
                "shop:cart_item:read",
                "shop:cart_item:update",
                "shop:cart_item:delete",
                "shop:cart_item:list",
                "shop:order:create",
                "shop:order:read",
                "shop:order:list",
                "shop:order_item:read",
                "shop:order_item:list",
                "shop:address:create",
                "shop:address:read",
                "shop:address:update",
                "shop:address:delete",
                "shop:address:list",
                "shop:review:create",
                "shop:review:read",
                "shop:review:list",
                "shop:shop_entity:read",
                "shop:shop_entity:list",
                "shop:user:read",
            ],
        }
    }
}

pub fn shop_permission_map() -> PermissionMap {
    PermissionMap::from_roles(&[&ShopRole::Admin, &ShopRole::Seller, &ShopRole::Buyer])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admin_has_all_permissions() {
        let map = shop_permission_map();
        assert!(map.is_allowed("admin", "shop:order:delete"));
        assert!(map.is_allowed("admin", "shop:product:create"));
        assert!(map.is_allowed("admin", "shop:user:delete"));
    }

    #[test]
    fn seller_can_create_product() {
        let map = shop_permission_map();
        assert!(map.is_allowed("seller", "shop:product:create"));
    }

    #[test]
    fn seller_cannot_create_cart_item() {
        let map = shop_permission_map();
        assert!(!map.is_allowed("seller", "shop:cart_item:create"));
    }

    #[test]
    fn buyer_can_create_order() {
        let map = shop_permission_map();
        assert!(map.is_allowed("buyer", "shop:order:create"));
    }

    #[test]
    fn buyer_cannot_create_product() {
        let map = shop_permission_map();
        assert!(!map.is_allowed("buyer", "shop:product:create"));
    }

    #[test]
    fn buyer_can_write_review() {
        let map = shop_permission_map();
        assert!(map.is_allowed("buyer", "shop:review:create"));
    }

    #[test]
    fn seller_cannot_write_review() {
        let map = shop_permission_map();
        assert!(!map.is_allowed("seller", "shop:review:create"));
    }
}
