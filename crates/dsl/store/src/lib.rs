//! OpenERP Store traits.
//!
//! Store traits define how models are persisted. A model implements
//! the trait to declare storage config (KEY, UNIQUE, INDEX) and hooks.
//! CRUD operations are provided by the framework.
//!
//! ```ignore
//! impl KvStore for User {
//!     const KEY: Field = Self::id;
//!     fn before_create(&mut self) { self.id = Id::new_uuid(); }
//! }
//! ```

pub mod admin;
pub mod facet;
pub mod format;
pub mod hierarchy;
pub mod kv;
pub mod schema;
pub mod search;
pub mod sql;
mod timestamp;
pub mod ui;
pub mod ui_macro;

pub use admin::{admin_kv_router, admin_sql_router};
pub use facet::FacetDef;
pub use format::{FacetListResponse, FacetResponse, negotiate_format};
pub use hierarchy::HierarchyNode;
pub use kv::{KvOps, KvStore};
pub use schema::{EnumDef, ModuleDef, ResourceDef, build_schema};
pub use search::{SearchOps, SearchStore};
pub use sql::{SqlOps, SqlStore};
pub use ui::{WidgetOverride, apply_overrides};
