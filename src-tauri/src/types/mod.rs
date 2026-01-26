use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VrcMrdUser {
    pub id: String,
    pub username: String,
    pub avatar_name: String,
    pub perf_rank: String,
    pub account_age: String, // e.g. "3y"
    pub join_time: String,   // e.g. "13:12"
    pub leave_time: String,  // e.g. "13:24"
    pub advisories: bool,
    pub age_verified: bool,
    pub platform: String,
}