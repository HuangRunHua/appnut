use flux_derive::request;
use serde::{Deserialize, Serialize};

#[request("auth/login")]
#[derive(Serialize, Deserialize)]
pub struct LoginReq {
    pub username: String,
    pub password: String,
}

#[request("auth/logout")]
#[derive(Serialize, Deserialize)]
pub struct LogoutReq;
