use flux_derive::request;
use serde::{Deserialize, Serialize};

#[request("address/load")]
#[derive(Serialize, Deserialize)]
pub struct LoadAddressListReq;
