use flux_derive::state;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoryItem {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub sort_order: u32,
}

#[state("catalog/categories")]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoriesState {
    pub items: Vec<CategoryItem>,
    pub loading: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductItem {
    pub id: String,
    pub title: String,
    pub price: u64,
    pub image: Option<String>,
    pub rating: u32,
    pub shop_name: String,
}

#[state("catalog/products")]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogProductsState {
    pub category_id: String,
    pub items: Vec<ProductItem>,
    pub loading: bool,
    pub has_more: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}
