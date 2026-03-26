//! ShopFlux server — standalone binary with admin dashboard.
//!
//! Usage: cargo run -p shopd

use std::sync::Arc;

use axum::routing::{get, post};
use axum::{Json, Router};
use tracing::info;

use flux_shop::server::model::*;
use openerp_store::KvOps;
use openerp_types::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let dir = tempfile::tempdir()?;
    let db_path = dir.path().join("shop.redb");
    info!("Database: {}", db_path.display());

    let kv: Arc<dyn openerp_kv::KVStore> = Arc::new(openerp_kv::RedbStore::open(&db_path)?);

    seed_data(&kv);
    info!("Seeded test data");

    let auth: Arc<dyn openerp_core::Authenticator> = Arc::new(openerp_core::AllowAll);
    let schema_json =
        openerp_store::build_schema("ShopFlux", vec![flux_shop::server::schema_def()]);

    let shop_admin = flux_shop::server::admin_router(kv.clone(), auth);

    let jwt = flux_shop::server::jwt::JwtService::shop_test();
    let facet_state = Arc::new(flux_shop::server::facet_handlers::FacetStateInner {
        users: KvOps::new(kv.clone()),
        shops: KvOps::new(kv.clone()),
        categories: KvOps::new(kv.clone()),
        products: KvOps::new(kv.clone()),
        cart_items: KvOps::new(kv.clone()),
        orders: KvOps::new(kv.clone()),
        order_items: KvOps::new(kv.clone()),
        reviews: KvOps::new(kv.clone()),
        addresses: KvOps::new(kv.clone()),
        jwt: jwt.clone(),
        i18n: Box::new(flux_shop::server::i18n::DefaultLocalizer),
        payment: Box::new(flux_shop::server::payment::MockPaymentProvider),
    });
    let facet = flux_shop::server::facet_handlers::facet_router(facet_state);

    let schema = schema_json.clone();
    let app = Router::new()
        .route(
            "/",
            get(|| async { axum::response::Html(openerp_web::login_html()) }),
        )
        .route(
            "/dashboard",
            get(|| async { axum::response::Html(openerp_web::dashboard_html()) }),
        )
        .route(
            "/meta/schema",
            get(move || {
                let s = schema.clone();
                async move { Json(s) }
            }),
        )
        .route(
            "/health",
            get(|| async { Json(serde_json::json!({"status": "ok"})) }),
        )
        .route("/auth/login", post(login_handler))
        .nest("/admin/shop", shop_admin)
        .nest("/app/shop", facet);

    let listen = "0.0.0.0:3001";
    info!("ShopFlux server listening on http://{}", listen);
    info!("Dashboard: http://localhost:3001/dashboard");
    info!("Login: root / any password");

    let listener = tokio::net::TcpListener::bind(listen).await?;
    axum::serve(listener, app).await?;

    drop(dir);
    Ok(())
}

#[derive(serde::Deserialize)]
struct LoginReq {
    username: String,
    #[allow(dead_code)]
    password: String,
}

async fn login_handler(Json(body): Json<LoginReq>) -> Json<serde_json::Value> {
    let header = base64_url_encode(r#"{"alg":"HS256","typ":"JWT"}"#);
    let now = chrono::Utc::now().timestamp();
    let payload_json = serde_json::json!({
        "sub": body.username,
        "name": body.username,
        "roles": ["admin"],
        "iat": now,
        "exp": now + 86400,
    });
    let payload = base64_url_encode(&payload_json.to_string());
    let signature = base64_url_encode("shop-test-signature");
    let token = format!("{}.{}.{}", header, payload, signature);

    Json(serde_json::json!({
        "access_token": token,
        "token_type": "Bearer",
        "expires_in": 86400,
    }))
}

fn base64_url_encode(input: &str) -> String {
    use base64::Engine;
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(input.as_bytes())
}

fn seed_data(kv: &Arc<dyn openerp_kv::KVStore>) {
    let users_ops = KvOps::<User>::new(kv.clone());
    let shops_ops = KvOps::<Shop>::new(kv.clone());
    let cats_ops = KvOps::<Category>::new(kv.clone());
    let products_ops = KvOps::<Product>::new(kv.clone());
    let addresses_ops = KvOps::<Address>::new(kv.clone());

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

    let bob_shop = shops_ops
        .save_new(Shop {
            id: Id::default(),
            owner: Name::new("shop/users/bob"),
            name: "Bob's Electronics".into(),
            shop_description: Some("Quality electronics and gadgets".into()),
            avatar: None,
            rating: 45,
            product_count: 3,
            display_name: Some("Bob's Electronics".into()),
            description: None,
            metadata: None,
            created_at: DateTime::default(),
            updated_at: DateTime::default(),
        })
        .unwrap();

    let carol_shop = shops_ops
        .save_new(Shop {
            id: Id::default(),
            owner: Name::new("shop/users/carol"),
            name: "Carol's Fashion".into(),
            shop_description: Some("Trendy fashion and accessories".into()),
            avatar: None,
            rating: 48,
            product_count: 3,
            display_name: Some("Carol's Fashion".into()),
            description: None,
            metadata: None,
            created_at: DateTime::default(),
            updated_at: DateTime::default(),
        })
        .unwrap();

    let mut e_name = LocalizedText::en("Electronics");
    e_name.set("zh-CN", "电子产品");
    let e = cats_ops
        .save_new(Category {
            id: Id::default(),
            cat_name: e_name,
            parent: None,
            sort_order: 1,
            display_name: None,
            description: None,
            metadata: None,
            created_at: DateTime::default(),
            updated_at: DateTime::default(),
        })
        .unwrap();

    let mut p_name = LocalizedText::en("Phones");
    p_name.set("zh-CN", "手机");
    let p = cats_ops
        .save_new(Category {
            id: Id::default(),
            cat_name: p_name,
            parent: Some(Name::new(format!("shop/categories/{}", e.id))),
            sort_order: 2,
            display_name: None,
            description: None,
            metadata: None,
            created_at: DateTime::default(),
            updated_at: DateTime::default(),
        })
        .unwrap();

    let mut c_name = LocalizedText::en("Clothing");
    c_name.set("zh-CN", "服装");
    let c = cats_ops
        .save_new(Category {
            id: Id::default(),
            cat_name: c_name,
            parent: None,
            sort_order: 3,
            display_name: None,
            description: None,
            metadata: None,
            created_at: DateTime::default(),
            updated_at: DateTime::default(),
        })
        .unwrap();

    let mut a_name = LocalizedText::en("Accessories");
    a_name.set("zh-CN", "配饰");
    cats_ops
        .save_new(Category {
            id: Id::default(),
            cat_name: a_name,
            parent: Some(Name::new(format!("shop/categories/{}", c.id))),
            sort_order: 4,
            display_name: None,
            description: None,
            metadata: None,
            created_at: DateTime::default(),
            updated_at: DateTime::default(),
        })
        .ok();

    let bob_products = [
        (
            "Rust Phone Pro",
            "Latest smartphone with Rust-powered firmware",
            99900u64,
            50u32,
            p.id.to_string(),
        ),
        (
            "USB-C Hub 7-in-1",
            "Premium USB-C hub with HDMI, USB-A, SD card",
            4999,
            200,
            e.id.to_string(),
        ),
        (
            "Mechanical Keyboard",
            "Cherry MX Blue switches, RGB backlight",
            12999,
            100,
            e.id.to_string(),
        ),
    ];
    for &(title, desc, price, stock, ref cat_id) in &bob_products {
        products_ops
            .save_new(Product {
                id: Id::default(),
                shop: Name::new(format!("shop/shops/{}", bob_shop.id)),
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

    let carol_products = [
        (
            "Silk Scarf",
            "100% mulberry silk, hand-rolled edges",
            8999u64,
            30u32,
        ),
        (
            "Leather Tote Bag",
            "Full-grain leather, handmade",
            25999,
            15,
        ),
        (
            "Cashmere Sweater",
            "Premium cashmere, ribbed collar",
            19999,
            25,
        ),
    ];
    for &(title, desc, price, stock) in &carol_products {
        products_ops
            .save_new(Product {
                id: Id::default(),
                shop: Name::new(format!("shop/shops/{}", carol_shop.id)),
                category: Name::new(format!("shop/categories/{}", c.id)),
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

    info!("Seeded: 4 users, 2 shops, 4 categories, 6 products, 2 addresses");
    info!("All users password: password");
}

fn hash_pw(password: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    password.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}
