use std::sync::Mutex;

use chrono::NaiveDate;
use tauri::{AppHandle, Emitter, Manager};
use vrchatapi::models::LimitedUserInstance;

use crate::{memory::users::Users, types::{VrcMrdUser, advisories::{ActiveAdvisory}, user::{CommonUser, GetTrustRank, TrustRank}}};

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
    }, { wait_for_api_ready: true }).await;
    let app = app.clone();
    match user {
        Ok(Some(user_info)) => {
            println!("Received user info for {} via API", user_id);
            // TODO: introduce advisory for account age
            // TODO: use the "system_probable_troll" field to add an advisory
            let vrcmrd_user = VrcMrdUser {
                age_verified: user_info.clone().age_verified,
                account_created: NaiveDate::parse_from_str(&user_info.date_joined, "%Y-%m-%d").ok().and_then(|d| d.and_hms_opt(0, 0, 0).and_then(|r| Some(r.and_utc().timestamp()))),
                advisories: with_advisories(user_info.clone().into(), base_user.clone().unwrap().advisories),
                platform: {
                    let platform = user_info.clone().last_platform;
                    if platform == "standalonewindows" {
                        Some("pc".to_string())
                    } else if platform == "android" {
                        Some("android".to_string())
                    } else if platform == "ios" {
                        Some("ios".to_string())
                    } else {
                        None
                    }
                },
                trust_rank: Some(user_info.trust_rank()),
                ..base_user.unwrap().clone()
            };
            let users_state = app.state::<Mutex<Users>>();
            let mut users_state = users_state.lock().unwrap();
            // Update or insert user
            if let Some(existing_user) = users_state.inner.iter_mut().find(|u| u.id == vrcmrd_user.id) {
                *existing_user = vrcmrd_user.clone();
            } else {
                users_state.inner.push(vrcmrd_user.clone());
            }
            let _ = app.emit("vrcmrd:update-user", vrcmrd_user);
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

pub fn with_advisories(user: CommonUser, existing_advisories: Vec<ActiveAdvisory>) -> Vec<ActiveAdvisory> {
    let mut advisories = existing_advisories.clone();
    let user: LimitedUserInstance = user.into();
    match user.trust_rank() {
        TrustRank::Nuisance => {
            if !advisories.iter().any(|a| a.id == format!("vrcmrd:trust_rank:{:?}", user.trust_rank())) {
                advisories.push(ActiveAdvisory { 
                    id: format!("vrcmrd:trust_rank:{:?}", user.trust_rank()),
                    message: "User is Nuisance rank.".to_string(),
                    level: crate::types::advisories::AdvisoryLevel::High,
                    relevant_group_id: None,
                });
            }
        },
        _ => {}
    };
    if let Some(date_joined) = user.date_joined.clone() {
        let joined_date = chrono::NaiveDate::parse_from_str(&date_joined, "%Y-%m-%d").ok();
        if let Some(joined_date) = joined_date {
            let account_age_days = (chrono::Local::now().naive_local().date() - joined_date).num_days();
            // TODO: make the threshold configurable
            if account_age_days < 3 {
                if !advisories.iter().any(|a| a.id == "vrcmrd:account_age") {
                    advisories.push(ActiveAdvisory {
                        id: "vrcmrd:account_age".to_string(),
                        message: format!("User's account is {} days old.", account_age_days),
                        level: crate::types::advisories::AdvisoryLevel::Medium,
                        relevant_group_id: None,
                    });
                }
            }
        } else {
            eprintln!("Failed to parse date_joined for user {}: {}", user.id, user.date_joined.clone().unwrap());
        }
    }
    advisories
}