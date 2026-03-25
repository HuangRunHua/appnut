use openerp_macro::model;
use openerp_types::*;

use super::{Order, Product, User};

/// A product review left by a buyer after purchase.
#[model(module = "shop")]
pub struct Review {
    pub id: Id,
    pub user: Name<User>,
    pub product: Name<Product>,
    pub order: Name<Order>,
    /// 1 – 5 stars.
    pub rating: u32,
    pub content: String,
    // display_name, description, metadata, created_at, updated_at → auto-injected
}
