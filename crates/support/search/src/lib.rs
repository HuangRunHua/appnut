pub mod error;
pub mod tantivy;
pub mod traits;

pub use error::SearchError;
pub use self::tantivy::TantivyEngine;
pub use traits::{SearchEngine, SearchResult};

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{SearchEngine, SearchResult, TantivyEngine};
    use tempfile::tempdir;

    fn new_engine() -> (tempfile::TempDir, TantivyEngine) {
        let dir = tempdir().unwrap();
        let engine = TantivyEngine::open(dir.path()).unwrap();
        (dir, engine)
    }

    #[test]
    fn test_index_and_search() {
        let (_dir, engine) = new_engine();
        let mut doc = HashMap::new();
        doc.insert("title".to_string(), "hello world".to_string());
        doc.insert("content".to_string(), "searchable text".to_string());

        engine.index("items", "doc1", doc).unwrap();

        let results = engine.search("items", "hello", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "doc1");
        assert_eq!(
            results[0].fields.get("title"),
            Some(&"hello world".to_string())
        );
        assert_eq!(
            results[0].fields.get("content"),
            Some(&"searchable text".to_string())
        );
    }

    #[test]
    fn test_delete() {
        let (_dir, engine) = new_engine();
        let mut doc = HashMap::new();
        doc.insert("content".to_string(), "to be deleted".to_string());

        engine.index("items", "doc1", doc).unwrap();

        let results = engine.search("items", "deleted", 10).unwrap();
        assert_eq!(results.len(), 1);

        engine.delete("items", "doc1").unwrap();

        let results = engine.search("items", "deleted", 10).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_update_reindex() {
        let (_dir, engine) = new_engine();
        let mut doc1 = HashMap::new();
        doc1.insert("content".to_string(), "original version".to_string());
        engine.index("items", "doc1", doc1).unwrap();

        let mut doc2 = HashMap::new();
        doc2.insert("content".to_string(), "updated version".to_string());
        engine.index("items", "doc1", doc2).unwrap();

        // Search for old term - should get nothing
        let results_old = engine.search("items", "original", 10).unwrap();
        assert!(results_old.is_empty());

        // Search for new term - should get latest version
        let results_new = engine.search("items", "updated", 10).unwrap();
        assert_eq!(results_new.len(), 1);
        assert_eq!(results_new[0].id, "doc1");
        assert_eq!(
            results_new[0].fields.get("content"),
            Some(&"updated version".to_string())
        );
    }

    #[test]
    fn test_multiple_collections() {
        let (_dir, engine) = new_engine();

        let mut doc_a = HashMap::new();
        doc_a.insert("content".to_string(), "collection a content".to_string());
        engine.index("col_a", "id1", doc_a).unwrap();

        let mut doc_b = HashMap::new();
        doc_b.insert("content".to_string(), "collection b content".to_string());
        engine.index("col_b", "id1", doc_b).unwrap();

        // Search in col_a - only col_a result
        let results_a = engine.search("col_a", "collection", 10).unwrap();
        assert_eq!(results_a.len(), 1);
        assert_eq!(results_a[0].fields.get("content"), Some(&"collection a content".to_string()));

        // Search in col_b - only col_b result
        let results_b = engine.search("col_b", "collection", 10).unwrap();
        assert_eq!(results_b.len(), 1);
        assert_eq!(results_b[0].fields.get("content"), Some(&"collection b content".to_string()));

        // Search for "col_a" specific term in col_b - empty
        let results_b_for_a = engine.search("col_b", "col_a", 10).unwrap();
        assert!(results_b_for_a.is_empty());
    }

    #[test]
    fn test_empty_search_results() {
        let (_dir, engine) = new_engine();
        let mut doc = HashMap::new();
        doc.insert("content".to_string(), "hello world".to_string());
        engine.index("items", "doc1", doc).unwrap();

        let results = engine.search("items", "nonexistent_term_xyz", 10).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_with_limit() {
        let (_dir, engine) = new_engine();

        for i in 0..5 {
            let mut doc = HashMap::new();
            doc.insert("content".to_string(), format!("common word {}", i));
            engine.index("items", &format!("doc{}", i), doc).unwrap();
        }

        let results = engine.search("items", "common", 2).unwrap();
        assert_eq!(results.len(), 2);

        let results = engine.search("items", "common", 10).unwrap();
        assert_eq!(results.len(), 5);

        let results = engine.search("items", "common", 1).unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_special_characters_in_query() {
        let (_dir, engine) = new_engine();
        let mut doc = HashMap::new();
        doc.insert("content".to_string(), "normal text".to_string());
        engine.index("items", "doc1", doc).unwrap();

        // Queries with special characters should not panic (may return Ok or Err)
        let _ = engine.search("items", "normal ( )", 10);
        let _ = engine.search("items", "normal AND text", 10);
        let _ = engine.search("items", "normal*", 10);
    }
}
