pub mod error;
pub mod file;
pub mod traits;

pub use error::BlobError;
pub use file::FileStore;
pub use traits::{BlobMeta, BlobStore};

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn new_store() -> (tempfile::TempDir, FileStore) {
        let dir = tempdir().expect("create temp dir");
        let store = FileStore::open(dir.path()).expect("open FileStore");
        (dir, store)
    }

    #[test]
    fn put_and_get() {
        let (_dir, store) = new_store();
        let data = b"hello world";
        store.put("test.bin", data).expect("put");
        let got = store.get("test.bin").expect("get").expect("some");
        assert_eq!(got, data);
    }

    #[test]
    fn delete() {
        let (_dir, store) = new_store();
        store.put("to_delete.bin", b"data").expect("put");
        store.delete("to_delete.bin").expect("delete");
        let got = store.get("to_delete.bin").expect("get");
        assert!(got.is_none());
    }

    #[test]
    fn exists() {
        let (_dir, store) = new_store();
        store.put("exists_check.bin", b"x").expect("put");
        assert!(store.exists("exists_check.bin").expect("exists"));
        store.delete("exists_check.bin").expect("delete");
        assert!(!store.exists("exists_check.bin").expect("exists"));
    }

    #[test]
    fn list_with_prefix() {
        let (_dir, store) = new_store();
        store.put("images/a.jpg", b"a").expect("put");
        store.put("images/b.png", b"b").expect("put");
        store.put("docs/readme.txt", b"c").expect("put");
        let list = store.list("images/").expect("list");
        assert_eq!(list.len(), 2);
        let keys: Vec<_> = list.iter().map(|m| m.key.as_str()).collect();
        assert!(keys.contains(&"images/a.jpg"));
        assert!(keys.contains(&"images/b.png"));
        assert!(!keys.contains(&"docs/readme.txt"));
    }

    #[test]
    fn list_all() {
        let (_dir, store) = new_store();
        store.put("a.bin", b"1").expect("put");
        store.put("b.bin", b"2").expect("put");
        store.put("subdir/c.bin", b"3").expect("put");
        let list = store.list("").expect("list");
        assert_eq!(list.len(), 3);
        let keys: Vec<_> = list.iter().map(|m| m.key.as_str()).collect();
        assert!(keys.contains(&"a.bin"));
        assert!(keys.contains(&"b.bin"));
        assert!(keys.contains(&"subdir/c.bin"));
    }

    #[test]
    fn overwrite() {
        let (_dir, store) = new_store();
        store.put("overwrite.bin", b"first").expect("put");
        store.put("overwrite.bin", b"second").expect("put");
        let got = store.get("overwrite.bin").expect("get").expect("some");
        assert_eq!(got, b"second");
    }

    #[test]
    fn get_nonexistent() {
        let (_dir, store) = new_store();
        let got = store.get("nonexistent.bin").expect("get");
        assert!(got.is_none());
    }

    #[test]
    fn empty_blob() {
        let (_dir, store) = new_store();
        store.put("empty.bin", &[]).expect("put");
        let got = store.get("empty.bin").expect("get").expect("some");
        assert!(got.is_empty());
    }

    #[test]
    fn nested_keys() {
        let (_dir, store) = new_store();
        let key = "images/photo.jpg";
        let data = b"image bytes";
        store.put(key, data).expect("put");
        let got = store.get(key).expect("get").expect("some");
        assert_eq!(got, data);
        assert!(store.exists(key).expect("exists"));
        let list = store.list("images/").expect("list");
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].key, key);
    }
}
