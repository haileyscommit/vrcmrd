use serde::{Deserialize, Serialize};
use vrchatapi::models::{LimitedUserInstance, User};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum TrustRank {
    Nuisance,
    Visitor,
    NewUser,
    User,
    KnownUser,
    TrustedUser,
    Admin,
}

pub trait GetTrustRank {
    fn trust_rank(&self) -> TrustRank;
}
impl GetTrustRank for User {
    fn trust_rank(&self) -> TrustRank {
        if self.tags.contains(&"admin_moderator".to_string()) {
            TrustRank::Admin
        } else if self.tags.contains(&"system_trust_veteran".to_string()) {
            TrustRank::TrustedUser
        } else if self.tags.contains(&"system_trust_trusted".to_string()) {
            TrustRank::KnownUser
        } else if self.tags.contains(&"system_trust_basic".to_string()) {
            TrustRank::NewUser
        } else if self.tags.contains(&"system_trust_known".to_string()) {
            TrustRank::User
        } else if self.tags.contains(&"system_nuisance".to_string()) {
            TrustRank::Nuisance
        } else {
            TrustRank::Visitor
        }
    }
}
impl GetTrustRank for LimitedUserInstance {
    fn trust_rank(&self) -> TrustRank {
        if self.tags.contains(&"admin_moderator".to_string()) {
            TrustRank::Admin
        } else if self.tags.contains(&"system_trust_veteran".to_string()) {
            TrustRank::TrustedUser
        } else if self.tags.contains(&"system_trust_trusted".to_string()) {
            TrustRank::KnownUser
        } else if self.tags.contains(&"system_trust_basic".to_string()) {
            TrustRank::NewUser
        } else if self.tags.contains(&"system_trust_known".to_string()) {
            TrustRank::User
        } else if self.tags.contains(&"system_nuisance".to_string()) {
            TrustRank::Nuisance
        } else {
            TrustRank::Visitor
        }
    }
}