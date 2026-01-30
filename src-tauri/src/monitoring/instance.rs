use std::sync::Mutex;

use chrono::NaiveDate;
use tauri::{AppHandle, Emitter, Manager};

use crate::{
    memory::{instance::{InstanceState, InstanceStateMutex}, users::Users}, monitoring::VrcLogEntry, try_request, types::VrcMrdInstanceId
};
use crate::api::VrchatApiStateMutex;

pub fn handle_joined_instance(app: AppHandle, line: &VrcLogEntry) -> Result<bool, tauri::Error> {
    // Determine if this is an instance join line
    let message = &line.message;
    if message.starts_with("[Behaviour] Joining wrld_") {
        let rest = &message["[Behaviour] Joining ".len()..];
        // Extract the instance ID from the log line
        // TODO: this may change to an inst_ UUID in the future
        let instance_id = rest;
        println!("Joined instance: {}", instance_id);
        let state = app.state::<Mutex<InstanceState>>();
        let mut state = state.lock().unwrap();
        state.id = Some(instance_id.to_string());
        // Parse the instance ID into its components, if possible
        let instance_id_info = VrcMrdInstanceId::from(instance_id);
        state.id_info = Some(instance_id_info.clone());
        // Mark instance as not settled
        state.settled = false;
        // Clear the user list when joining a new instance
        let users_state = app.state::<Mutex<Users>>();
        let mut users_state = users_state.lock().unwrap();
        users_state.inner.clear();
        // Emit an event
        app.emit("vrcmrd:instance", instance_id.to_string())?;
        return Ok(true);
    }
    Ok(false)
}

/// Start thread to get instance details from API
pub fn query_instance_info(app: AppHandle, instance_id: &VrcMrdInstanceId) {
    if app.try_state::<VrchatApiStateMutex>().is_some() {
        println!("Fetching instance info for ID: {}", instance_id.to_string());
        //let app_handle = app.clone();
        //let config = config.clone();
        //let instance_id = instance_id.to_string();
        let instance_id = instance_id.clone();
        tauri::async_runtime::spawn(async move {
            let handle = app.clone();
            let response = try_request!(handle, |config| {
                vrchatapi::apis::instances_api::get_instance(config, &instance_id.world, &instance_id.id)
            }, { wait_for_api_ready: true }).await;
            // match response...
            match response {
                Ok(Some(instance_info)) => {
                    let handle = app.clone();
                    //println!("Fetched instance info: {:?}", instance_info);
                    println!("Received instance info for {}", &instance_info.id);
                    let _ = handle.emit("vrcmrd:instance_details", instance_info.clone());
                    let name = {
                        if let Some(display_name) = instance_info.clone().display_name.flatten() {
                            display_name
                        } else {
                            // TODO: get group name
                            instance_info.world.name.clone()
                        }
                    };
                    let mut handled: Vec<String> = vec![];
                    handle.state::<InstanceStateMutex>().lock().unwrap().info = Some(instance_info.clone());
                    {
                        // Update users in list based on instance info
                        let users_state = handle.state::<Mutex<Users>>();
                        let mut users_state = users_state.lock().unwrap();
                        for member in instance_info.clone().users.unwrap_or_default() {
                            for user in users_state.inner.iter_mut() {
                                if user.id == member.clone().id {
                                    println!("Updating user {} in user list based on instance info", &member.id);
                                    &handled.push(member.id.clone());
                                    user.platform = {
                                        let platform = member.clone().last_platform;
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
                                    user.age_verified = member.clone().age_verified;
                                    if let Some(date_joined) = member.clone().date_joined {
                                        user.account_created = NaiveDate::parse_from_str(&date_joined, "%Y-%m-%d").ok().and_then(|d| d.and_hms_opt(0, 0, 0).and_then(|r| Some(r.and_utc().timestamp())));
                                    }
                                    break; // from inner loop
                                }
                            };
                        }
                        let _ = handle.emit("vrcmrd:users-updated", ());
                    }
                    // Now, for any users not handled, manually query their info
                    let users_state = handle.state::<Mutex<Users>>();
                    let users_state = users_state.lock().unwrap();
                    for user in users_state.inner.iter() {
                        if !handled.contains(&user.id) {
                            println!("User {} not found in instance info, updating manually", &user.id);
                            // Manually query user info for users not in instance info
                            let handle = handle.clone();
                            let user_id = user.id.clone();
                            tauri::async_runtime::spawn(async move {
                                crate::api::user::query_user_info(handle, &user_id).await;
                            });
                        }
                    }
                    // Set window title to reflect instance name
                    let title = format!("VRCMRD - {}", name);
                    let _ = handle.get_webview_window("main").map(|w| w.set_title(&title));
                }
                Ok(None) => {
                    eprintln!("API not ready, no instance info fetched for ID: {}", &instance_id.to_string());
                }
                Err(e) => {
                    eprintln!("Failed to fetch instance info: {:?}", e);
                }
            };
        });
    } else {
        eprintln!("API not ready, cannot fetch instance info for ID: {}", &instance_id.to_string());
    }
}