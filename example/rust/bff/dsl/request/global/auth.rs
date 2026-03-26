//! Auth requests.

use flux_derive::request;
use serde::{Deserialize, Serialize};

/// Login with username + password.
#[request("auth/login")]
#[derive(Serialize, Deserialize)]
pub struct LoginReq {
    pub username: String,
    pub password: String,
}

/// Logout — clear session.
#[request("auth/logout")]
#[derive(Serialize, Deserialize)]
pub struct LogoutReq;
