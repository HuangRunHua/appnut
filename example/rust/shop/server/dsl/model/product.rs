use openerp_macro::model;
use openerp_types::*;

use super::{Category, Shop};

/// A product listed in a shop.
#[model(module = "shop", name = "shop/products/{id}")]
pub struct Product {
    pub id: Id,
    pub shop: Name<Shop>,
    pub category: Name<Category>,
    pub title: String,
    pub product_description: Option<String>,
    /// Price in cents to avoid floating-point precision issues.
    pub price: u64,
    pub stock: u32,
    /// JSON array of image URLs, e.g. `["https://picsum.photos/400"]`.
    pub images: String,
    pub rating: u32,
    pub review_count: u32,
    /// "on_sale" or "off_sale"
    pub status: String,
    // display_name, description, metadata, created_at, updated_at → auto-injected
}
