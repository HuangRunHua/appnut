use openerp_macro::model;
use openerp_types::*;

use super::{Order, Product};

/// A line item inside an order — snapshot of the product at purchase time.
#[model(module = "shop")]
pub struct OrderItem {
    pub id: Id,
    pub order: Name<Order>,
    pub product: Name<Product>,
    pub title_snapshot: String,
    pub price_snapshot: u64,
    pub quantity: u32,
    // display_name, description, metadata, created_at, updated_at → auto-injected
}
