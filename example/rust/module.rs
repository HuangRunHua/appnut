//! TwitterFlux module — reference implementation of the Flux plugin interface.
//!
//! # How to register a BFF module with the Flux framework
//!
//! 1. Implement [`FluxModule`] for core BFF logic (handlers, i18n, schema)
//! 2. Implement [`ServerModule`] for server-side setup (routes, seed data)
//! 3. Call [`register_twitter_module`] before `flux_create()`
//!
//! ```ignore
//! // In your application's initialization code:
//! flux_golden::register_twitter_module();
//! let handle = flux_ffi::flux_create(); // C FFI entry point
//! ```

use std::sync::{Arc, OnceLock};

use flux_ffi::module::{FluxModule, ServerContext, ServerModule};
use openerp_flux::{Flux, I18nStore};

use crate::handlers::TwitterBff;
use crate::server;

/// The TwitterFlux application module.
///
/// Wraps [`TwitterBff`] and provides all the hooks the framework needs
/// to set up the embedded server and register BFF handlers.
pub struct TwitterModule {
    bff: OnceLock<Arc<TwitterBff>>,
}

impl Default for TwitterModule {
    fn default() -> Self {
        Self {
            bff: OnceLock::new(),
        }
    }
}

impl TwitterModule {
    pub fn new() -> Self {
        Self::default()
    }
}

impl FluxModule for TwitterModule {
    fn name(&self) -> &str {
        "twitter"
    }

    fn on_server_ready(&self, server_url: &str) {
        self.bff.set(Arc::new(TwitterBff::new(server_url))).ok();
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
        openerp_store::build_schema("Twitter", vec![server::schema_def()])
    }
}

impl ServerModule for TwitterModule {
    fn facet_router(&self, ctx: &ServerContext) -> axum::Router {
        let facet_state = Arc::new(server::facet_handlers::FacetStateInner {
            users: openerp_store::KvOps::new(ctx.kv.clone()),
            tweets: openerp_store::KvOps::new(ctx.kv.clone()),
            likes: openerp_store::KvOps::new(ctx.kv.clone()),
            follows: openerp_store::KvOps::new(ctx.kv.clone()),
            messages: openerp_store::KvOps::new(ctx.kv.clone()),
            jwt: server::jwt::JwtService::golden_test(),
            i18n: Box::new(server::i18n::DefaultLocalizer),
            blobs: ctx.blobs.clone(),
            blob_base_url: ctx.server_url.clone(),
        });
        server::facet_handlers::facet_router(facet_state)
    }

    fn admin_router(&self, ctx: &ServerContext) -> axum::Router {
        let rbac = openerp_core::RbacAuthenticator::new(
            server::jwt::GOLDEN_TEST_SECRET,
            server::roles::twitter_permission_map(),
        );
        let auth = openerp_core::resolve_auth_mode(rbac);
        server::admin_router(ctx.kv.clone(), auth)
    }

    fn seed_data(&self, ctx: &ServerContext) {
        seed_demo_data(&ctx.kv);
    }
}

// ---------------------------------------------------------------------------
// Seed data (moved from crates/bff/ffi/src/lib.rs)
// ---------------------------------------------------------------------------

fn ffi_hash_pw(password: &str) -> String {
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
    let tweets_ops = KvOps::<Tweet>::new(kv.clone());

    for &(username, display, bio) in &[
        (
            "alice",
            "Alice Wang",
            "Rust developer & open source enthusiast",
        ),
        ("bob", "Bob Li", "Product designer at Haivivi"),
        ("carol", "Carol Zhang", "Full-stack engineer"),
    ] {
        users_ops
            .save_new(User {
                id: Id::default(),
                username: username.into(),
                password_hash: Some(PasswordHash::new(ffi_hash_pw("password"))),
                bio: Some(bio.into()),
                avatar: Some(Avatar::new(format!(
                    "https://api.dicebear.com/7.x/initials/svg?seed={}",
                    username
                ))),
                follower_count: 0,
                following_count: 0,
                tweet_count: 0,
                display_name: Some(display.into()),
                description: None,
                metadata: None,
                created_at: DateTime::default(),
                updated_at: DateTime::default(),
            })
            .ok();
    }

    for &(author, content) in &[
        (
            "alice",
            "Just shipped Flux — a cross-platform state engine in Rust!",
        ),
        ("bob", "Dark mode design system is ready. Ship it!"),
        ("carol", "Hot take: Bazel > Cargo for monorepos."),
    ] {
        tweets_ops
            .save_new(Tweet {
                id: Id::default(),
                author: Name::new(format!("twitter/users/{}", author)),
                content: content.into(),
                image_url: None,
                like_count: 0,
                reply_count: 0,
                reply_to: None,
                display_name: None,
                description: None,
                metadata: None,
                created_at: DateTime::default(),
                updated_at: DateTime::default(),
            })
            .ok();
        if let Ok(Some(mut u)) = users_ops.get(author) {
            u.tweet_count += 1;
            let _ = users_ops.save(u);
        }
        std::thread::sleep(std::time::Duration::from_millis(2));
    }

    let msgs_ops = KvOps::<Message>::new(kv.clone());

    let mut t1 = LocalizedText::new();
    t1.set("en", "Welcome to TwitterFlux!");
    t1.set("zh-CN", "欢迎来到 TwitterFlux！");
    t1.set("ja", "TwitterFlux へようこそ！");
    t1.set("es", "¡Bienvenido a TwitterFlux!");
    let mut b1 = LocalizedText::new();
    b1.set(
        "en",
        "Thanks for joining! Follow some users and post your first tweet.",
    );
    b1.set("zh-CN", "感谢加入！快去关注用户，发你的第一条推文吧！");
    b1.set(
        "ja",
        "ご参加ありがとうございます！ユーザーをフォローして最初のツイートを！",
    );
    b1.set(
        "es",
        "¡Gracias por unirte! Sigue a usuarios y publica tu primer tweet.",
    );
    msgs_ops
        .save_new(Message {
            id: Id::default(),
            kind: "broadcast".into(),
            sender: None,
            recipient: None,
            title: t1,
            body: b1,
            read: false,
            display_name: None,
            description: None,
            metadata: None,
            created_at: DateTime::default(),
            updated_at: DateTime::default(),
        })
        .ok();

    let mut t2 = LocalizedText::new();
    t2.set("en", "New Feature: Multi-language Support");
    t2.set("zh-CN", "新功能：多语言支持");
    t2.set("ja", "新機能：多言語サポート");
    t2.set("es", "Nueva función: Soporte multilingüe");
    let mut b2 = LocalizedText::new();
    b2.set(
        "en",
        "Switch between English, Chinese, Japanese and Spanish in Settings.",
    );
    b2.set("zh-CN", "在设置中切换英文、中文、日文和西班牙文。");
    b2.set(
        "ja",
        "設定から英語・中国語・日本語・スペイン語を切り替えられます。",
    );
    b2.set(
        "es",
        "Cambia entre inglés, chino, japonés y español en Configuración.",
    );
    msgs_ops
        .save_new(Message {
            id: Id::default(),
            kind: "system".into(),
            sender: None,
            recipient: None,
            title: t2,
            body: b2,
            read: false,
            display_name: None,
            description: None,
            metadata: None,
            created_at: DateTime::default(),
            updated_at: DateTime::default(),
        })
        .ok();

    let mut t3 = LocalizedText::en("Your account has been verified");
    t3.set("zh-CN", "你的账号已通过认证");
    t3.set("ja", "アカウントが認証されました");
    t3.set("es", "Tu cuenta ha sido verificada");
    let mut b3 = LocalizedText::en("Congratulations! You now have access to the API dashboard.");
    b3.set("zh-CN", "恭喜！你现在可以访问 API 管理面板了。");
    b3.set(
        "ja",
        "おめでとうございます！APIダッシュボードにアクセスできます。",
    );
    b3.set(
        "es",
        "¡Felicitaciones! Ahora tienes acceso al panel de API.",
    );
    msgs_ops
        .save_new(Message {
            id: Id::default(),
            kind: "personal".into(),
            sender: None,
            recipient: Some(Name::new("twitter/users/alice")),
            title: t3,
            body: b3,
            read: false,
            display_name: None,
            description: None,
            metadata: None,
            created_at: DateTime::default(),
            updated_at: DateTime::default(),
        })
        .ok();
}

/// Register the Twitter module with the Flux FFI framework.
///
/// Call this **once** before `flux_create()`:
/// ```ignore
/// flux_golden::register_twitter_module();
/// // Now safe to call flux_create() from C/Swift/Dart
/// ```
pub fn register_twitter_module() {
    flux_ffi::flux_register_module(|| Box::new(TwitterModule::new()));
}
