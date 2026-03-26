//! ShopFlux module — FluxModule + ServerModule implementation.

use std::sync::{Arc, OnceLock};

use flux_ffi::module::{FluxModule, ServerContext, ServerModule};
use openerp_flux::{Flux, I18nStore};

use crate::handlers::ShopBff;
use crate::server;

pub struct ShopModule {
    bff: OnceLock<Arc<ShopBff>>,
}

impl Default for ShopModule {
    fn default() -> Self {
        Self {
            bff: OnceLock::new(),
        }
    }
}

impl ShopModule {
    pub fn new() -> Self {
        Self::default()
    }
}

impl FluxModule for ShopModule {
    fn name(&self) -> &str {
        "shop"
    }

    fn on_server_ready(&self, server_url: &str) {
        self.bff.set(Arc::new(ShopBff::new(server_url))).ok();
    }

    fn register_handlers(&self, flux: &Flux) {
        self.bff
            .get()
            .expect("on_server_ready must be called before register_handlers")
            .register(flux);
    }

    fn register_i18n(&self, store: &I18nStore) {
        crate::handlers::i18n_strings::register_all(store);
    }

    fn schema(&self) -> serde_json::Value {
        openerp_store::build_schema("ShopFlux", vec![server::schema_def()])
    }
}

impl ServerModule for ShopModule {
    fn facet_router(&self, ctx: &ServerContext) -> axum::Router {
        let facet_state = Arc::new(server::facet_handlers::FacetStateInner {
            users: openerp_store::KvOps::new(ctx.kv.clone()),
            shops: openerp_store::KvOps::new(ctx.kv.clone()),
            categories: openerp_store::KvOps::new(ctx.kv.clone()),
            products: openerp_store::KvOps::new(ctx.kv.clone()),
            cart_items: openerp_store::KvOps::new(ctx.kv.clone()),
            orders: openerp_store::KvOps::new(ctx.kv.clone()),
            order_items: openerp_store::KvOps::new(ctx.kv.clone()),
            reviews: openerp_store::KvOps::new(ctx.kv.clone()),
            addresses: openerp_store::KvOps::new(ctx.kv.clone()),
            jwt: server::jwt::JwtService::shop_test(),
            i18n: Box::new(server::i18n::DefaultLocalizer),
            payment: Box::new(server::payment::MockPaymentProvider),
        });
        server::facet_handlers::facet_router(facet_state)
    }

    fn admin_router(&self, ctx: &ServerContext) -> axum::Router {
        let rbac = openerp_core::RbacAuthenticator::new(
            server::jwt::SHOP_TEST_SECRET,
            server::roles::shop_permission_map(),
        );
        let auth = openerp_core::resolve_auth_mode(rbac);
        server::admin_router(ctx.kv.clone(), auth)
    }

    fn seed_data(&self, ctx: &ServerContext) {
        seed_demo_data(&ctx.kv);
    }
}

fn hash_pw(password: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    password.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

fn seed_demo_data(kv: &Arc<dyn openerp_kv::KVStore>) {
    use crate::server::model::*;
    use openerp_store::KvOps;
    use openerp_types::*;

    let users_ops = KvOps::<User>::new(kv.clone());
    let shops_ops = KvOps::<Shop>::new(kv.clone());
    let cats_ops = KvOps::<Category>::new(kv.clone());
    let products_ops = KvOps::<Product>::new(kv.clone());
    let addresses_ops = KvOps::<Address>::new(kv.clone());
    let orders_ops = KvOps::<Order>::new(kv.clone());
    let order_items_ops = KvOps::<OrderItem>::new(kv.clone());
    let reviews_ops = KvOps::<Review>::new(kv.clone());

    // E-06-01: Users
    for &(username, display, role) in &[
        ("alice", "Alice Wang", "buyer"),
        ("bob", "Bob Li", "seller"),
        ("carol", "Carol Zhang", "seller"),
        ("dave", "Dave Chen", "buyer"),
    ] {
        users_ops
            .save_new(User {
                id: Id::default(),
                username: username.into(),
                password_hash: Some(PasswordHash::new(hash_pw("password"))),
                avatar: Some(Avatar::new(format!(
                    "https://api.dicebear.com/7.x/initials/svg?seed={}",
                    username
                ))),
                role: role.into(),
                display_name: Some(display.into()),
                description: Some(format!("@{}", username)),
                metadata: None,
                created_at: DateTime::default(),
                updated_at: DateTime::default(),
            })
            .ok();
    }

    // E-06-02: Shops
    let bob_shop = shops_ops
        .save_new(Shop {
            id: Id::default(),
            owner: Name::new("shop/users/bob"),
            name: "Bob's Electronics".into(),
            shop_description: Some("Quality electronics and gadgets".into()),
            avatar: None,
            rating: 45,
            product_count: 0,
            display_name: Some("Bob's Electronics".into()),
            description: None,
            metadata: None,
            created_at: DateTime::default(),
            updated_at: DateTime::default(),
        })
        .unwrap();
    let bob_shop_id = bob_shop.id.to_string();

    let carol_shop = shops_ops
        .save_new(Shop {
            id: Id::default(),
            owner: Name::new("shop/users/carol"),
            name: "Carol's Fashion".into(),
            shop_description: Some("Trendy fashion and accessories".into()),
            avatar: None,
            rating: 48,
            product_count: 0,
            display_name: Some("Carol's Fashion".into()),
            description: None,
            metadata: None,
            created_at: DateTime::default(),
            updated_at: DateTime::default(),
        })
        .unwrap();
    let carol_shop_id = carol_shop.id.to_string();

    // E-06-03: Categories
    let mut electronics_name = LocalizedText::en("Electronics");
    electronics_name.set("zh-CN", "电子产品");
    let electronics = cats_ops
        .save_new(Category {
            id: Id::default(),
            cat_name: electronics_name,
            parent: None,
            sort_order: 1,
            display_name: None,
            description: None,
            metadata: None,
            created_at: DateTime::default(),
            updated_at: DateTime::default(),
        })
        .unwrap();
    let electronics_id = electronics.id.to_string();

    let mut phones_name = LocalizedText::en("Phones");
    phones_name.set("zh-CN", "手机");
    let phones = cats_ops
        .save_new(Category {
            id: Id::default(),
            cat_name: phones_name,
            parent: Some(Name::new(format!("shop/categories/{}", electronics_id))),
            sort_order: 2,
            display_name: None,
            description: None,
            metadata: None,
            created_at: DateTime::default(),
            updated_at: DateTime::default(),
        })
        .unwrap();
    let phones_id = phones.id.to_string();

    let mut clothing_name = LocalizedText::en("Clothing");
    clothing_name.set("zh-CN", "服装");
    let clothing = cats_ops
        .save_new(Category {
            id: Id::default(),
            cat_name: clothing_name,
            parent: None,
            sort_order: 3,
            display_name: None,
            description: None,
            metadata: None,
            created_at: DateTime::default(),
            updated_at: DateTime::default(),
        })
        .unwrap();
    let clothing_id = clothing.id.to_string();

    let mut accessories_name = LocalizedText::en("Accessories");
    accessories_name.set("zh-CN", "配饰");
    cats_ops
        .save_new(Category {
            id: Id::default(),
            cat_name: accessories_name,
            parent: Some(Name::new(format!("shop/categories/{}", clothing_id))),
            sort_order: 4,
            display_name: None,
            description: None,
            metadata: None,
            created_at: DateTime::default(),
            updated_at: DateTime::default(),
        })
        .ok();

    // E-06-04: Products (Bob's shop — 3 electronics)
    let product_data_bob = [
        (
            "Rust Phone Pro",
            "Latest smartphone with Rust-powered firmware",
            99900u64,
            50u32,
            &phones_id,
        ),
        (
            "USB-C Hub 7-in-1",
            "Premium USB-C hub with HDMI, USB-A, SD card",
            4999,
            200,
            &electronics_id,
        ),
        (
            "Mechanical Keyboard",
            "Cherry MX Blue switches, RGB backlight",
            12999,
            100,
            &electronics_id,
        ),
    ];
    let mut bob_product_ids = Vec::new();
    for &(title, desc, price, stock, cat_id) in &product_data_bob {
        let p = products_ops
            .save_new(Product {
                id: Id::default(),
                shop: Name::new(format!("shop/shops/{}", bob_shop_id)),
                category: Name::new(format!("shop/categories/{}", cat_id)),
                title: title.into(),
                product_description: Some(desc.into()),
                price,
                stock,
                images: format!(
                    "[\"https://picsum.photos/seed/{}/400\"]",
                    title.replace(' ', "_").to_lowercase()
                ),
                rating: 0,
                review_count: 0,
                status: "on_sale".into(),
                display_name: Some(title.into()),
                description: None,
                metadata: None,
                created_at: DateTime::default(),
                updated_at: DateTime::default(),
            })
            .unwrap();
        bob_product_ids.push(p.id.to_string());
        std::thread::sleep(std::time::Duration::from_millis(2));
    }

    // Products (Carol's shop — 3 fashion)
    let product_data_carol = [
        (
            "Silk Scarf",
            "100% mulberry silk, hand-rolled edges",
            8999u64,
            30u32,
            &clothing_id,
        ),
        (
            "Leather Tote Bag",
            "Full-grain leather, handmade",
            25999,
            15,
            &clothing_id,
        ),
        (
            "Cashmere Sweater",
            "Premium cashmere, ribbed collar",
            19999,
            25,
            &clothing_id,
        ),
    ];
    for &(title, desc, price, stock, cat_id) in &product_data_carol {
        products_ops
            .save_new(Product {
                id: Id::default(),
                shop: Name::new(format!("shop/shops/{}", carol_shop_id)),
                category: Name::new(format!("shop/categories/{}", cat_id)),
                title: title.into(),
                product_description: Some(desc.into()),
                price,
                stock,
                images: format!(
                    "[\"https://picsum.photos/seed/{}/400\"]",
                    title.replace(' ', "_").to_lowercase()
                ),
                rating: 0,
                review_count: 0,
                status: "on_sale".into(),
                display_name: Some(title.into()),
                description: None,
                metadata: None,
                created_at: DateTime::default(),
                updated_at: DateTime::default(),
            })
            .ok();
        std::thread::sleep(std::time::Duration::from_millis(2));
    }

    // Update shop product counts
    if let Ok(Some(mut s)) = shops_ops.get(&bob_shop_id) {
        s.product_count = 3;
        let _ = shops_ops.save(s);
    }
    if let Ok(Some(mut s)) = shops_ops.get(&carol_shop_id) {
        s.product_count = 3;
        let _ = shops_ops.save(s);
    }

    // E-06-05: Addresses
    addresses_ops
        .save_new(Address {
            id: Id::default(),
            user: Name::new("shop/users/alice"),
            recipient_name: "Alice Wang".into(),
            phone: "13800138001".into(),
            province: "Beijing".into(),
            city: "Beijing".into(),
            district: "Haidian".into(),
            detail: "Zhongguancun Software Park, Building 8".into(),
            is_default: true,
            display_name: None,
            description: None,
            metadata: None,
            created_at: DateTime::default(),
            updated_at: DateTime::default(),
        })
        .ok();
    addresses_ops
        .save_new(Address {
            id: Id::default(),
            user: Name::new("shop/users/dave"),
            recipient_name: "Dave Chen".into(),
            phone: "13900139001".into(),
            province: "Shanghai".into(),
            city: "Shanghai".into(),
            district: "Pudong".into(),
            detail: "Lujiazui Financial Center, Tower 2".into(),
            is_default: true,
            display_name: None,
            description: None,
            metadata: None,
            created_at: DateTime::default(),
            updated_at: DateTime::default(),
        })
        .ok();

    // E-06-06: One completed order (alice bought Rust Phone Pro from bob)
    let phone_product_id = &bob_product_ids[0];
    let order = orders_ops
        .save_new(Order {
            id: Id::default(),
            buyer: Name::new("shop/users/alice"),
            shop: Name::new(format!("shop/shops/{}", bob_shop_id)),
            status: "completed".into(),
            total_amount: 99900,
            shipping_address: r#"{"name":"Alice Wang","phone":"13800138001","province":"Beijing","city":"Beijing","district":"Haidian","detail":"Zhongguancun Software Park, Building 8"}"#.into(),
            items_snapshot: format!(
                r#"[{{"product_id":"{}","title":"Rust Phone Pro","price":99900,"quantity":1}}]"#,
                phone_product_id
            ),
            paid_at: Some(chrono::Utc::now().to_rfc3339()),
            display_name: None,
            description: None,
            metadata: None,
            created_at: DateTime::default(),
            updated_at: DateTime::default(),
        })
        .unwrap();

    order_items_ops
        .save_new(OrderItem {
            id: Id::default(),
            order: Name::new(format!("shop/orders/{}", order.id)),
            product: Name::new(format!("shop/products/{}", phone_product_id)),
            title_snapshot: "Rust Phone Pro".into(),
            price_snapshot: 99900,
            quantity: 1,
            display_name: None,
            description: None,
            metadata: None,
            created_at: DateTime::default(),
            updated_at: DateTime::default(),
        })
        .ok();

    // E-06-07: One review from alice
    reviews_ops
        .save_new(Review {
            id: Id::default(),
            user: Name::new("shop/users/alice"),
            product: Name::new(format!("shop/products/{}", phone_product_id)),
            order: Name::new(format!("shop/orders/{}", order.id)),
            rating: 5,
            content: "Excellent phone! The Rust firmware makes it incredibly fast and reliable."
                .into(),
            display_name: None,
            description: None,
            metadata: None,
            created_at: DateTime::default(),
            updated_at: DateTime::default(),
        })
        .ok();

    if let Ok(Some(mut p)) = products_ops.get(phone_product_id.as_str()) {
        p.review_count = 1;
        p.rating = 5;
        p.stock = p.stock.saturating_sub(1);
        let _ = products_ops.save(p);
    }
}

pub fn register_shop_module() {
    flux_ffi::flux_register_module(|| Box::new(ShopModule::new()));
}
