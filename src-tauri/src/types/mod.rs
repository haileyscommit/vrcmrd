use serde::{Deserialize, Serialize};
pub mod advisories;
mod instance_id;
pub mod user;
pub mod xsoverlay;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VrcMrdUser {
    pub id: String,
    pub username: String,
    pub avatar_name: String,
    pub perf_rank: String,
    pub account_created: Option<i64>, // e.g. "3y"
    pub join_time: i64,               // e.g. "13:12"
    pub leave_time: Option<i64>,      // e.g. "13:24"
    pub advisories: Vec<advisories::ActiveAdvisory>,
    pub age_verified: bool,
    pub platform: Option<String>,
    pub trust_rank: Option<user::TrustRank>,
    pub groups: Vec<PartialGroup>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PartialGroup {
    pub id: String,
    pub name: String,
}

/* References:
* wrld_4a64b637-195f-475c-99bc-9d51e7aa52bc:16648~hidden(usr_81bb2af3-8c5e-4cc3-8046-c6646e6ab343)~region(use)
* wrld_a5e9ec13-36b1-4e63-ae0c-dab9023401f9:97887~friends(usr_50d961bc-ceec-4c59-873e-67f339aa1529)~region(use)
* wrld_976e4d46-ba35-4c6d-a7f6-38714ee38fbf:37669~group(grp_b33be984-4b29-46f2-a19b-3acfff8aac00)~groupAccessType(public)~region(us)
* wrld_f0907129-7bc7-4571-aae1-701e4561346f:70073~group(grp_c81502ae-eef7-4b3b-bd41-0df12d87963b)~groupAccessType(plus)~region(jp)
* wrld_f0907129-7bc7-4571-aae1-701e4561346f:98730~group(grp_c81502ae-eef7-4b3b-bd41-0df12d87963b)~groupAccessType(members)~region(jp)
* wrld_f0907129-7bc7-4571-aae1-701e4561346f:79862~private(usr_50d961bc-ceec-4c59-873e-67f339aa1529)~canRequestInvite~region(use)
*/
/// Information extracted from a VRChat instance ID.
/// Some information is not available in the instance ID string, such as the VRC+ instance name.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VrcMrdInstanceId {
    raw: String,
    // The "raw" instance ID (the number)
    pub id: String,
    // The world ID the instance belongs to
    pub world: String,
    /// The general type of instance. One of: "public", "friends+", "friends", "invite+", "invite", "group", "group+", "group public"
    pub instance_type: String,
    /// The ID of the owner. Starts with "usr_" for friends/invite/+ instances, "grp_" for group instances, and is empty for public instances.
    pub owner: Option<String>,
    /// Whether the instance is public (i.e., visible in the instance list). True for "public" and "group public" instances, false otherwise.
    pub public: bool,
    /// The region code of the instance. One of: "us", "usw", "eu", "jp"
    pub region: String,
}

impl ToString for VrcMrdInstanceId {
    fn to_string(&self) -> String {
        self.raw.clone()
    }
}
