use openerp_macro::model;
use openerp_types::*;

use super::{Product, User};

/// A shopping cart entry — one per (user, product) pair.
#[model(module = "shop")]
pub struct CartItem {
    pub id: Id,
    pub user: Name<User>,
    pub product: Name<Product>,
    pub quantity: u32,
    // display_name, description, metadata, created_at, updated_at → auto-injected
}
