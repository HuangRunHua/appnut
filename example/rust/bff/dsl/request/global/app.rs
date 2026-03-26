//! App lifecycle requests.

use flux_derive::request;
use serde::{Deserialize, Serialize};

/// Initialize app state.
#[request("app/initialize")]
#[derive(Serialize, Deserialize)]
pub struct InitializeReq;

/// Refresh timeline.
#[request("timeline/load")]
#[derive(Serialize, Deserialize)]
pub struct TimelineLoadReq;

/// Update a compose form field.
#[request("compose/update-field")]
#[derive(Serialize, Deserialize)]
pub struct ComposeUpdateReq {
    pub field: String,
    pub value: String,
}

/// Set the BFF locale (triggers inbox reload with correct language).
#[request("app/set-locale")]
#[derive(Serialize, Deserialize)]
pub struct SetLocaleReq {
    pub locale: String,
}
