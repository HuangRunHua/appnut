use flux_derive::request;
use serde::{Deserialize, Serialize};

#[request("search/products")]
#[derive(Serialize, Deserialize)]
pub struct SearchProductsReq {
    pub query: String,
}
