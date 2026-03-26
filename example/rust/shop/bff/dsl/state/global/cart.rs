use flux_derive::state;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CartEntry {
    pub id: String,
    pub product_id: String,
    pub title: String,
    pub price: u64,
    pub image: Option<String>,
    pub quantity: u32,
}

#[state("cart/items")]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CartState {
    pub items: Vec<CartEntry>,
    pub total_amount: u64,
    pub loading: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}
