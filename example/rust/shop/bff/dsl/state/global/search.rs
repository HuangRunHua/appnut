use flux_derive::state;
use serde::{Deserialize, Serialize};

use super::catalog::ProductItem;

#[state("search/results")]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResultsState {
    pub query: String,
    pub items: Vec<ProductItem>,
    pub loading: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}
