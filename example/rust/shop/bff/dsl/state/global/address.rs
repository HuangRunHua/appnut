use flux_derive::state;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddressEntry {
    pub id: String,
    pub recipient_name: String,
    pub phone: String,
    pub full_address: String,
    pub is_default: bool,
}

#[state("address/list")]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddressListState {
    pub items: Vec<AddressEntry>,
    pub loading: bool,
}
