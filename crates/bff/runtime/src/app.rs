use std::future::Future;
use std::sync::Arc;

use crate::router::Router;
use crate::store::StateStore;
use crate::value::SubscriptionId;

/// Flux — the cross-platform state engine.
///
/// Three primitives, all path-based:
/// - `get(path)` — read stored JSON bytes at a path
/// - `emit(path, payload)` — send a JSON bytes request, Trie-routed to handler(s)
/// - `subscribe(pattern)` — observe state changes as JSON bytes
///
/// # Examples
///
/// ```ignore
/// let flux = Flux::new();
///
/// // Register a handler.
/// flux.on("auth/login", |path, payload, store| async move {
///     let req: LoginRequest = serde_json::from_slice(&payload).unwrap();
///     store.set("auth/state", AuthState { phase: "authenticated" });
/// });
///
/// // Subscribe to state changes.
/// flux.subscribe("auth/#", |path, bytes| {
///     println!("{} changed ({} bytes)", path, bytes.len());
/// });
///
/// // Emit a request.
/// let payload = serde_json::to_vec(&LoginRequest { .. }).unwrap();
/// flux.emit("auth/login", &payload).await;
///
/// // Read state.
/// let bytes = flux.get("auth/state").unwrap();
/// let state: AuthState = serde_json::from_slice(&bytes).unwrap();
/// ```
pub struct Flux {
    store: Arc<StateStore>,
    router: Router,
}

impl Flux {
    /// Create a new Flux instance with empty state and no handlers.
    pub fn new() -> Self {
        Self {
            store: Arc::new(StateStore::new()),
            router: Router::new(),
        }
    }

    // ====================================================================
    // State — read
    // ====================================================================

    /// Read the stored JSON bytes at a path.
    ///
    /// Returns `None` if no value is set.
    ///
    /// ```ignore
    /// let bytes = flux.get("auth/state")?;
    /// let auth: AuthState = serde_json::from_slice(&bytes)?;
    /// ```
    pub fn get(&self, path: &str) -> Option<Vec<u8>> {
        self.store.get(path)
    }

    /// Scan all state entries under a prefix path.
    ///
    /// Returns `(path, json_bytes)` pairs whose path starts with `{prefix}/`.
    /// Does NOT include the exact `prefix` path itself.
    /// Results are ordered by path.
    pub fn scan(&self, prefix: &str) -> Vec<(String, Vec<u8>)> {
        self.store.scan(prefix)
    }

    /// Check if a state value exists at the given path.
    pub fn contains(&self, path: &str) -> bool {
        self.store.contains(path)
    }

    /// Get the total number of state entries.
    pub fn len(&self) -> usize {
        self.store.len()
    }

    /// Check if the state store is empty.
    pub fn is_empty(&self) -> bool {
        self.store.is_empty()
    }

    /// Get a snapshot of all state entries as `(path, json_bytes)` pairs.
    pub fn snapshot(&self) -> Vec<(String, Vec<u8>)> {
        self.store.snapshot()
    }

    // ====================================================================
    // Requests — emit
    // ====================================================================

    /// Emit a request with JSON bytes payload.
    ///
    /// The payload is routed to all handlers matching the path via
    /// Trie pattern matching. Handlers execute sequentially.
    ///
    /// If no handler matches, this is a silent no-op.
    pub async fn emit(&self, path: &str, payload: &[u8]) {
        self.router
            .dispatch(path, payload.to_vec(), Arc::clone(&self.store))
            .await;
    }

    // ====================================================================
    // Requests — register handlers
    // ====================================================================

    /// Register an async request handler for a path pattern.
    ///
    /// The handler receives:
    /// - `path: String` — the matched request path
    /// - `payload: Vec<u8>` — JSON-serialized request payload
    /// - `store: Arc<StateStore>` — state store for reading/writing state
    ///
    /// Pattern supports MQTT-style wildcards (`+`, `#`).
    pub fn on<F, Fut>(&self, pattern: &str, handler: F)
    where
        F: Fn(String, Vec<u8>, Arc<StateStore>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.router.on(pattern, handler);
    }

    /// Check if any handler would match the given path.
    pub fn has_handler(&self, path: &str) -> bool {
        self.router.matches(path)
    }

    // ====================================================================
    // Subscriptions — observe state changes
    // ====================================================================

    /// Subscribe to state changes matching a Trie pattern.
    ///
    /// The handler receives the changed path and JSON bytes of the new value.
    /// Called synchronously on the thread that calls `set`.
    /// Pattern supports MQTT-style wildcards (`+`, `#`).
    ///
    /// Returns a `SubscriptionId` for unsubscribing.
    pub fn subscribe<F>(&self, pattern: &str, handler: F) -> SubscriptionId
    where
        F: Fn(&str, &[u8]) + Send + Sync + 'static,
    {
        self.store.subscribe(pattern, handler)
    }

    /// Unsubscribe a handler by its ID and the pattern it was registered with.
    pub fn unsubscribe(&self, pattern: &str, id: SubscriptionId) {
        self.store.unsubscribe(pattern, id);
    }

    // ====================================================================
    // Advanced
    // ====================================================================

    /// Get a reference to the underlying StateStore.
    ///
    /// Useful for handlers that need direct store access, or for testing.
    pub fn store(&self) -> &Arc<StateStore> {
        &self.store
    }
}

impl Default for Flux {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use std::sync::atomic::{AtomicU64, Ordering};

    // ========================================================================
    // Construction
    // ========================================================================

    #[test]
    fn new_creates_empty_flux() {
        let flux = Flux::new();
        assert!(flux.is_empty());
        assert_eq!(flux.len(), 0);
        assert!(flux.get("anything").is_none());
    }

    #[test]
    fn default_creates_empty_flux() {
        let flux = Flux::default();
        assert!(flux.is_empty());
    }

    // ========================================================================
    // State: get / contains / len
    // ========================================================================

    #[test]
    fn get_after_store_set() {
        let flux = Flux::new();
        flux.store().set("counter", 42u32);

        let bytes = flux.get("counter").unwrap();
        let v: u32 = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(v, 42);
    }

    #[test]
    fn contains_after_set() {
        let flux = Flux::new();
        flux.store().set("auth/state", 1u32);

        assert!(flux.contains("auth/state"));
        assert!(!flux.contains("auth/terms"));
    }

    #[test]
    fn len_tracks_entries() {
        let flux = Flux::new();
        assert_eq!(flux.len(), 0);

        flux.store().set("a", 1u32);
        flux.store().set("b", 2u32);
        assert_eq!(flux.len(), 2);
    }

    // ========================================================================
    // State: scan
    // ========================================================================

    #[test]
    fn scan_children() {
        let flux = Flux::new();
        flux.store().set("items/1", "a");
        flux.store().set("items/2", "b");
        flux.store().set("items/3", "c");
        flux.store().set("other", "x");

        let results = flux.scan("items");
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn scan_empty() {
        let flux = Flux::new();
        assert!(flux.scan("anything").is_empty());
    }

    // ========================================================================
    // State: snapshot
    // ========================================================================

    #[test]
    fn snapshot_returns_all_entries() {
        let flux = Flux::new();
        flux.store().set("a", 1u32);
        flux.store().set("b", 2u32);

        let snap = flux.snapshot();
        assert_eq!(snap.len(), 2);
    }

    // ========================================================================
    // Emit + Handler: basic flow
    // ========================================================================

    #[tokio::test]
    async fn emit_routes_to_handler() {
        let flux = Flux::new();
        let called = Arc::new(AtomicU64::new(0));
        let called_c = called.clone();

        flux.on("ping", move |_, _, _| {
            let c = called_c.clone();
            async move {
                c.fetch_add(1, Ordering::Relaxed);
            }
        });

        flux.emit("ping", &[]).await;
        assert_eq!(called.load(Ordering::Relaxed), 1);
    }

    #[tokio::test]
    async fn emit_no_handler_is_silent() {
        let flux = Flux::new();
        flux.emit("nonexistent", &[]).await;
    }

    #[tokio::test]
    async fn emit_json_payload() {
        #[derive(Debug, Serialize, Deserialize)]
        struct LoginReq {
            phone: String,
        }

        let flux = Flux::new();
        let received = Arc::new(std::sync::RwLock::new(String::new()));
        let r = received.clone();

        flux.on("auth/login", move |_, payload, _| {
            let r = r.clone();
            async move {
                let req: LoginReq = serde_json::from_slice(&payload).unwrap();
                *r.write().unwrap() = req.phone;
            }
        });

        let payload = serde_json::to_vec(&LoginReq {
            phone: "13800138000".into(),
        })
        .unwrap();
        flux.emit("auth/login", &payload).await;

        assert_eq!(*received.read().unwrap(), "13800138000");
    }

    // ========================================================================
    // Emit + Handler: state updates
    // ========================================================================

    #[tokio::test]
    async fn handler_sets_state() {
        let flux = Flux::new();

        flux.on("auth/login", |_, _, store: Arc<StateStore>| async move {
            store.set("auth/state", "authenticated");
        });

        flux.emit("auth/login", &[]).await;

        let bytes = flux.get("auth/state").unwrap();
        let v: String = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(v, "authenticated");
    }

    #[tokio::test]
    async fn handler_sets_multiple_states() {
        let flux = Flux::new();

        flux.on(
            "app/initialize",
            |_, _, store: Arc<StateStore>| async move {
                store.set("auth/state", "unauthenticated");
                store.set("auth/terms", false);
                store.set("app/route", "/onboarding");
            },
        );

        flux.emit("app/initialize", &[]).await;

        let v: String = serde_json::from_slice(&flux.get("auth/state").unwrap()).unwrap();
        assert_eq!(v, "unauthenticated");

        let v: bool = serde_json::from_slice(&flux.get("auth/terms").unwrap()).unwrap();
        assert!(!v);

        let v: String = serde_json::from_slice(&flux.get("app/route").unwrap()).unwrap();
        assert_eq!(v, "/onboarding");
    }

    #[tokio::test]
    async fn handler_reads_and_updates_state() {
        let flux = Flux::new();
        flux.store().set("counter", 0u32);

        flux.on("increment", |_, _, store: Arc<StateStore>| async move {
            let current: u32 = store
                .get("counter")
                .and_then(|bytes| serde_json::from_slice(&bytes).ok())
                .unwrap_or(0);
            store.set("counter", current + 1);
        });

        flux.emit("increment", &[]).await;
        flux.emit("increment", &[]).await;
        flux.emit("increment", &[]).await;

        let v: u32 = serde_json::from_slice(&flux.get("counter").unwrap()).unwrap();
        assert_eq!(v, 3);
    }

    // ========================================================================
    // Subscribe: notifications from emit
    // ========================================================================

    #[tokio::test]
    async fn subscribe_notified_by_handler_set() {
        let flux = Flux::new();
        let notified = Arc::new(AtomicU64::new(0));
        let n = notified.clone();

        flux.subscribe("auth/state", move |_path, _bytes| {
            n.fetch_add(1, Ordering::Relaxed);
        });

        flux.on("auth/login", |_, _, store: Arc<StateStore>| async move {
            store.set("auth/state", "authenticated");
        });

        flux.emit("auth/login", &[]).await;
        assert_eq!(notified.load(Ordering::Relaxed), 1);
    }

    #[tokio::test]
    async fn subscribe_wildcard_catches_handler_updates() {
        let flux = Flux::new();
        let paths_changed = Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
        let pc = paths_changed.clone();

        flux.subscribe("#", move |path, _bytes| {
            pc.lock().unwrap().push(path.to_string());
        });

        flux.on(
            "app/initialize",
            |_, _, store: Arc<StateStore>| async move {
                store.set("auth/state", "unauthenticated");
                store.set("auth/terms", false);
                store.set("app/route", "/onboarding");
            },
        );

        flux.emit("app/initialize", &[]).await;

        let paths = paths_changed.lock().unwrap();
        assert_eq!(paths.len(), 3);
        assert!(paths.contains(&"auth/state".to_string()));
        assert!(paths.contains(&"auth/terms".to_string()));
        assert!(paths.contains(&"app/route".to_string()));
    }

    #[tokio::test]
    async fn subscribe_receives_correct_value() {
        let flux = Flux::new();
        let received = Arc::new(std::sync::RwLock::new(None::<String>));
        let r = received.clone();

        flux.subscribe("auth/state", move |_path, bytes| {
            let s: String = serde_json::from_slice(bytes).unwrap();
            *r.write().unwrap() = Some(s);
        });

        flux.on("auth/login", |_, _, store: Arc<StateStore>| async move {
            store.set("auth/state", "authenticated");
        });

        flux.emit("auth/login", &[]).await;
        assert_eq!(*received.read().unwrap(), Some("authenticated".to_string()));
    }

    // ========================================================================
    // Unsubscribe
    // ========================================================================

    #[tokio::test]
    async fn unsubscribe_stops_notifications() {
        let flux = Flux::new();
        let count = Arc::new(AtomicU64::new(0));
        let c = count.clone();

        let id = flux.subscribe("auth/state", move |_, _bytes| {
            c.fetch_add(1, Ordering::Relaxed);
        });

        flux.on("update", |_, _, store: Arc<StateStore>| async move {
            store.set("auth/state", "x");
        });

        flux.emit("update", &[]).await;
        assert_eq!(count.load(Ordering::Relaxed), 1);

        flux.unsubscribe("auth/state", id);
        flux.emit("update", &[]).await;
        assert_eq!(count.load(Ordering::Relaxed), 1);
    }

    // ========================================================================
    // has_handler
    // ========================================================================

    #[test]
    fn has_handler_check() {
        let flux = Flux::new();
        flux.on("auth/login", |_, _, _| async {});

        assert!(flux.has_handler("auth/login"));
        assert!(!flux.has_handler("auth/logout"));
    }

    #[test]
    fn has_handler_wildcard() {
        let flux = Flux::new();
        flux.on("auth/#", |_, _, _| async {});

        assert!(flux.has_handler("auth/login"));
        assert!(flux.has_handler("auth/deep/path"));
        assert!(!flux.has_handler("home/devices"));
    }

    // ========================================================================
    // emit with pre-serialized bytes
    // ========================================================================

    #[tokio::test]
    async fn emit_prebuilt_bytes() {
        let flux = Flux::new();
        let called = Arc::new(AtomicU64::new(0));
        let called_c = called.clone();

        flux.on("test", move |_, payload, _| {
            let called = called_c.clone();
            async move {
                let v: u32 = serde_json::from_slice(&payload).unwrap();
                assert_eq!(v, 42);
                called.fetch_add(1, Ordering::Relaxed);
            }
        });

        let bytes = serde_json::to_vec(&42u32).unwrap();
        flux.emit("test", &bytes).await;
        assert_eq!(called.load(Ordering::Relaxed), 1);
    }

    // ========================================================================
    // Full flow: bff-alpha style
    // ========================================================================

    #[tokio::test]
    async fn full_flow_initialize_accept_terms_login() {
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        struct AuthState {
            phase: String,
            busy: bool,
        }

        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        struct TermsState {
            accepted: bool,
        }

        #[derive(Debug, Serialize, Deserialize)]
        struct AcceptTermsReq {
            accepted: bool,
        }

        let flux = Flux::new();

        flux.on(
            "app/initialize",
            |_, _, store: Arc<StateStore>| async move {
                store.set(
                    "auth/state",
                    AuthState {
                        phase: "unauthenticated".into(),
                        busy: false,
                    },
                );
                store.set("auth/terms", TermsState { accepted: false });
                store.set("app/route", "/onboarding");
            },
        );

        flux.on(
            "auth/accept-terms",
            |_, payload, store: Arc<StateStore>| async move {
                let req: AcceptTermsReq = serde_json::from_slice(&payload).unwrap();
                store.set(
                    "auth/terms",
                    TermsState {
                        accepted: req.accepted,
                    },
                );
                if req.accepted {
                    store.set("app/route", "/login");
                }
            },
        );

        flux.on("auth/login", |_, _, store: Arc<StateStore>| async move {
            store.set(
                "auth/state",
                AuthState {
                    phase: "authenticated".into(),
                    busy: false,
                },
            );
            store.set("app/route", "/home");
        });

        let timeline = Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
        let tl = timeline.clone();
        flux.subscribe("#", move |path, _bytes| {
            tl.lock().unwrap().push(path.to_string());
        });

        // 1. Initialize
        flux.emit("app/initialize", &[]).await;

        let auth: AuthState = serde_json::from_slice(&flux.get("auth/state").unwrap()).unwrap();
        assert_eq!(auth.phase, "unauthenticated");

        let terms: TermsState = serde_json::from_slice(&flux.get("auth/terms").unwrap()).unwrap();
        assert!(!terms.accepted);

        let route: String = serde_json::from_slice(&flux.get("app/route").unwrap()).unwrap();
        assert_eq!(route, "/onboarding");

        // 2. Accept terms
        let payload = serde_json::to_vec(&AcceptTermsReq { accepted: true }).unwrap();
        flux.emit("auth/accept-terms", &payload).await;

        let terms: TermsState = serde_json::from_slice(&flux.get("auth/terms").unwrap()).unwrap();
        assert!(terms.accepted);

        let route: String = serde_json::from_slice(&flux.get("app/route").unwrap()).unwrap();
        assert_eq!(route, "/login");

        // 3. Login
        flux.emit("auth/login", &[]).await;

        let auth: AuthState = serde_json::from_slice(&flux.get("auth/state").unwrap()).unwrap();
        assert_eq!(auth.phase, "authenticated");

        let route: String = serde_json::from_slice(&flux.get("app/route").unwrap()).unwrap();
        assert_eq!(route, "/home");

        let tl = timeline.lock().unwrap();
        assert!(tl.len() >= 7);
    }

    // ========================================================================
    // Compile-time: Flux is Send + Sync
    // ========================================================================

    fn _assert_flux_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<Flux>();
        assert_sync::<Flux>();
    }
}
