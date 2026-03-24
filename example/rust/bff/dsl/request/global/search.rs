//! Search requests.

use flux_derive::request;
use serde::{Deserialize, Serialize};

/// Search users and tweets by keyword.
#[request("search/query")]
#[derive(Serialize, Deserialize)]
pub struct SearchReq {
    pub query: String,
}

impl SearchReq {
    pub const CLEAR_PATH: &'static str = "search/clear";
}

/// Clear search results.
#[request("search/clear")]
#[derive(Serialize, Deserialize)]
pub struct SearchClearReq;
