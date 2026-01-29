use std::sync::Mutex;

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
        // Clear the user list when joining a new instance
        let users_state = app.state::<Mutex<Users>>();
        let mut users_state = users_state.lock().unwrap();
        users_state.inner.clear();
        // Emit an event
        app.emit("vrcmrd:instance", instance_id.to_string())?;

        query_instance_info(app.clone(), &instance_id_info.clone());
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
                    handle.state::<InstanceStateMutex>().lock().unwrap().info = Some(instance_info);
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