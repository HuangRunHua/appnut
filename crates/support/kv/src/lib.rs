pub mod error;
pub mod file_loader;
pub mod overlay;
pub mod redb;
pub mod traits;

pub use error::KVError;
pub use file_loader::FileLoader;
pub use overlay::OverlayKV;
pub use redb::RedbStore;
pub use traits::KVStore;

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use crate::error::KVError;
    use crate::overlay::OverlayKV;
    use crate::redb::RedbStore;
    use crate::traits::KVStore;

    fn open_temp_store() -> (RedbStore, tempfile::TempDir) {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.redb");
        let store = RedbStore::open(&path).unwrap();
        (store, dir)
    }

    #[test]
    fn redb_store_basic_crud() {
        let (store, _dir) = open_temp_store();

        store.set("key1", b"value1").unwrap();
        assert_eq!(store.get("key1").unwrap(), Some(b"value1".to_vec()));

        store.set("key1", b"value1_updated").unwrap();
        assert_eq!(store.get("key1").unwrap(), Some(b"value1_updated".to_vec()));

        store.delete("key1").unwrap();
        assert_eq!(store.get("key1").unwrap(), None);
    }

    #[test]
    fn get_nonexistent_key_returns_none() {
        let (store, _dir) = open_temp_store();
        assert_eq!(store.get("nonexistent").unwrap(), None);
    }

    #[test]
    fn delete_nonexistent_key_does_not_panic() {
        let (store, _dir) = open_temp_store();
        store.delete("nonexistent").unwrap();
    }

    #[test]
    fn scan_with_prefix() {
        let (store, _dir) = open_temp_store();

        store.set("config:a:1", b"v1").unwrap();
        store.set("config:a:2", b"v2").unwrap();
        store.set("config:b:1", b"v3").unwrap();
        store.set("other:x", b"v4").unwrap();

        let results = store.scan("config:a:").unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0], ("config:a:1".to_string(), b"v1".to_vec()));
        assert_eq!(results[1], ("config:a:2".to_string(), b"v2".to_vec()));

        let results = store.scan("config:").unwrap();
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].0, "config:a:1");
        assert_eq!(results[1].0, "config:a:2");
        assert_eq!(results[2].0, "config:b:1");
    }

    #[test]
    fn scan_with_empty_prefix_returns_all_entries() {
        let (store, _dir) = open_temp_store();

        store.set("a:1", b"v1").unwrap();
        store.set("b:1", b"v2").unwrap();
        store.set("c:1", b"v3").unwrap();

        let results = store.scan("").unwrap();
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].0, "a:1");
        assert_eq!(results[1].0, "b:1");
        assert_eq!(results[2].0, "c:1");
    }

    #[test]
    fn batch_set_and_batch_delete() {
        let (store, _dir) = open_temp_store();

        store
            .batch_set(&[
                ("k1", b"v1" as &[u8]),
                ("k2", b"v2"),
                ("k3", b"v3"),
            ])
            .unwrap();

        assert_eq!(store.get("k1").unwrap(), Some(b"v1".to_vec()));
        assert_eq!(store.get("k2").unwrap(), Some(b"v2".to_vec()));
        assert_eq!(store.get("k3").unwrap(), Some(b"v3".to_vec()));

        store.batch_delete(&["k1", "k3"]).unwrap();

        assert_eq!(store.get("k1").unwrap(), None);
        assert_eq!(store.get("k2").unwrap(), Some(b"v2".to_vec()));
        assert_eq!(store.get("k3").unwrap(), None);
    }

    #[test]
    fn overlay_file_layer_visible_via_get() {
        let (db, _dir) = open_temp_store();
        let overlay = OverlayKV::new(db);

        overlay.insert_file_entry("file:key".to_string(), b"file_value".to_vec());
        assert_eq!(overlay.get("file:key").unwrap(), Some(b"file_value".to_vec()));
    }

    #[test]
    fn overlay_scan_merges_both_layers() {
        let (db, _dir) = open_temp_store();
        let overlay = OverlayKV::new(db);

        overlay.insert_file_entry("prefix:a".to_string(), b"file_a".to_vec());
        overlay.insert_file_entry("prefix:b".to_string(), b"file_b".to_vec());
        overlay.set("prefix:c", b"db_c").unwrap();

        let results = overlay.scan("prefix:").unwrap();
        assert_eq!(results.len(), 3);
        assert_eq!(results[0], ("prefix:a".to_string(), b"file_a".to_vec()));
        assert_eq!(results[1], ("prefix:b".to_string(), b"file_b".to_vec()));
        assert_eq!(results[2], ("prefix:c".to_string(), b"db_c".to_vec()));
    }

    #[test]
    fn overlay_write_readonly_key_returns_error() {
        let (db, _dir) = open_temp_store();
        let overlay = OverlayKV::new(db);

        overlay.insert_file_entry("readonly:key".to_string(), b"immutable".to_vec());

        let err = overlay.set("readonly:key", b"new_value").unwrap_err();
        assert!(matches!(err, KVError::ReadOnly(_)));

        let err = overlay.delete("readonly:key").unwrap_err();
        assert!(matches!(err, KVError::ReadOnly(_)));
    }
}
