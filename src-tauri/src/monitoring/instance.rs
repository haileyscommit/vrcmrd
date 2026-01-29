use std::sync::Mutex;

use tauri::{AppHandle, Emitter, Manager};
use vrchatapi::models::instance;

use crate::{
    api::{VrchatApiMode, VrchatApiStateMutex}, memory::{instance::{InstanceState, InstanceStateMutex}, users::Users}, monitoring::VrcLogEntry, types::VrcMrdInstanceId
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
    if app.try_state::<VrchatApiStateMutex>().is_some() && app.state::<VrchatApiStateMutex>().try_lock().is_ok_and(|v| v.mode == VrchatApiMode::Ready) {
        println!("Fetching instance info for ID: {}", instance_id.to_string());
        let handle = app.clone();
        let state = handle.state::<VrchatApiStateMutex>();
        let guard = state.try_lock().unwrap();
        if let Some(config) = guard.config.as_ref() {
            let app_handle = app.clone();
            let config = config.clone();
            //let instance_id = instance_id.to_string();
            let instance_id = instance_id.clone();
            tauri::async_runtime::spawn(async move {
                match vrchatapi::apis::instances_api::get_instance(&config, &instance_id.world, &instance_id.id).await {
                    Ok(instance_info) => {
                        println!("Fetched instance info: {:?}", instance_info);
                        let _ = app_handle.emit("vrcmrd:instance_details", instance_info.clone());
                        app_handle.state::<InstanceStateMutex>().lock().unwrap().info = Some(instance_info);
                    }
                    Err(e) => {
                        eprintln!("Failed to fetch instance info: {:?}", e);
                    }
                }
            });
        }
    } else {
        eprintln!("API not ready, cannot fetch instance info for ID: {}", &instance_id.to_string());
    }
}