//! Twitter model definitions.
//!
//! Each model is a single file with `#[model]` — the macro generates
//! serde, Field consts, IR metadata, and common fields.

pub mod follow;
pub mod like;
pub mod message;
pub mod tweet;
pub mod user;

pub use follow::Follow;
pub use like::Like;
pub use message::Message;
pub use tweet::Tweet;
pub use user::User;
