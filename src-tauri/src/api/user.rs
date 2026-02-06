use std::{cell::RefCell, collections::HashMap, ops::Deref, sync::Mutex};

use chrono::NaiveDate;
use tauri::{AppHandle, Emitter, Manager};
use vrchatapi::models::{LimitedUserGroups, LimitedUserInstance};

use crate::{
    advisories::apply_templating,
    memory::{advisories::AdvisoryMemory, users::Users},
    notices::publish_notice,
    types::{
        advisories::{make_notice, ActiveAdvisory, AdvisoryCondition},
        user::{CommonUser, GetTrustRank},
        VrcMrdUser,
    },
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
    let user = try_request!(app.clone(), |config| {
        vrchatapi::apis::users_api::get_user(config, user_id)
    }, { wait_for_api_ready: true })
    .await;
    let groups = {
        let has_group_membership_advisory = {
            let advisories = app.state::<Mutex<AdvisoryMemory>>();
            let advisories = advisories.lock().unwrap();
            advisories.deref().has_group_membership_advisory
        };
        if has_group_membership_advisory {
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
                    groups
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

impl VrcMrdUser {
    pub fn update_from(
        &mut self,
        app: AppHandle,
        other: &CommonUser,
        groups: Vec<LimitedUserGroups>,
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
        self.advisories =
            with_advisories(app.clone(), other.into(), self.advisories.clone(), groups);
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
        self
    }
}

pub fn with_advisories(
    app: AppHandle,
    user: CommonUser,
    existing_advisories: Vec<ActiveAdvisory>,
    groups: Vec<LimitedUserGroups>,
) -> Vec<ActiveAdvisory> {
    let mut advisories = existing_advisories.clone();
    let user: LimitedUserInstance = user.into();
    let active_advisories = app
        .state::<Mutex<AdvisoryMemory>>()
        .lock()
        .unwrap()
        .deref()
        .active_advisories
        .clone();
    for advisory in active_advisories.iter() {
        // TODO: handle templating in advisory messages (pass some info up here)
        let relevant_group_id: RefCell<Option<String>> = RefCell::new(None);
        let templates = RefCell::new(HashMap::new());
        templates
            .borrow_mut()
            .insert("username", user.display_name.clone());
        if advisory.condition.evaluate(&|condition| match condition {
            AdvisoryCondition::UsernameContains(string) => user
                .display_name
                .to_lowercase()
                .contains(&string.to_lowercase()),
            AdvisoryCondition::AgeNotVerified => !user.age_verified,
            AdvisoryCondition::TrustRankAtMost(trust_rank) => user.trust_rank() <= trust_rank,
            AdvisoryCondition::PlatformIs(platform) => {
                user.last_platform.to_lowercase() == platform.to_lowercase()
            }
            AdvisoryCondition::IsGroupMember(group_id) => {
                if groups.is_empty() && advisories.iter().any(|a| a.id == advisory.id) {
                    // if the advisory is already active and groups are not available, assume the user is
                    // still in the group.
                    return true;
                }
                for group in groups.iter() {
                    if let Some(gid) = &group.group_id {
                        if *gid != group_id {
                            continue;
                        }
                        *relevant_group_id.borrow_mut() = Some(group_id.clone());
                        templates.borrow_mut().insert("group_id", group_id.clone());
                        templates
                            .borrow_mut()
                            .insert("group_name", group.name.clone().unwrap_or_default());
                        return true;
                    }
                }
                return false;
            }
            AdvisoryCondition::AccountAgeAtMostDays(days) => {
                if let Some(date_joined) = user.date_joined.clone() {
                    if let Ok(joined_date) =
                        chrono::NaiveDate::parse_from_str(&date_joined, "%Y-%m-%d")
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
                            user.id, date_joined
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
                        &user.id,
                        Some(format!("“{}” joined", user.display_name)),
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
    for advisory in existing_advisories.iter() {
        // Remove any advisories that are no longer active (or have been deleted)
        if !active_advisories.iter().any(|a| a.id == advisory.id) {
            advisories.retain(|a| a.id != advisory.id);
        }
    }
    advisories
    // TODO: emit notices for each advisory added
}
