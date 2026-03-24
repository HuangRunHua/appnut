pub mod error;
pub mod sqlite;
pub mod traits;

pub use error::SQLError;
pub use sqlite::SqliteStore;
pub use traits::{Row, SQLStore, Value};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_crud() {
        let store = SqliteStore::open_in_memory().unwrap();
        let empty: Vec<Value> = vec![];

        store.exec("CREATE TABLE t1 (id INTEGER PRIMARY KEY, name TEXT)", &empty).unwrap();

        store.exec("INSERT INTO t1 (id, name) VALUES (1, 'alice')", &empty).unwrap();
        store.exec("INSERT INTO t1 (id, name) VALUES (?, ?)", &[Value::Integer(2), Value::Text("bob".into())]).unwrap();

        let rows = store.query("SELECT id, name FROM t1 ORDER BY id", &empty).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].get_i64("id"), Some(1));
        assert_eq!(rows[0].get_str("name"), Some("alice"));
        assert_eq!(rows[1].get_i64("id"), Some(2));
        assert_eq!(rows[1].get_str("name"), Some("bob"));

        let n = store.exec("UPDATE t1 SET name = ? WHERE id = 1", &[Value::Text("alice2".into())]).unwrap();
        assert_eq!(n, 1);

        let rows = store.query("SELECT name FROM t1 WHERE id = 1", &empty).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].get_str("name"), Some("alice2"));

        let n = store.exec("DELETE FROM t1 WHERE id = 2", &empty).unwrap();
        assert_eq!(n, 1);

        let rows = store.query("SELECT * FROM t1", &empty).unwrap();
        assert_eq!(rows.len(), 1);
    }

    #[test]
    fn test_parameter_binding() {
        let store = SqliteStore::open_in_memory().unwrap();
        store.exec("CREATE TABLE params (i INTEGER, r REAL, t TEXT, b BLOB, n INTEGER)", &[]).unwrap();

        store.exec(
            "INSERT INTO params (i, r, t, b, n) VALUES (?, ?, ?, ?, ?)",
            &[
                Value::Integer(42),
                Value::Real(3.14),
                Value::Text("hello".into()),
                Value::Blob(b"bytes".to_vec()),
                Value::Null,
            ],
        )
        .unwrap();

        let rows = store.query("SELECT i, r, t, b, n FROM params", &[]).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].get_i64("i"), Some(42));
        assert_eq!(rows[0].get_f64("r"), Some(3.14));
        assert_eq!(rows[0].get_str("t"), Some("hello"));
        match rows[0].get("b") {
            Some(Value::Blob(b)) => assert_eq!(b, b"bytes"),
            _ => panic!("expected Blob"),
        }
        assert!(matches!(rows[0].get("n"), Some(Value::Null)));
    }

    #[test]
    fn test_row_access_valid_columns() {
        let store = SqliteStore::open_in_memory().unwrap();
        store.exec("CREATE TABLE row_test (id INTEGER, name TEXT, score REAL)", &[]).unwrap();
        store.exec("INSERT INTO row_test VALUES (1, 'x', 2.5)", &[]).unwrap();

        let rows = store.query("SELECT id, name, score FROM row_test", &[]).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].get_str("name"), Some("x"));
        assert_eq!(rows[0].get_i64("id"), Some(1));
        assert_eq!(rows[0].get_f64("score"), Some(2.5));
    }

    #[test]
    fn test_row_access_invalid_columns() {
        let store = SqliteStore::open_in_memory().unwrap();
        store.exec("CREATE TABLE row_test2 (id INTEGER)", &[]).unwrap();
        store.exec("INSERT INTO row_test2 VALUES (1)", &[]).unwrap();

        let rows = store.query("SELECT id FROM row_test2", &[]).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].get_str("nonexistent"), None);
        assert_eq!(rows[0].get_i64("nonexistent"), None);
        assert_eq!(rows[0].get_f64("nonexistent"), None);
    }

    #[test]
    fn test_in_memory_database() {
        let store = SqliteStore::open_in_memory().unwrap();
        store.exec("CREATE TABLE mem (x INTEGER)", &[]).unwrap();
        store.exec("INSERT INTO mem VALUES (123)", &[]).unwrap();
        let rows = store.query("SELECT x FROM mem", &[]).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].get_i64("x"), Some(123));
    }

    #[test]
    fn test_error_invalid_sql() {
        let store = SqliteStore::open_in_memory().unwrap();
        let err = store.exec("INVALID SYNTAX !!", &[]).unwrap_err();
        assert!(matches!(err, SQLError::Execution(_)));
    }

    #[test]
    fn test_error_invalid_query_sql() {
        let store = SqliteStore::open_in_memory().unwrap();
        let err = store.query("SELECT * FROM nonexistent_table_xyz", &[]).unwrap_err();
        assert!(matches!(err, SQLError::Query(_)));
    }

    #[test]
    fn test_multiple_tables_isolated() {
        let store = SqliteStore::open_in_memory().unwrap();
        store.exec("CREATE TABLE a (id INTEGER)", &[]).unwrap();
        store.exec("CREATE TABLE b (id INTEGER)", &[]).unwrap();

        store.exec("INSERT INTO a VALUES (1)", &[]).unwrap();
        store.exec("INSERT INTO b VALUES (2)", &[]).unwrap();

        let rows_a = store.query("SELECT * FROM a", &[]).unwrap();
        let rows_b = store.query("SELECT * FROM b", &[]).unwrap();
        assert_eq!(rows_a.len(), 1);
        assert_eq!(rows_b.len(), 1);
        assert_eq!(rows_a[0].get_i64("id"), Some(1));
        assert_eq!(rows_b[0].get_i64("id"), Some(2));
    }

    #[test]
    fn test_wal_mode() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.db");
        let store = SqliteStore::open(path.as_path()).unwrap();

        let rows = store.query("PRAGMA journal_mode", &[]).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].get_str("journal_mode"), Some("wal"));
    }
}
