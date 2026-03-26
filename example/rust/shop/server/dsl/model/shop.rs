use openerp_macro::model;
use openerp_types::*;

use super::User;

/// A seller's store.
#[model(module = "shop", name = "shop/shops/{id}")]
pub struct Shop {
    pub id: Id,
    pub owner: Name<User>,
    pub name: String,
    pub shop_description: Option<String>,
    pub avatar: Option<Avatar>,
    pub rating: u32,
    pub product_count: u32,
    // display_name, description, metadata, created_at, updated_at → auto-injected
}
