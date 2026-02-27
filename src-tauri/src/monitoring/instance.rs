use parking_lot::Mutex;

use tauri::{AppHandle, Emitter, Manager};

use crate::api::VrchatApiStateMutex;
use crate::{
    memory::{
        instance::{InstanceState, InstanceStateMutex},
        users::Users,
    },
    monitoring::VrcLogEntry,
    try_request,
    types::{user::CommonUser, VrcMrdInstanceId},
};

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
        let mut state = state.lock();
        state.info = None; // Clear instance info since we're in a new instance now
        state.id = Some(instance_id.to_string());
        // Parse the instance ID into its components, if possible
        let instance_id_info = VrcMrdInstanceId::from(instance_id);
        state.id_info = Some(instance_id_info.clone());
        // Mark instance as not settled
        state.settled = false;
        // Clear the notices list, since those are instance-specific
        {
            let advisory_memory = app.state::<Mutex<crate::memory::advisories::AdvisoryMemory>>();
            let mut advisory_memory = advisory_memory.lock();
            advisory_memory.notices.clear();
        }
        // Clear the user list when joining a new instance
        {
            let users_state = app.state::<Mutex<Users>>();
            let mut users_state = users_state.lock();
            users_state.inner.clear();
        }
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
                    let name = {
                        if let Some(display_name) = instance_info.clone().display_name.flatten() {
                            display_name
                        } else {
                            // TODO: get group name
                            instance_info.world.name.clone()
                        }
                    };
                    // Set window title to reflect instance name
                    let title = format!("VRCMRD - {}", name);
                    let _ = handle
                        .get_webview_window("main")
                        .map(|w| w.set_title(&title));
                    let _ = handle.emit("vrcmrd:instance_details", instance_info.clone());
                    let mut handled: Vec<String> = vec![];
                    handle.state::<InstanceStateMutex>().lock().info =
                        Some(instance_info.clone());
                    {
                        // Update users in list based on instance info
                        // NOTE: apparently this only works sometimes; the VRC API has conditions for when it
                        // includes this information. Therefore, this isn't very well tested, since I'm logging
                        // in to test the API with an account that isn't logged in to the game...
                        let users_state = handle.state::<Mutex<Users>>();
                        let mut users_state = users_state.lock();
                        for member in instance_info.clone().users.unwrap_or_default() {
                            for user in users_state.inner.iter_mut() {
                                if user.id == member.clone().id {
                                    // TODO: use the "system_probable_troll" field to add an advisory
                                    println!(
                                        "Updating user {} in user list based on instance info",
                                        &member.id
                                    );
                                    handled.push(member.id.clone());
                                    let updated_user = user.update_from(
                                        handle.clone(),
                                        &CommonUser::from(member.clone()),
                                        Vec::new(),
                                    );
                                    *user = updated_user.clone();
                                    break; // from inner loop
                                }
                            }
                        }
                        let _ = handle.emit("vrcmrd:users-updated", ());
                    }
                    // Now, for any users not handled, manually query their info
                    let users_state = handle.state::<Mutex<Users>>();
                    let users_state = users_state.lock();
                    for user in users_state.inner.iter() {
                        if !handled.contains(&user.id) {
                            if user.trust_rank.is_none() {
                                println!(
                                    "User {} not found in instance info, updating manually",
                                    &user.id
                                );
                                // Manually query user info for users not in instance info
                                let handle = handle.clone();
                                let user_id = user.id.clone();
                                tauri::async_runtime::spawn(async move {
                                    crate::api::user::query_user_info(handle, &user_id).await;
                                });
                            }
                        }
                    }
                }
                Ok(None) => {
                    eprintln!(
                        "API not ready, no instance info fetched for ID: {}",
                        &instance_id.to_string()
                    );
                }
                Err(e) => {
                    eprintln!("Failed to fetch instance info: {:?}", e);
                }
            };
        });
    } else {
        eprintln!(
            "API not ready, cannot fetch instance info for ID: {}",
            &instance_id.to_string()
        );
    }
}
