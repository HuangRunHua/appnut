use flux_derive::request;
use serde::{Deserialize, Serialize};

#[request("catalog/load_categories")]
#[derive(Serialize, Deserialize)]
pub struct LoadCategoriesReq;

#[request("catalog/load_products")]
#[derive(Serialize, Deserialize)]
pub struct LoadCategoryProductsReq {
    pub category_id: String,
}

#[request("product/load_detail")]
#[derive(Serialize, Deserialize)]
pub struct LoadProductDetailReq {
    pub product_id: String,
}
