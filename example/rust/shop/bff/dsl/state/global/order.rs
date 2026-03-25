use flux_derive::state;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderSummary {
    pub id: String,
    pub shop_name: String,
    pub status: String,
    pub total_amount: u64,
    pub item_count: u32,
    pub created_at: String,
}

#[state("order/list")]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderListState {
    pub items: Vec<OrderSummary>,
    pub loading: bool,
    pub has_more: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderLineItem {
    pub product_id: String,
    pub title: String,
    pub price: u64,
    pub quantity: u32,
}

#[state("order/detail")]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderDetailState {
    pub id: String,
    pub shop_name: String,
    pub status: String,
    pub total_amount: u64,
    pub items: Vec<OrderLineItem>,
    pub shipping_address: String,
    pub paid_at: Option<String>,
    pub created_at: String,
    pub loading: bool,
}
