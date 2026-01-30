use std::sync::Mutex;

use crate::api::user::thread_query_user_info;
use crate::memory::users::Users;
use tauri::{AppHandle, Emitter, Manager};

use crate::monitoring::VrcLogEntry;
use crate::types::VrcMrdUser;

/// Handle a join/leave log entry, emitting Tauri events as appropriate.
/// If the line was a join/leave event, emits the event and returns Ok(true).
/// If not, returns Ok(false). If an error occurs during emitting, returns Err.
pub fn handle_join_leave(app: AppHandle, line: &VrcLogEntry) -> Result<bool, tauri::Error> {
    let message = &line.message;
    if let Some(rest) = message.strip_prefix("[Behaviour] OnPlayerJoined ") {
        // rest is like "Player Name (usr_xxx)"
        if let (Some(open), Some(close)) = (rest.rfind('('), rest.rfind(')')) {
            if open < close {
                let player_name = rest[..open].trim().to_string();
                let player_id = rest[open + 1..close].to_string();
                //println!("Player joined: {} ({})", player_name, player_id);
                let avatar_name: Option<String> = {
                    let avatars_state = app.state::<crate::memory::users::avatar::AvatarsStateMutex>();
                    let mut avatars_state = avatars_state.lock().unwrap();
                    // Check if there's a pending avatar name for this user
                    if let Some(index) = avatars_state.pendingAvatarNames.iter().position(|(username, _)| username == &player_name) {
                        let (_, avatar_name) = avatars_state.pendingAvatarNames.remove(index);
                        println!("Found pending avatar name '{}' for joining user '{}'", avatar_name, player_name);
                        Some(avatar_name)
                    } else {
                        None
                    }
                };
                let user = VrcMrdUser {
                    id: player_id,
                    username: player_name,
                    avatar_name: avatar_name.unwrap_or_default(),
                    perf_rank: "VeryPoor".to_string(), // TODO: determine actual perf rank
                    account_created: None,
                    join_time: parse_timestamp(&line.timestamp),
                    leave_time: None,
                    advisories: false, // this should contain the actual advisories
                    age_verified: false,
                    platform: None,
                };
                let state = app.state::<Mutex<Users>>();
                let mut state = state.lock().unwrap();
                state.inner.retain(|e| e.id != user.id);
                state.inner.push(user.clone());
                // TODO: figure out when the instance is "settled" and up to date, and only do this then.
                // When the instance is "settled", the instance information contains the full user list.
                if let Some(instance_state) = app.try_state::<crate::memory::instance::InstanceStateMutex>() {
                    let instance_state = instance_state.lock().unwrap();
                    if instance_state.settled {
                        drop(instance_state); // Release the lock early
                        thread_query_user_info(app.clone(), &user.id);
                    }
                }
                return app.emit("vrcmrd:join", user).and(Ok(true));
            }
        }
    } else if let Some(rest) = message.strip_prefix("[Behaviour] OnPlayerLeft ") {
        // rest is like "Player Name (usr_xxx)"
        if let (Some(open), Some(close)) = (rest.rfind('('), rest.rfind(')')) {
            if open < close {
                let player_name = rest[..open].trim().to_string();
                let player_id = rest[open + 1..close].to_string();
                //println!("Player left: {} ({})", player_name, player_id);
                let user = VrcMrdUser {
                    id: player_id,
                    username: player_name,
                    avatar_name: String::new(),
                    perf_rank: "VeryPoor".to_string(),
                    account_created: None,
                    join_time: 0, // TODO: store it as a unix timestamp and format on frontend
                    leave_time: Some(parse_timestamp(&line.timestamp)),
                    advisories: false, // this should contain the actual advisories
                    age_verified: false,
                    platform: None,
                };
                let state = app.state::<Mutex<Users>>();
                let mut state = state.lock().unwrap();
                // Update the user's leave_time if they exist
                if let Some(existing_user) = state.inner.iter_mut().find(|u| u.id == user.id) {
                    existing_user.leave_time = user.leave_time.clone();
                } else {
                    // If user not found, add them (this shouldn't normally happen)
                    state.inner.push(user.clone());
                }
                return app.emit("vrcmrd:leave", user).and(Ok(true));
            }
        }
    }
    Ok(false)
}

/// Parse a log timestamp string (2026.01.27 16:24:32) from the log into a Unix timestamp representation.
pub fn parse_timestamp(_timestamp_str: &str) -> i64 {
    let _dt = chrono::NaiveDateTime::parse_from_str(_timestamp_str, "%Y.%m.%d %H:%M:%S");
    if let Some(dt) = _dt.ok() {
        let unix_timestamp = dt.and_local_timezone(chrono::Local).unwrap().timestamp() as i64;
        return unix_timestamp;
    } else {
        return 0;
    }
}
