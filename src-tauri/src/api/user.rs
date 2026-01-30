use std::sync::Mutex;

use chrono::NaiveDate;
use tauri::{AppHandle, Emitter, Manager, Runtime, State};

use crate::{api::user, memory::users::Users, types::VrcMrdUser};

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
            println!("Received user info for {}: {:?}", user_id, user_info);
            // TODO: introduce advisory for account age
            let vrcmrd_user = VrcMrdUser {
                age_verified: user_info.clone().age_verified,
                account_created: NaiveDate::parse_from_str(&user_info.date_joined, "%Y-%m-%d").ok().and_then(|d| d.and_hms_opt(0, 0, 0).and_then(|r| Some(r.and_utc().timestamp()))),
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