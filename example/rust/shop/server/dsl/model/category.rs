use openerp_macro::model;
use openerp_types::*;

/// Product category — supports hierarchy via optional parent.
#[model(module = "shop", name = "shop/categories/{id}")]
pub struct Category {
    pub id: Id,
    pub cat_name: LocalizedText,
    pub parent: Option<Name<Category>>,
    pub sort_order: u32,
    // display_name, description, metadata, created_at, updated_at → auto-injected
}
