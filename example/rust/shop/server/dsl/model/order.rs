use openerp_macro::model;
use openerp_types::*;

use super::{Shop, User};

/// An order placed by a buyer in a shop.
#[model(module = "shop", name = "shop/orders/{id}")]
pub struct Order {
    pub id: Id,
    pub buyer: Name<User>,
    pub shop: Name<Shop>,
    /// pending_payment | paid | shipped | completed | cancelled
    pub status: String,
    /// Total in cents.
    pub total_amount: u64,
    /// Serialised shipping address (JSON).
    pub shipping_address: String,
    /// JSON snapshot of order items at creation time.
    pub items_snapshot: String,
    pub paid_at: Option<String>,
    // display_name, description, metadata, created_at, updated_at → auto-injected
}
