use openerp_macro::model;
use openerp_types::*;

/// A shop user — can be a buyer or seller.
#[model(module = "shop", name = "shop/users/{id}")]
pub struct User {
    pub id: Id,
    pub username: String,
    pub password_hash: Option<PasswordHash>,
    pub avatar: Option<Avatar>,
    /// "buyer" or "seller"
    pub role: String,
    // display_name, description, metadata, created_at, updated_at → auto-injected
}
