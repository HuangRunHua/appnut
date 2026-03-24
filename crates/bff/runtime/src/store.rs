use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

use serde::Serialize;

use crate::trie::Trie;
use crate::value::SubscriptionId;

/// Callback type for state change notifications.
///
/// Receives the changed path and the JSON-serialized bytes of the new value.
pub type ChangeHandler = Arc<dyn Fn(&str, &[u8]) + Send + Sync>;

/// Per-path state store with Trie-based subscription routing.
///
/// All values are stored as JSON bytes (`Vec<u8>`). This decouples the store
/// from concrete business types — callers serialize on write and deserialize
/// on read.
///
/// - `set(path, value)` serializes `value` to JSON and stores the bytes.
/// - `set_bytes(path, bytes)` stores pre-serialized bytes directly.
/// - `get(path)` returns the stored JSON bytes.
/// - `scan(prefix)` lists all children under a prefix path.
/// - `subscribe(pattern, handler)` registers a change handler.
/// - `unsubscribe(pattern, id)` removes a handler.
///
/// Uses `BTreeMap` internally for ordered prefix scanning.
pub struct StateStore {
    values: RwLock<BTreeMap<String, Vec<u8>>>,
    handlers: Trie<HandlerEntry>,
    next_id: AtomicU64,
}

#[derive(Clone)]
struct HandlerEntry {
    id: SubscriptionId,
    handler: ChangeHandler,
}

impl StateStore {
    /// Create a new empty StateStore.
    pub fn new() -> Self {
        Self {
            values: RwLock::new(BTreeMap::new()),
            handlers: Trie::new(),
            next_id: AtomicU64::new(1),
        }
    }

    /// Serialize `value` to JSON bytes and store at the given path.
    ///
    /// Notifies all subscribers whose pattern matches this path.
    ///
    /// # Panics
    ///
    /// Panics if serialization fails (should not happen for valid `Serialize` types).
    pub fn set<T: Serialize>(&self, path: &str, value: T) {
        let bytes = serde_json::to_vec(&value).expect("StateStore::set: serialization failed");
        self.set_bytes(path, bytes);
    }

    /// Store pre-serialized JSON bytes at the given path.
    ///
    /// Notifies all subscribers whose pattern matches this path.
    pub fn set_bytes(&self, path: &str, bytes: Vec<u8>) {
        {
            let mut values = self.values.write().unwrap();
            values.insert(path.to_string(), bytes.clone());
        }
        let entries = self.handlers.match_topic(path);
        for entry in entries {
            (entry.handler)(path, &bytes);
        }
    }

    /// Get the stored JSON bytes at the given path.
    ///
    /// Returns `None` if no value is set at this path.
    pub fn get(&self, path: &str) -> Option<Vec<u8>> {
        let values = self.values.read().unwrap();
        values.get(path).cloned()
    }

    /// Remove the value at the given path.
    ///
    /// Returns the old JSON bytes if present. Does NOT notify subscribers.
    pub fn remove(&self, path: &str) -> Option<Vec<u8>> {
        let mut values = self.values.write().unwrap();
        values.remove(path)
    }

    /// Scan all entries whose path starts with `{prefix}/`.
    ///
    /// Does NOT include the exact `prefix` path itself — only children.
    /// Results are ordered by path (BTreeMap ordering).
    pub fn scan(&self, prefix: &str) -> Vec<(String, Vec<u8>)> {
        let values = self.values.read().unwrap();
        let scan_prefix = format!("{}/", prefix);
        values
            .range(scan_prefix.clone()..)
            .take_while(|(k, _)| k.starts_with(&scan_prefix))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    /// Check if a value exists at the given path.
    pub fn contains(&self, path: &str) -> bool {
        let values = self.values.read().unwrap();
        values.contains_key(path)
    }

    /// Get the total number of stored paths.
    pub fn len(&self) -> usize {
        let values = self.values.read().unwrap();
        values.len()
    }

    /// Check if the store is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Subscribe to state changes matching the given Trie pattern.
    ///
    /// The handler is called synchronously whenever `set` or `set_bytes`
    /// is called on a path that matches the pattern. The handler receives
    /// the path and the JSON bytes of the new value.
    ///
    /// Returns a `SubscriptionId` that can be used to unsubscribe.
    pub fn subscribe<F>(&self, pattern: &str, handler: F) -> SubscriptionId
    where
        F: Fn(&str, &[u8]) + Send + Sync + 'static,
    {
        let id = SubscriptionId(self.next_id.fetch_add(1, Ordering::Relaxed));
        let entry = HandlerEntry {
            id,
            handler: Arc::new(handler),
        };
        self.handlers.insert(pattern, entry);
        id
    }

    /// Unsubscribe a handler by its subscription ID and pattern.
    pub fn unsubscribe(&self, pattern: &str, id: SubscriptionId) {
        self.handlers.remove(pattern, |entry| entry.id == id);
    }

    /// Get a snapshot of all paths and JSON bytes.
    ///
    /// Returns entries ordered by path.
    pub fn snapshot(&self) -> Vec<(String, Vec<u8>)> {
        let values = self.values.read().unwrap();
        values.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    }

    /// Get all paths currently stored.
    pub fn paths(&self) -> Vec<String> {
        let values = self.values.read().unwrap();
        values.keys().cloned().collect()
    }
}

impl Default for StateStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use std::sync::atomic::AtomicU64;

    // ========================================================================
    // Basic get/set
    // ========================================================================

    #[test]
    fn set_and_get_u32() {
        let store = StateStore::new();
        store.set("counter", 42u32);

        let bytes = store.get("counter").unwrap();
        let v: u32 = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(v, 42);
    }

    #[test]
    fn set_and_get_string() {
        let store = StateStore::new();
        store.set("name", "hello");

        let bytes = store.get("name").unwrap();
        let v: String = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(v, "hello");
    }

    #[test]
    fn set_and_get_struct() {
        #[derive(Debug, PartialEq, Serialize, Deserialize)]
        struct AuthState {
            phase: String,
            busy: bool,
        }

        let store = StateStore::new();
        store.set(
            "auth/state",
            AuthState {
                phase: "authenticated".to_string(),
                busy: false,
            },
        );

        let bytes = store.get("auth/state").unwrap();
        let state: AuthState = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(state.phase, "authenticated");
        assert!(!state.busy);
    }

    #[test]
    fn get_missing_returns_none() {
        let store = StateStore::new();
        assert!(store.get("nonexistent").is_none());
    }

    #[test]
    fn set_overwrites_previous_value() {
        let store = StateStore::new();
        store.set("counter", 1u32);
        store.set("counter", 2u32);

        let bytes = store.get("counter").unwrap();
        let v: u32 = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(v, 2);
    }

    #[test]
    fn set_overwrites_different_type() {
        let store = StateStore::new();
        store.set("value", 42u32);
        store.set("value", "now a string");

        let bytes = store.get("value").unwrap();
        assert!(serde_json::from_slice::<u32>(&bytes).is_err());
        let v: String = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(v, "now a string");
    }

    #[test]
    fn get_returns_valid_json_bytes() {
        let store = StateStore::new();
        store.set("list", vec![1u32, 2, 3]);

        let bytes = store.get("list").unwrap();
        let v: Vec<u32> = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(v, vec![1, 2, 3]);
    }

    #[test]
    fn set_bytes_prebuilt() {
        let store = StateStore::new();
        let bytes = serde_json::to_vec(&42u32).unwrap();
        store.set_bytes("counter", bytes);

        let got = store.get("counter").unwrap();
        let v: u32 = serde_json::from_slice(&got).unwrap();
        assert_eq!(v, 42);
    }

    // ========================================================================
    // Remove
    // ========================================================================

    #[test]
    fn remove_existing_returns_value() {
        let store = StateStore::new();
        store.set("counter", 42u32);

        let old = store.remove("counter").unwrap();
        let v: u32 = serde_json::from_slice(&old).unwrap();
        assert_eq!(v, 42);
        assert!(store.get("counter").is_none());
    }

    #[test]
    fn remove_missing_returns_none() {
        let store = StateStore::new();
        assert!(store.remove("nonexistent").is_none());
    }

    #[test]
    fn remove_then_set_again() {
        let store = StateStore::new();
        store.set("counter", 1u32);
        store.remove("counter");
        store.set("counter", 2u32);

        let bytes = store.get("counter").unwrap();
        let v: u32 = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(v, 2);
    }

    // ========================================================================
    // Scan
    // ========================================================================

    #[test]
    fn scan_returns_children() {
        let store = StateStore::new();
        store.set("home/devices/items/1", "device-1");
        store.set("home/devices/items/2", "device-2");
        store.set("home/devices/items/3", "device-3");

        let results = store.scan("home/devices/items");
        assert_eq!(results.len(), 3);

        let v0: String = serde_json::from_slice(&results[0].1).unwrap();
        let v1: String = serde_json::from_slice(&results[1].1).unwrap();
        assert_eq!(v0, "device-1");
        assert_eq!(v1, "device-2");
    }

    #[test]
    fn scan_does_not_include_exact_prefix() {
        let store = StateStore::new();
        store.set("home/devices", "parent");
        store.set("home/devices/1", "child-1");
        store.set("home/devices/2", "child-2");

        let results = store.scan("home/devices");
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|(k, _)| k != "home/devices"));
    }

    #[test]
    fn scan_returns_nested_children() {
        let store = StateStore::new();
        store.set("a/b/c", 1u32);
        store.set("a/b/c/d", 2u32);
        store.set("a/b/c/d/e", 3u32);

        let results = store.scan("a/b");
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn scan_no_matches() {
        let store = StateStore::new();
        store.set("auth/state", 1u32);

        let results = store.scan("home/devices");
        assert!(results.is_empty());
    }

    #[test]
    fn scan_does_not_match_similar_prefix() {
        let store = StateStore::new();
        store.set("auth/state", 1u32);
        store.set("authorization/state", 2u32);

        let results = store.scan("auth");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, "auth/state");
    }

    #[test]
    fn scan_empty_store() {
        let store = StateStore::new();
        assert!(store.scan("any/prefix").is_empty());
    }

    #[test]
    fn scan_results_are_ordered() {
        let store = StateStore::new();
        store.set("items/c", 3u32);
        store.set("items/a", 1u32);
        store.set("items/b", 2u32);

        let results = store.scan("items");
        let paths: Vec<&str> = results.iter().map(|(k, _)| k.as_str()).collect();
        assert_eq!(paths, vec!["items/a", "items/b", "items/c"]);
    }

    // ========================================================================
    // Contains / len / is_empty
    // ========================================================================

    #[test]
    fn contains_existing() {
        let store = StateStore::new();
        store.set("auth/state", 1u32);

        assert!(store.contains("auth/state"));
        assert!(!store.contains("auth/terms"));
    }

    #[test]
    fn len_and_is_empty() {
        let store = StateStore::new();
        assert!(store.is_empty());
        assert_eq!(store.len(), 0);

        store.set("a", 1u32);
        assert!(!store.is_empty());
        assert_eq!(store.len(), 1);

        store.set("b", 2u32);
        assert_eq!(store.len(), 2);

        store.remove("a");
        assert_eq!(store.len(), 1);
    }

    // ========================================================================
    // Snapshot / paths
    // ========================================================================

    #[test]
    fn snapshot_returns_all() {
        let store = StateStore::new();
        store.set("a", 1u32);
        store.set("b", 2u32);
        store.set("c", 3u32);

        let snap = store.snapshot();
        assert_eq!(snap.len(), 3);
        let paths: Vec<&str> = snap.iter().map(|(k, _)| k.as_str()).collect();
        assert_eq!(paths, vec!["a", "b", "c"]);

        let v: u32 = serde_json::from_slice(&snap[0].1).unwrap();
        assert_eq!(v, 1);
    }

    #[test]
    fn snapshot_empty_store() {
        let store = StateStore::new();
        assert!(store.snapshot().is_empty());
    }

    #[test]
    fn paths_returns_all_keys() {
        let store = StateStore::new();
        store.set("auth/state", 1u32);
        store.set("app/route", 2u32);

        let mut paths = store.paths();
        paths.sort();
        assert_eq!(paths, vec!["app/route", "auth/state"]);
    }

    // ========================================================================
    // Subscribe — exact match
    // ========================================================================

    #[test]
    fn subscribe_exact_notifies_on_match() {
        let store = StateStore::new();
        let called = Arc::new(AtomicU64::new(0));
        let called_c = called.clone();

        store.subscribe("auth/state", move |path, _bytes| {
            assert_eq!(path, "auth/state");
            called_c.fetch_add(1, Ordering::Relaxed);
        });

        store.set("auth/state", 1u32);
        assert_eq!(called.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn subscribe_exact_does_not_notify_other_paths() {
        let store = StateStore::new();
        let called = Arc::new(AtomicU64::new(0));
        let called_c = called.clone();

        store.subscribe("auth/state", move |_path, _bytes| {
            called_c.fetch_add(1, Ordering::Relaxed);
        });

        store.set("auth/terms", 1u32);
        store.set("home/devices", 2u32);
        assert_eq!(called.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn subscribe_receives_correct_value() {
        let store = StateStore::new();
        let received = Arc::new(RwLock::new(None::<u32>));
        let received_c = received.clone();

        store.subscribe("counter", move |_path, bytes| {
            let v: u32 = serde_json::from_slice(bytes).unwrap();
            *received_c.write().unwrap() = Some(v);
        });

        store.set("counter", 42u32);
        assert_eq!(*received.read().unwrap(), Some(42));
    }

    #[test]
    fn subscribe_called_on_every_set() {
        let store = StateStore::new();
        let count = Arc::new(AtomicU64::new(0));
        let count_c = count.clone();

        store.subscribe("counter", move |_path, _bytes| {
            count_c.fetch_add(1, Ordering::Relaxed);
        });

        store.set("counter", 1u32);
        store.set("counter", 2u32);
        store.set("counter", 3u32);
        assert_eq!(count.load(Ordering::Relaxed), 3);
    }

    // ========================================================================
    // Subscribe — wildcard patterns
    // ========================================================================

    #[test]
    fn subscribe_single_wildcard() {
        let store = StateStore::new();
        let paths_seen = Arc::new(RwLock::new(Vec::<String>::new()));
        let paths_c = paths_seen.clone();

        store.subscribe("auth/+", move |path, _bytes| {
            paths_c.write().unwrap().push(path.to_string());
        });

        store.set("auth/state", 1u32);
        store.set("auth/terms", 2u32);
        store.set("home/devices", 3u32); // should NOT trigger

        let paths = paths_seen.read().unwrap();
        assert_eq!(paths.len(), 2);
        assert!(paths.contains(&"auth/state".to_string()));
        assert!(paths.contains(&"auth/terms".to_string()));
    }

    #[test]
    fn subscribe_multi_wildcard() {
        let store = StateStore::new();
        let count = Arc::new(AtomicU64::new(0));
        let count_c = count.clone();

        store.subscribe("auth/#", move |_path, _bytes| {
            count_c.fetch_add(1, Ordering::Relaxed);
        });

        store.set("auth/state", 1u32);
        store.set("auth/terms", 2u32);
        store.set("auth/deep/nested/path", 3u32);
        store.set("home/devices", 4u32); // should NOT trigger

        assert_eq!(count.load(Ordering::Relaxed), 3);
    }

    #[test]
    fn subscribe_root_wildcard() {
        let store = StateStore::new();
        let count = Arc::new(AtomicU64::new(0));
        let count_c = count.clone();

        store.subscribe("#", move |_path, _bytes| {
            count_c.fetch_add(1, Ordering::Relaxed);
        });

        store.set("auth/state", 1u32);
        store.set("home/devices", 2u32);
        store.set("any/path/at/all", 3u32);

        assert_eq!(count.load(Ordering::Relaxed), 3);
    }

    // ========================================================================
    // Multiple subscribers
    // ========================================================================

    #[test]
    fn multiple_subscribers_same_pattern() {
        let store = StateStore::new();
        let count_a = Arc::new(AtomicU64::new(0));
        let count_b = Arc::new(AtomicU64::new(0));
        let ca = count_a.clone();
        let cb = count_b.clone();

        store.subscribe("auth/state", move |_, _bytes| {
            ca.fetch_add(1, Ordering::Relaxed);
        });
        store.subscribe("auth/state", move |_, _bytes| {
            cb.fetch_add(1, Ordering::Relaxed);
        });

        store.set("auth/state", 1u32);

        assert_eq!(count_a.load(Ordering::Relaxed), 1);
        assert_eq!(count_b.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn multiple_subscribers_different_patterns() {
        let store = StateStore::new();
        let exact = Arc::new(AtomicU64::new(0));
        let wild = Arc::new(AtomicU64::new(0));
        let all = Arc::new(AtomicU64::new(0));
        let e = exact.clone();
        let w = wild.clone();
        let a = all.clone();

        store.subscribe("auth/state", move |_, _bytes| {
            e.fetch_add(1, Ordering::Relaxed);
        });
        store.subscribe("auth/+", move |_, _bytes| {
            w.fetch_add(1, Ordering::Relaxed);
        });
        store.subscribe("#", move |_, _bytes| {
            a.fetch_add(1, Ordering::Relaxed);
        });

        store.set("auth/state", 1u32);

        assert_eq!(exact.load(Ordering::Relaxed), 1);
        assert_eq!(wild.load(Ordering::Relaxed), 1);
        assert_eq!(all.load(Ordering::Relaxed), 1);
    }

    // ========================================================================
    // Unsubscribe
    // ========================================================================

    #[test]
    fn unsubscribe_stops_notifications() {
        let store = StateStore::new();
        let count = Arc::new(AtomicU64::new(0));
        let count_c = count.clone();

        let id = store.subscribe("auth/state", move |_, _bytes| {
            count_c.fetch_add(1, Ordering::Relaxed);
        });

        store.set("auth/state", 1u32);
        assert_eq!(count.load(Ordering::Relaxed), 1);

        store.unsubscribe("auth/state", id);
        store.set("auth/state", 2u32);
        assert_eq!(count.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn unsubscribe_one_keeps_others() {
        let store = StateStore::new();
        let count_a = Arc::new(AtomicU64::new(0));
        let count_b = Arc::new(AtomicU64::new(0));
        let ca = count_a.clone();
        let cb = count_b.clone();

        let id_a = store.subscribe("auth/state", move |_, _bytes| {
            ca.fetch_add(1, Ordering::Relaxed);
        });
        let _id_b = store.subscribe("auth/state", move |_, _bytes| {
            cb.fetch_add(1, Ordering::Relaxed);
        });

        store.unsubscribe("auth/state", id_a);
        store.set("auth/state", 1u32);

        assert_eq!(count_a.load(Ordering::Relaxed), 0);
        assert_eq!(count_b.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn unsubscribe_wildcard() {
        let store = StateStore::new();
        let count = Arc::new(AtomicU64::new(0));
        let count_c = count.clone();

        let id = store.subscribe("auth/#", move |_, _bytes| {
            count_c.fetch_add(1, Ordering::Relaxed);
        });

        store.set("auth/state", 1u32);
        assert_eq!(count.load(Ordering::Relaxed), 1);

        store.unsubscribe("auth/#", id);
        store.set("auth/state", 2u32);
        assert_eq!(count.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn unsubscribe_nonexistent_is_noop() {
        let store = StateStore::new();
        store.unsubscribe("auth/state", SubscriptionId(999));
    }

    // ========================================================================
    // Subscription IDs are unique
    // ========================================================================

    #[test]
    fn subscription_ids_are_monotonic() {
        let store = StateStore::new();

        let id1 = store.subscribe("a", |_, _| {});
        let id2 = store.subscribe("b", |_, _| {});
        let id3 = store.subscribe("c", |_, _| {});

        assert!(id1 != id2);
        assert!(id2 != id3);
        assert!(id1 != id3);
    }

    // ========================================================================
    // Notification ordering
    // ========================================================================

    #[test]
    fn subscriber_sees_value_after_store_updated() {
        let store = Arc::new(StateStore::new());
        let store_c = store.clone();

        store.subscribe("counter", move |path, _bytes| {
            let stored = store_c.get(path).unwrap();
            let v: u32 = serde_json::from_slice(&stored).unwrap();
            assert_eq!(v, 42);
        });

        store.set("counter", 42u32);
    }

    // ========================================================================
    // set_bytes also notifies
    // ========================================================================

    #[test]
    fn set_bytes_triggers_subscription() {
        let store = StateStore::new();
        let called = Arc::new(AtomicU64::new(0));
        let called_c = called.clone();

        store.subscribe("test", move |_, _bytes| {
            called_c.fetch_add(1, Ordering::Relaxed);
        });

        let bytes = serde_json::to_vec(&42u32).unwrap();
        store.set_bytes("test", bytes);
        assert_eq!(called.load(Ordering::Relaxed), 1);
    }

    // ========================================================================
    // Thread safety
    // ========================================================================

    #[test]
    fn concurrent_set_and_get() {
        use std::thread;

        let store = Arc::new(StateStore::new());
        let mut handles = vec![];

        let store_w = store.clone();
        handles.push(thread::spawn(move || {
            for i in 0u32..1000 {
                store_w.set(&format!("item/{}", i), i);
            }
        }));

        let store_r = store.clone();
        handles.push(thread::spawn(move || {
            for _ in 0..1000 {
                let _ = store_r.get("item/0");
                let _ = store_r.scan("item");
            }
        }));

        for h in handles {
            h.join().unwrap();
        }

        assert_eq!(store.len(), 1000);
    }

    #[test]
    fn concurrent_subscribe_and_set() {
        use std::thread;

        let store = Arc::new(StateStore::new());
        let total = Arc::new(AtomicU64::new(0));

        let total_c = total.clone();
        store.subscribe("#", move |_, _bytes| {
            total_c.fetch_add(1, Ordering::Relaxed);
        });

        let mut handles = vec![];
        for t in 0..4 {
            let store_c = store.clone();
            handles.push(thread::spawn(move || {
                for i in 0..100 {
                    store_c.set(&format!("thread/{}/{}", t, i), i as u32);
                }
            }));
        }

        for h in handles {
            h.join().unwrap();
        }

        assert_eq!(total.load(Ordering::Relaxed), 400);
        assert_eq!(store.len(), 400);
    }

    // ========================================================================
    // Default trait
    // ========================================================================

    #[test]
    fn default_creates_empty_store() {
        let store = StateStore::default();
        assert!(store.is_empty());
    }
}
