//! User requests.

use flux_derive::request;
use serde::{Deserialize, Serialize};

/// Follow a user.
#[request("user/follow")]
#[derive(Serialize, Deserialize)]
pub struct FollowUserReq {
    pub user_id: String,
}

/// Unfollow a user.
#[request("user/unfollow")]
#[derive(Serialize, Deserialize)]
pub struct UnfollowUserReq {
    pub user_id: String,
}

/// Load a user's profile page.
#[request("profile/load")]
#[derive(Serialize, Deserialize)]
pub struct LoadProfileReq {
    pub user_id: String,
}
