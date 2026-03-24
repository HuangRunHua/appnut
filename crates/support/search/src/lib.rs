pub mod error;
pub mod tantivy;
pub mod traits;

pub use self::tantivy::TantivyEngine;
pub use error::SearchError;
pub use traits::{SearchEngine, SearchResult};
