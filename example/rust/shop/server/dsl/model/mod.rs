//! ShopFlux model definitions.
//!
//! Each model uses `#[model]` — the macro generates
//! serde, Field consts, IR metadata, and common fields.

pub mod address;
pub mod cart_item;
pub mod category;
pub mod order;
pub mod order_item;
pub mod product;
pub mod review;
pub mod shop;
pub mod user;

pub use address::Address;
pub use cart_item::CartItem;
pub use category::Category;
pub use order::Order;
pub use order_item::OrderItem;
pub use product::Product;
pub use review::Review;
pub use shop::Shop;
pub use user::User;
