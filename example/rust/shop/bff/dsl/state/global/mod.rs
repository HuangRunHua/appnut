//! ShopFlux state definitions.

pub mod address;
pub mod auth;
pub mod cart;
pub mod catalog;
pub mod order;
pub mod product;
pub mod search;

pub use address::{AddressEntry, AddressListState};
pub use auth::{AuthPhase, AuthState};
pub use cart::{CartEntry, CartState};
pub use catalog::{CatalogProductsState, CategoriesState, CategoryItem, ProductItem};
pub use order::{OrderDetailState, OrderLineItem, OrderListState, OrderSummary};
pub use product::ProductDetailState;
pub use search::SearchResultsState;
