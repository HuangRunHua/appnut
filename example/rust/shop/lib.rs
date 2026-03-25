//! ShopFlux — E-commerce example App for the Appnut framework.
//!
//! Structure mirrors TwitterFlux:
//! - `server/` — DSL models + facet handlers + admin CRUD
//! - `bff/dsl/state/` — BFF state definitions
//! - `bff/dsl/request/` — BFF request definitions
//! - `bff/src/` — handler implementations
//! - `module.rs` — FluxModule + ServerModule plugin registration

#[path = "server/src/mod.rs"]
pub mod server;

#[path = "bff/dsl/state/global/mod.rs"]
pub mod state;

#[path = "bff/dsl/request/global/mod.rs"]
pub mod request;

#[path = "bff/src/mod.rs"]
pub mod handlers;

pub mod module;

pub use module::{ShopModule, register_shop_module};
