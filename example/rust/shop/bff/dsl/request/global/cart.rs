use flux_derive::request;
use serde::{Deserialize, Serialize};

#[request("cart/load")]
#[derive(Serialize, Deserialize)]
pub struct LoadCartReq;

#[request("cart/add")]
#[derive(Serialize, Deserialize)]
pub struct AddToCartReq {
    pub product_id: String,
    pub quantity: u32,
}

#[request("cart/update")]
#[derive(Serialize, Deserialize)]
pub struct UpdateCartReq {
    pub item_id: String,
    pub quantity: u32,
}
