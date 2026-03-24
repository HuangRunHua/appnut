//! Profile page state — stored at `profile/{user_id}`.

use super::auth::UserProfile;
use super::timeline::FeedItem;
use flux_derive::state;
use serde::{Deserialize, Serialize};

/// A user's profile page.
#[state("profile")]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfilePage {
    pub user: UserProfile,
    pub tweets: Vec<FeedItem>,
    pub followed_by_me: bool,
    pub loading: bool,
}

impl ProfilePage {
    pub fn path(user_id: &str) -> String {
        format!("profile/{}", user_id)
    }
}
