use flux_derive::state;
use serde::{Deserialize, Serialize};

#[state("auth/state")]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthState {
    pub phase: AuthPhase,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    pub busy: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AuthPhase {
    Unauthenticated,
    Authenticated,
}
