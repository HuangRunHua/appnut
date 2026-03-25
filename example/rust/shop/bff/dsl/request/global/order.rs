use flux_derive::request;
use serde::{Deserialize, Serialize};

#[request("order/create")]
#[derive(Serialize, Deserialize)]
pub struct CreateOrderReq {
    pub cart_item_ids: Vec<String>,
    pub address_id: String,
}

#[request("order/load_list")]
#[derive(Serialize, Deserialize)]
pub struct LoadOrderListReq;

#[request("order/load_detail")]
#[derive(Serialize, Deserialize)]
pub struct LoadOrderDetailReq {
    pub order_id: String,
}

#[request("order/pay")]
#[derive(Serialize, Deserialize)]
pub struct PayOrderReq {
    pub order_id: String,
}
