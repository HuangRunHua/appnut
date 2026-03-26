//! ShopFlux request definitions.

pub mod address;
pub mod auth;
pub mod cart;
pub mod catalog;
pub mod order;
pub mod search;

pub use address::LoadAddressListReq;
pub use auth::{LoginReq, LogoutReq};
pub use cart::{AddToCartReq, LoadCartReq, UpdateCartReq};
pub use catalog::{LoadCategoriesReq, LoadCategoryProductsReq, LoadProductDetailReq};
pub use order::{CreateOrderReq, LoadOrderDetailReq, LoadOrderListReq, PayOrderReq};
pub use search::SearchProductsReq;
