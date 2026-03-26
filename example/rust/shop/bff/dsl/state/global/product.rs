use flux_derive::state;
use serde::{Deserialize, Serialize};

#[state("product/detail")]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductDetailState {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub price: u64,
    pub stock: u32,
    pub images: Vec<String>,
    pub rating: u32,
    pub review_count: u32,
    pub shop_id: String,
    pub shop_name: String,
    pub loading: bool,
}
