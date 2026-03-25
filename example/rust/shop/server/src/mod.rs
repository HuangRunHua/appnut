//! ShopFlux server module — DSL-driven backend, all in-memory.

#[path = "../dsl/model/mod.rs"]
pub mod model;

#[path = "../dsl/rest/app.rs"]
pub mod rest_app;

pub mod facet_handlers;
pub mod i18n;
pub mod jwt;
pub mod payment;
pub mod store_impls;

use std::sync::Arc;

use axum::Router;
use openerp_store::{HierarchyNode, KvOps, ModuleDef, ResourceDef, admin_kv_router};
use openerp_types::DslModel;

use model::*;

pub fn admin_router(
    kv: Arc<dyn openerp_kv::KVStore>,
    auth: Arc<dyn openerp_core::Authenticator>,
) -> Router {
    let mut router = Router::new();
    router = router.merge(admin_kv_router(
        KvOps::<User>::new(kv.clone()),
        auth.clone(),
        "shop",
        "users",
        "user",
    ));
    router = router.merge(admin_kv_router(
        KvOps::<Shop>::new(kv.clone()),
        auth.clone(),
        "shop",
        "shops",
        "shop_entity",
    ));
    router = router.merge(admin_kv_router(
        KvOps::<Category>::new(kv.clone()),
        auth.clone(),
        "shop",
        "categories",
        "category",
    ));
    router = router.merge(admin_kv_router(
        KvOps::<Product>::new(kv.clone()),
        auth.clone(),
        "shop",
        "products",
        "product",
    ));
    router = router.merge(admin_kv_router(
        KvOps::<CartItem>::new(kv.clone()),
        auth.clone(),
        "shop",
        "cart_items",
        "cart_item",
    ));
    router = router.merge(admin_kv_router(
        KvOps::<Order>::new(kv.clone()),
        auth.clone(),
        "shop",
        "orders",
        "order",
    ));
    router = router.merge(admin_kv_router(
        KvOps::<OrderItem>::new(kv.clone()),
        auth.clone(),
        "shop",
        "order_items",
        "order_item",
    ));
    router = router.merge(admin_kv_router(
        KvOps::<Review>::new(kv.clone()),
        auth.clone(),
        "shop",
        "reviews",
        "review",
    ));
    router = router.merge(admin_kv_router(
        KvOps::<Address>::new(kv.clone()),
        auth,
        "shop",
        "addresses",
        "address",
    ));
    router
}

pub fn schema_def() -> ModuleDef {
    ModuleDef {
        id: "shop",
        label: "Shop",
        icon: "shopping-cart",
        resources: vec![
            ResourceDef::from_ir("shop", User::__dsl_ir()).with_desc("User accounts"),
            ResourceDef::from_ir("shop", Shop::__dsl_ir()).with_desc("Seller shops"),
            ResourceDef::from_ir("shop", Category::__dsl_ir()).with_desc("Product categories"),
            ResourceDef::from_ir("shop", Product::__dsl_ir()).with_desc("Products"),
            ResourceDef::from_ir("shop", CartItem::__dsl_ir()).with_desc("Shopping cart"),
            ResourceDef::from_ir("shop", Order::__dsl_ir()).with_desc("Orders"),
            ResourceDef::from_ir("shop", OrderItem::__dsl_ir()).with_desc("Order line items"),
            ResourceDef::from_ir("shop", Review::__dsl_ir()).with_desc("Product reviews"),
            ResourceDef::from_ir("shop", Address::__dsl_ir()).with_desc("Shipping addresses"),
        ],
        hierarchy: hierarchy(),
        enums: vec![],
    }
}

fn hierarchy() -> Vec<HierarchyNode> {
    vec![
        HierarchyNode {
            resource: "user",
            label: "Users",
            icon: "users",
            description: "User accounts",
            children: vec![HierarchyNode::leaf(
                "address",
                "Addresses",
                "map-pin",
                "Shipping addresses",
            )],
        },
        HierarchyNode {
            resource: "shop_entity",
            label: "Shops",
            icon: "store",
            description: "Seller shops",
            children: vec![],
        },
        HierarchyNode {
            resource: "category",
            label: "Categories",
            icon: "folder",
            description: "Product categories",
            children: vec![HierarchyNode::leaf(
                "product",
                "Products",
                "package",
                "Products",
            )],
        },
        HierarchyNode {
            resource: "order",
            label: "Orders",
            icon: "clipboard-list",
            description: "Customer orders",
            children: vec![
                HierarchyNode::leaf("order_item", "Items", "list", "Order line items"),
                HierarchyNode::leaf("review", "Reviews", "star", "Product reviews"),
            ],
        },
        HierarchyNode::leaf("cart_item", "Cart", "shopping-cart", "Shopping cart items"),
    ]
}
