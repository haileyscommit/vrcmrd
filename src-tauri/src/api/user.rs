use std::{cell::RefCell, collections::HashMap, ops::Deref, sync::Mutex};

use chrono::NaiveDate;
use tauri::{AppHandle, Emitter, Manager};
use vrchatapi::models::LimitedUserInstance;

use crate::{
    advisories::apply_templating, memory::{advisories::AdvisoryMemory, users::Users}, notices::publish_notice,
    types::{
        PartialGroup, VrcMrdUser, advisories::{ActiveAdvisory, AdvisoryCondition, make_notice}, user::{CommonUser, GetTrustRank}
    }
};

/// Queries user information from the VRChat API and updates the user memory.
pub async fn query_user_info(app: AppHandle, user_id: &str) {
    // TODO: pull from cache instead if available (and cache epoch is current)
    // Check if user is already in memory (i.e. in instance)
    let base_user = {
        let users_state = app.state::<Mutex<Users>>();
        let users_state = users_state.lock().unwrap();
        users_state.inner.iter().find(|u| u.id == user_id).cloned()
    };
    if base_user.is_none() {
        // TODO: cache anyway
        return;
    }
    println!("Fetching user info for ID: {}", user_id);
    // TODO: don't get new user info until everyone's user info has been fetched once.
    let user = try_request!(app.clone(), |config| {
        vrchatapi::apis::users_api::get_user(config, user_id)
    }, { wait_for_api_ready: true })
    .await;
    if user.is_ok() && user.as_ref().unwrap().is_none() {
        eprintln!("User not found in list for ID: {}", user_id);
        return;
    }
    let groups = {
        let has_group_membership_advisory = {
            let advisories = app.state::<Mutex<AdvisoryMemory>>();
            let advisories = advisories.lock().unwrap();
            advisories.deref().has_group_membership_advisory
        };
        let base_groups = base_user.clone().unwrap().groups.clone();
        if !base_groups.is_empty() {
            // TODO: use a cache epoch to ignore the cache after a refresh (i.e. if someone joins/leaves a group and rejoins)
            // If we already have groups, use them (this avoids an unnecessary API call and also ensures we have group names for advisories)
            base_groups
        } else if has_group_membership_advisory {
            println!("Fetching group list for user ID: {}", user_id);
            let group_list = try_request!(app.clone(), |config| {
                vrchatapi::apis::users_api::get_user_groups(config, user_id)
            }, { wait_for_api_ready: true })
            .await;
            match group_list {
                Ok(Some(groups)) => {
                    println!(
                        "Fetched group list for user ID {}: {} groups",
                        user_id,
                        groups.len()
                    );
                    groups.iter().map(|v| PartialGroup {
                        id: v.clone().group_id.unwrap_or_default(),
                        name: v.clone().name.unwrap_or(v.clone().group_id.unwrap_or_default()),
                    }).collect()
                }
                _ => {
                    eprintln!("Failed to fetch group list for user ID: {}", user_id);
                    Vec::new()
                }
            }
        } else {
            Vec::new()
        }
    };
    let app = app.clone();
    match user {
        Ok(Some(user_info)) => {
            println!("Received user info for {} via API", user_id);
            // TODO: use the "system_probable_troll" field to add an advisory
            let mut vrcmrd_user = base_user.unwrap();
            let vrcmrd_user = vrcmrd_user.update_from(
                app.clone(),
                &CommonUser::from(user_info.clone()),
                groups.clone(),
            );
            let users_state = app.state::<Mutex<Users>>();
            let mut users_state = users_state.lock().unwrap();
            // Update or insert user
            if let Some(existing_user) = users_state
                .inner
                .iter_mut()
                .find(|u| u.id == vrcmrd_user.id)
            {
                *existing_user = vrcmrd_user.clone();
            } else {
                users_state.inner.push(vrcmrd_user.clone());
            }
            let _ = app.emit("vrcmrd:update-user", vrcmrd_user.clone());
        }
        Ok(None) => {
            eprintln!("User not found for ID: {}", user_id);
        }
        Err(e) => {
            eprintln!("Error fetching user info for ID {}: {:?}", user_id, e);
        }
    }
}

pub fn thread_query_user_info(app: AppHandle, user_id: &str) {
    let handle = app.clone();
    let user_id = user_id.to_string();
    tauri::async_runtime::spawn(async move {
        query_user_info(handle, &user_id).await;
    });
}

pub enum AdvisoryTrigger {
    JoinLeave,
    AvatarSwitched,
}

impl VrcMrdUser {
    pub fn update_from(
        &mut self,
        app: AppHandle,
        other: &CommonUser,
        groups: Vec<PartialGroup>,
    ) -> &mut Self {
        let user: LimitedUserInstance = other.into();
        self.age_verified = user.age_verified;
        if let Some(date_joined) = user.date_joined.clone() {
            self.account_created = NaiveDate::parse_from_str(&date_joined, "%Y-%m-%d")
                .ok()
                .and_then(|d| {
                    d.and_hms_opt(0, 0, 0)
                        .and_then(|r| Some(r.and_utc().timestamp()))
                });
        }
        self.trust_rank = Some(user.trust_rank());
        self.platform = {
            let platform = user.last_platform;
            if platform == "standalonewindows" {
                Some("pc".to_string())
            } else if platform == "android" {
                Some("android".to_string())
            } else if platform == "ios" {
                Some("ios".to_string())
            } else {
                None
            }
        };
        self.groups = groups;
        self.advisories =
            self.with_advisories(app.clone(), AdvisoryTrigger::JoinLeave);
        self
    }
    pub fn with_advisories(
        &self,
        app: AppHandle,
        trigger: AdvisoryTrigger,
    ) -> Vec<ActiveAdvisory> {
        let mut advisories = self.advisories.clone();
        //let user: LimitedUserInstance = user.into();
        let active_advisories = app
            .state::<Mutex<AdvisoryMemory>>()
            .lock()
            .unwrap()
            .deref()
            .active_advisories
            .clone();
        for advisory in active_advisories.iter() {
            let relevant_group_id: RefCell<Option<String>> = RefCell::new(None);
            let templates = RefCell::new(HashMap::new());
            templates
                .borrow_mut()
                    .insert("username", self.username.clone());
            if advisory.condition.evaluate(&|condition| match condition {
                    AdvisoryCondition::UsernameContains(string) => self
                        .username
                    .to_lowercase()
                    .contains(&string.to_lowercase()),
                    AdvisoryCondition::AgeNotVerified => !self.age_verified,
                    AdvisoryCondition::TrustRankAtMost(trust_rank) => self.trust_rank.is_some() && self.trust_rank.clone().unwrap() <= trust_rank,
                AdvisoryCondition::PlatformIs(platform) => {
                        self.platform.as_deref().unwrap_or("").to_lowercase() == platform.to_lowercase()
                }
                AdvisoryCondition::IsGroupMember(group_id) => {
                        if self.groups.is_empty() && advisories.iter().any(|a| a.id == advisory.id) {
                        // if the advisory is already active and groups are not available, assume the user is
                        // still in the group.
                        return true;
                    }
                        for group in self.groups.iter() {
                            if group.id != group_id {
                                continue;
                            }
                            *relevant_group_id.borrow_mut() = Some(group_id.clone());
                            templates.borrow_mut().insert("group_id", group_id.clone());
                            templates
                                .borrow_mut()
                                .insert("group_name", group.name.clone());
                            return true;
                        }
                    return false;
                }
                AdvisoryCondition::AccountAgeAtMostDays(days) => {
                        if let Some(date_joined) = self.account_created.clone() {
                            if let Some(joined_date) =
                                chrono::DateTime::from_timestamp(date_joined, 0)
                                    .map(|dt| dt.naive_utc().date())
                        {
                            let account_age_days =
                                (chrono::Local::now().naive_local().date() - joined_date).num_days();
                            // We add this template variable here because we have account_age_days here
                            templates
                                .borrow_mut()
                                .insert("account_age_days", account_age_days.to_string());
                            account_age_days <= days as i64
                        } else {
                            eprintln!(
                                "Failed to parse date_joined for user {}: {}",
                                    self.id, date_joined
                            );
                            false
                        }
                    } else {
                        advisories.iter().any(|a| a.id == advisory.id)
                    }
                }
                AdvisoryCondition::InstanceOwner(owner_id) => {
                    let instance_state = app.state::<crate::memory::instance::InstanceStateMutex>();
                    let instance_state = instance_state.lock().unwrap();
                    if let Some(owner) = instance_state.info.clone().unwrap().owner_id.flatten() {
                        if owner == owner_id {
                            return true;
                        }
                    }
                    if let Some(owner) = &instance_state.id_info.clone().and_then(|v| v.owner.clone()) {
                        if owner.as_str() == owner_id.as_str() {
                            return true;
                        } else {
                            return false;
                        }
                    }
                    false
                }
                AdvisoryCondition::AvatarNameContains(needle) => self.avatar_name.to_lowercase().contains(&needle.to_lowercase()),
                AdvisoryCondition::InGroupNameContains(needle) => {
                    let group = self.groups.iter().find(|g| g.name.to_lowercase().contains(&needle.to_lowercase()));
                    if let Some(group) = group {
                        templates.borrow_mut().insert("group_name", group.name.clone());
                        return true;
                    }
                    false
                }
                _ => {
                    println!(
                        "Advisory condition not implemented in user advisory evaluation: {:?}",
                        condition
                    );
                    advisories.iter().any(|a| a.id == advisory.id)
                    // If the condition is not implemented, keep the existing advisory if it's already present
                }
            }) {
                if advisories.iter().any(|a| a.id == advisory.id) {
                    // Update the existing advisory, especially if the advisory settings changed
                    for existing in advisories.iter_mut() {
                        if existing.id == advisory.id {
                            existing.message = apply_templating(
                                advisory.message_template.clone().as_str(),
                                &templates.borrow(),
                            );
                            existing.level = advisory.level.clone();
                            existing.relevant_group_id = relevant_group_id.borrow().clone();
                        }
                    }
                } else {
                    let active_advisory = ActiveAdvisory {
                        id: advisory.id.clone(),
                        message: apply_templating(
                            advisory.message_template.clone().as_str(),
                            &templates.borrow(),
                        ),
                        level: advisory.level.clone(),
                        relevant_group_id: relevant_group_id.borrow().clone(),
                    };
                    advisories.push(active_advisory.clone());
                    publish_notice(
                        app.clone(),
                        make_notice(
                            advisory,
                            &active_advisory,
                                &self.id,
                                Some(match trigger {
                                    AdvisoryTrigger::JoinLeave => format!("“{}” joined", self.username),
                                    //AdvisoryTrigger::UserInfoUpdated => format!("User info updated for “{}”", self.username),
                                    AdvisoryTrigger::AvatarSwitched => format!("“{}” changed to avatar “{}”", self.username, self.avatar_name),
                                }),
                        ),
                    )
                    .unwrap_or_else(|e| {
                        eprintln!(
                            "Failed to publish notice for advisory {}: {}",
                            advisory.id, e
                        );
                    });
                }
            } else {
                // If the advisory condition is not met, remove it if it exists
                advisories.retain(|a| a.id != advisory.id);
            }
        }
            for advisory in self.advisories.iter() {
            // Remove any advisories that are no longer active (or have been deleted)
            if !active_advisories.iter().any(|a| a.id == advisory.id) {
                advisories.retain(|a| a.id != advisory.id);
            }
        }
        advisories
        // TODO: emit notices for each advisory added
    }
}