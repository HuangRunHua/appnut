//! Tweet requests.

use flux_derive::request;
use serde::{Deserialize, Serialize};

/// Create a new tweet (or reply).
#[request("tweet/create")]
#[derive(Serialize, Deserialize)]
pub struct CreateTweetReq {
    pub content: String,
    pub reply_to_id: Option<String>,
}

/// Like a tweet.
#[request("tweet/like")]
#[derive(Serialize, Deserialize)]
pub struct LikeTweetReq {
    pub tweet_id: String,
}

/// Unlike a tweet.
#[request("tweet/unlike")]
#[derive(Serialize, Deserialize)]
pub struct UnlikeTweetReq {
    pub tweet_id: String,
}

/// Load tweet detail with replies.
#[request("tweet/load")]
#[derive(Serialize, Deserialize)]
pub struct LoadTweetReq {
    pub tweet_id: String,
}
