use serde::{Deserialize, Serialize};
use vrchatapi::models::{LimitedUserInstance, User};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, PartialOrd, Eq, ts_rs::TS)]
#[ts(export)]
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

/// A common user type for API responses, to make things easier.
pub struct CommonUser {
    pub inner: LimitedUserInstance,
}

impl From<User> for CommonUser {
    fn from(user: User) -> CommonUser {
        CommonUser{inner: LimitedUserInstance {
            id: user.id,
            display_name: user.display_name,
            current_avatar_image_url: user.current_avatar_image_url,
            tags: user.tags,
            age_verification_status: user.age_verification_status,
            last_platform: user.last_platform,
            bio: user.bio.into(),
            date_joined: user.date_joined.into(),
            developer_type: user.developer_type,
            age_verified: user.age_verified,
            bio_links: user.bio_links.into(),
            allow_avatar_copying: user.allow_avatar_copying,
            current_avatar_tags: user.current_avatar_tags,
            current_avatar_thumbnail_image_url: user.current_avatar_thumbnail_image_url,
            profile_pic_override: user.profile_pic_override.into(),
            image_url: None,
            friend_key: user.friend_key.into(),
            is_friend: user.is_friend,
            last_activity: user.last_activity.into(),
            status: user.status,
            status_description: user.status_description.into(),
            last_mobile: user.last_mobile,
            platform: user.platform.into(),
            profile_pic_override_thumbnail: user.profile_pic_override_thumbnail.into(),
            pronouns: user.pronouns.into(),
            state: user.state.into(),
            user_icon: user.user_icon.into(),
        }}
    }
}

impl Into<LimitedUserInstance> for CommonUser {
    fn into(self) -> LimitedUserInstance {
        self.inner
    }
}

impl Into<LimitedUserInstance> for &CommonUser {
    fn into(self) -> LimitedUserInstance {
        self.inner.clone()
    }
}

impl From<LimitedUserInstance> for CommonUser {
    fn from(user: LimitedUserInstance) -> CommonUser {
        CommonUser{inner: user}
    }
}

impl From<&CommonUser> for CommonUser {
    fn from(user: &CommonUser) -> CommonUser {
        CommonUser{inner: user.inner.clone()}
    }
}