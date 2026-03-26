use openerp_macro::model;
use openerp_types::*;

use super::User;

/// A shipping address belonging to a user.
#[model(module = "shop", name = "shop/addresses/{id}")]
pub struct Address {
    pub id: Id,
    pub user: Name<User>,
    pub recipient_name: String,
    pub phone: String,
    pub province: String,
    pub city: String,
    pub district: String,
    pub detail: String,
    pub is_default: bool,
    // display_name, description, metadata, created_at, updated_at → auto-injected
}
