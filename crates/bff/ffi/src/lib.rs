//! Flux FFI — C-compatible API for cross-platform bindings.
//!
//! Architecture:
//! 1. Application calls [`flux_register_module`] to provide its [`ServerModule`]
//! 2. `flux_create()` starts an embedded HTTP server with the module's routes
//! 3. All state flows through JSON bytes — no type-specific (de)serialization
//! 4. iOS/Android/Desktop all share the same backend data

#![allow(clippy::not_unsafe_ptr_arg_deref)]

pub mod module;

use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::{CStr, CString, c_char};
use std::panic::AssertUnwindSafe;
use std::sync::{Mutex, OnceLock};

use openerp_flux::{Flux, SubscriptionId};

pub use module::{FluxModule, ServerContext, ServerModule};

/// C function pointer type for state change notifications.
pub type FluxChangeCallback = unsafe extern "C" fn(path: *const c_char, json: *const c_char);

/// Opaque handle to a Flux instance + embedded server.
pub struct FluxHandle {
    flux: Flux,
    _module: Box<dyn ServerModule>,
    i18n: openerp_flux::I18nStore,
    rt: tokio::runtime::Runtime,
    server_url: CString,
    subscriptions: Mutex<HashMap<u64, String>>,
}

/// Byte buffer returned from FFI calls.
#[repr(C)]
pub struct FluxBytes {
    pub ptr: *const u8,
    pub len: usize,
}

// ---------------------------------------------------------------------------
// Module registration
// ---------------------------------------------------------------------------

type ModuleFactory = Box<dyn Fn() -> Box<dyn ServerModule> + Send + Sync>;
static MODULE_FACTORY: OnceLock<ModuleFactory> = OnceLock::new();

/// Register an application module factory.
///
/// Must be called **once** before `flux_create`. The factory is invoked
/// each time `flux_create` is called to produce a fresh module instance.
///
/// ```ignore
/// flux_ffi::flux_register_module(|| Box::new(MyAppModule::new()));
/// ```
pub fn flux_register_module<F>(factory: F)
where
    F: Fn() -> Box<dyn ServerModule> + Send + Sync + 'static,
{
    MODULE_FACTORY.set(Box::new(factory)).ok();
}

// ---------------------------------------------------------------------------
// Error handling
// ---------------------------------------------------------------------------

thread_local! {
    static LAST_ERROR: RefCell<Option<CString>> = const { RefCell::new(None) };
}

fn set_last_error(msg: &str) {
    LAST_ERROR.with(|cell| {
        *cell.borrow_mut() = CString::new(msg).ok();
    });
}

/// Return the last error message as a null-terminated C string.
/// Returns null if no error has been recorded.
/// The pointer is valid until the next FFI call on the same thread.
#[unsafe(no_mangle)]
pub extern "C" fn flux_last_error() -> *const c_char {
    LAST_ERROR.with(|cell| {
        cell.borrow()
            .as_ref()
            .map_or(std::ptr::null(), |s| s.as_ptr())
    })
}

// ============================================================================
// Lifecycle
// ============================================================================

/// Create a new Flux instance.
///
/// A [`ServerModule`] must have been registered via [`flux_register_module`]
/// before calling this function.
///
/// Starts an embedded HTTP server with admin dashboard + REST API.
/// Returns an opaque handle. Must be freed with `flux_free`.
#[unsafe(no_mangle)]
pub extern "C" fn flux_create() -> *mut FluxHandle {
    let factory = match MODULE_FACTORY.get() {
        Some(f) => f,
        None => {
            set_last_error("flux_create: no module registered — call flux_register_module first");
            return std::ptr::null_mut();
        }
    };

    let module = factory();

    let rt = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(2)
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            set_last_error(&format!("failed to create tokio runtime: {e}"));
            return std::ptr::null_mut();
        }
    };

    let server_url = match rt.block_on(start_embedded_server(module.as_ref())) {
        Ok(url) => url,
        Err(e) => {
            set_last_error(&e);
            return std::ptr::null_mut();
        }
    };

    module.on_server_ready(&server_url);

    let flux = Flux::new();
    module.register_handlers(&flux);

    let i18n = openerp_flux::I18nStore::new("en");
    module.register_i18n(&i18n);

    let c_url = match CString::new(server_url) {
        Ok(s) => s,
        Err(e) => {
            set_last_error(&format!("server_url contains interior null byte: {e}"));
            return std::ptr::null_mut();
        }
    };

    let handle = Box::new(FluxHandle {
        flux,
        _module: module,
        i18n,
        rt,
        server_url: c_url,
        subscriptions: Mutex::new(HashMap::new()),
    });

    Box::into_raw(handle)
}

/// Free a Flux handle.
#[unsafe(no_mangle)]
pub extern "C" fn flux_free(handle: *mut FluxHandle) {
    if !handle.is_null() {
        unsafe {
            drop(Box::from_raw(handle));
        }
    }
}

/// Get the server URL (e.g. "http://192.168.1.100:3000").
/// Returns a null-terminated C string. Do NOT free it.
#[unsafe(no_mangle)]
pub extern "C" fn flux_server_url(handle: *const FluxHandle) -> *const c_char {
    if handle.is_null() {
        set_last_error("flux_server_url: null handle");
        return std::ptr::null();
    }
    let handle = unsafe { &*handle };
    handle.server_url.as_ptr()
}

// ============================================================================
// State — read
// ============================================================================

/// Read state at the given path as JSON bytes.
///
/// Returns the raw JSON bytes stored by the BFF handler.
/// Returns a null/zero `FluxBytes` if the path has no state.
#[unsafe(no_mangle)]
pub extern "C" fn flux_get(handle: *const FluxHandle, path: *const c_char) -> FluxBytes {
    if handle.is_null() {
        set_last_error("flux_get: null handle");
        return FluxBytes {
            ptr: std::ptr::null(),
            len: 0,
        };
    }
    let handle = unsafe { &*handle };
    let path = unsafe { CStr::from_ptr(path) }.to_str().unwrap_or("");

    match handle.flux.get(path) {
        Some(bytes) => bytes_to_ffi(bytes),
        None => FluxBytes {
            ptr: std::ptr::null(),
            len: 0,
        },
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn flux_bytes_free(bytes: FluxBytes) {
    if !bytes.ptr.is_null() && bytes.len > 0 {
        unsafe {
            let _ = Vec::from_raw_parts(bytes.ptr as *mut u8, bytes.len, bytes.len);
        }
    }
}

// ============================================================================
// I18n — synchronous translation
// ============================================================================

/// Get a translated string. Synchronous.
/// `url` is "path" or "path?key=value&key2=value2".
/// Returns a C string. Caller must free with `flux_bytes_free`.
#[unsafe(no_mangle)]
pub extern "C" fn flux_i18n_get(handle: *const FluxHandle, url: *const c_char) -> FluxBytes {
    if handle.is_null() {
        set_last_error("flux_i18n_get: null handle");
        return FluxBytes {
            ptr: std::ptr::null(),
            len: 0,
        };
    }
    let handle = unsafe { &*handle };
    let url = unsafe { CStr::from_ptr(url) }.to_str().unwrap_or("");
    let text = handle.i18n.get(url);
    bytes_to_ffi(text.into_bytes())
}

/// Set the i18n locale (e.g. "zh-CN", "en", "ja", "es").
/// Updates UI strings (I18nStore) AND emits `app/set-locale` for BFF handlers.
#[unsafe(no_mangle)]
pub extern "C" fn flux_i18n_set_locale(handle: *const FluxHandle, locale: *const c_char) {
    if handle.is_null() {
        set_last_error("flux_i18n_set_locale: null handle");
        return;
    }
    let handle = unsafe { &*handle };
    let locale_str = unsafe { CStr::from_ptr(locale) }.to_str().unwrap_or("en");
    handle.i18n.set_locale(locale_str);
    let payload = serde_json::json!({"locale": locale_str}).to_string();
    handle.rt.block_on(async {
        handle.flux.emit("app/set-locale", payload.as_bytes()).await;
    });
}

// ============================================================================
// Requests — emit
// ============================================================================

/// Emit a request to the Flux engine.
///
/// `payload_json` is a JSON-encoded C string (or null for empty payload).
/// The JSON bytes are passed directly to the registered BFF handler —
/// no framework-level deserialization occurs here.
#[unsafe(no_mangle)]
pub extern "C" fn flux_emit(
    handle: *mut FluxHandle,
    path: *const c_char,
    payload_json: *const c_char,
) {
    if handle.is_null() {
        set_last_error("flux_emit: null handle");
        return;
    }
    let handle = unsafe { &*handle };
    let path_str = unsafe { CStr::from_ptr(path) }.to_str().unwrap_or("");
    let json_bytes: &[u8] = if payload_json.is_null() {
        &[]
    } else {
        unsafe { CStr::from_ptr(payload_json) }.to_bytes()
    };

    handle.rt.block_on(async {
        handle.flux.emit(path_str, json_bytes).await;
    });
}

// ============================================================================
// Subscriptions
// ============================================================================

/// Subscribe to state changes matching the given pattern.
///
/// The callback is invoked with the changed path and its JSON value.
/// Returns a non-zero subscription ID on success, or 0 on error.
#[unsafe(no_mangle)]
pub extern "C" fn flux_subscribe(
    handle: *mut FluxHandle,
    pattern: *const c_char,
    callback: FluxChangeCallback,
) -> u64 {
    if handle.is_null() {
        set_last_error("flux_subscribe: null handle");
        return 0;
    }
    let handle = unsafe { &*handle };
    let pattern_str = match unsafe { CStr::from_ptr(pattern) }.to_str() {
        Ok(s) => s,
        Err(e) => {
            set_last_error(&format!("flux_subscribe: invalid pattern: {e}"));
            return 0;
        }
    };

    let id = handle
        .flux
        .subscribe(pattern_str, move |path, json_bytes: &[u8]| {
            let _ = std::panic::catch_unwind(AssertUnwindSafe(|| {
                let json = String::from_utf8_lossy(json_bytes);
                if let (Ok(c_path), Ok(c_json)) = (CString::new(path), CString::new(json.as_ref()))
                {
                    unsafe { callback(c_path.as_ptr(), c_json.as_ptr()) };
                }
            }));
        });

    handle
        .subscriptions
        .lock()
        .unwrap()
        .insert(id.0, pattern_str.to_string());

    id.0
}

/// Remove a subscription previously registered with `flux_subscribe`.
#[unsafe(no_mangle)]
pub extern "C" fn flux_unsubscribe(handle: *mut FluxHandle, subscription_id: u64) {
    if handle.is_null() {
        set_last_error("flux_unsubscribe: null handle");
        return;
    }
    let handle = unsafe { &*handle };
    if let Some(pattern) = handle
        .subscriptions
        .lock()
        .unwrap()
        .remove(&subscription_id)
    {
        handle
            .flux
            .unsubscribe(&pattern, SubscriptionId(subscription_id));
    }
}

// ============================================================================
// Server startup
// ============================================================================

async fn start_embedded_server(module: &dyn ServerModule) -> Result<String, String> {
    use std::sync::Arc;

    let dir = tempfile::tempdir().map_err(|e| format!("failed to create temp dir: {e}"))?;
    let dir_path = dir.path().to_path_buf();
    let kv: Arc<dyn openerp_kv::KVStore> = Arc::new(
        openerp_kv::RedbStore::open(&dir_path.join("flux.redb"))
            .map_err(|e| format!("failed to open redb: {e}"))?,
    );
    std::mem::forget(dir);

    let auth: Arc<dyn openerp_core::Authenticator> = Arc::new(openerp_core::AllowAll);

    let lan_ip = get_lan_ip().unwrap_or_else(|| "127.0.0.1".to_string());
    let listener = tokio::net::TcpListener::bind("0.0.0.0:0")
        .await
        .map_err(|e| format!("failed to bind: {e}"))?;
    let port = listener
        .local_addr()
        .map_err(|e| format!("failed to get local addr: {e}"))?
        .port();
    let server_url = format!("http://{}:{}", lan_ip, port);

    let blob_dir = dir_path.join("blobs");
    std::fs::create_dir_all(&blob_dir).ok();
    let blobs: Arc<dyn openerp_blob::BlobStore> = Arc::new(
        openerp_blob::FileStore::open(&blob_dir)
            .map_err(|e| format!("failed to open blob store: {e}"))?,
    );

    let ctx = ServerContext {
        kv,
        blobs,
        auth,
        server_url: server_url.clone(),
    };

    module.seed_data(&ctx);

    let module_name = module.name();
    let facet_router = module.facet_router(&ctx);
    let admin_router = module.admin_router(&ctx);
    let schema_json = module.schema();

    tracing::info!("Embedded server: {}", server_url);
    tracing::info!("Dashboard: {}/dashboard", server_url);

    let schema = schema_json.clone();

    let login_handler = axum::routing::post(|| async {
        let now = chrono::Utc::now().timestamp();
        let header = base64_url("{}");
        let payload = base64_url(
            &serde_json::json!({
                "sub": "app", "roles": ["admin"],
                "iat": now, "exp": now + 86400,
            })
            .to_string(),
        );
        let sig = base64_url("sig");
        let token = format!("{}.{}.{}", header, payload, sig);
        axum::Json(serde_json::json!({
            "access_token": token, "token_type": "Bearer", "expires_in": 86400,
        }))
    });

    let app = axum::Router::new()
        .route(
            "/",
            axum::routing::get(|| async { axum::response::Html(openerp_web::login_html()) }),
        )
        .route(
            "/dashboard",
            axum::routing::get(|| async { axum::response::Html(openerp_web::dashboard_html()) }),
        )
        .route(
            "/meta/schema",
            axum::routing::get(move || {
                let s = schema.clone();
                async move { axum::Json(s) }
            }),
        )
        .route(
            "/health",
            axum::routing::get(|| async { axum::Json(serde_json::json!({"status": "ok"})) }),
        )
        .route("/auth/login", login_handler)
        .nest(&format!("/app/{module_name}"), facet_router)
        .nest(&format!("/admin/{module_name}"), admin_router);

    tokio::spawn(async move {
        axum::serve(listener, app).await.ok();
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    Ok(server_url)
}

fn get_lan_ip() -> Option<String> {
    use std::net::UdpSocket;
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    socket.local_addr().ok().map(|a| a.ip().to_string())
}

fn base64_url(input: &str) -> String {
    use base64::Engine;
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(input.as_bytes())
}

fn bytes_to_ffi(bytes: Vec<u8>) -> FluxBytes {
    let len = bytes.len();
    let ptr = bytes.as_ptr();
    std::mem::forget(bytes);
    FluxBytes { ptr, len }
}
