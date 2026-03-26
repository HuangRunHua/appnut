//! Settings requests.

use flux_derive::request;
use serde::{Deserialize, Serialize};

/// Load current user's profile into settings form.
#[request("settings/load")]
#[derive(Serialize, Deserialize)]
pub struct SettingsLoadReq;

/// Save profile changes (display name, bio).
#[request("settings/save")]
#[derive(Serialize, Deserialize)]
pub struct SettingsSaveReq {
    pub display_name: String,
    pub bio: String,
}

/// Change password.
#[request("settings/change-password")]
#[derive(Serialize, Deserialize)]
pub struct ChangePasswordReq {
    pub old_password: String,
    pub new_password: String,
}
