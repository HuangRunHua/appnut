//! Timeline state — stored at `timeline/feed`.

use super::auth::UserProfile;
use flux_derive::state;
use serde::{Deserialize, Serialize};

/// Home timeline feed.
#[state("timeline/feed")]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimelineFeed {
    pub items: Vec<FeedItem>,
    pub loading: bool,
    pub has_more: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// A single tweet rendered in a feed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedItem {
    pub tweet_id: String,
    pub author: UserProfile,
    pub content: String,
    pub like_count: u32,
    pub liked_by_me: bool,
    pub reply_count: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_to_id: Option<String>,
    pub created_at: String,
}
