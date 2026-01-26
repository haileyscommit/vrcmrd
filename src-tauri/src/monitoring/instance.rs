use std::sync::Mutex;

use tauri::{AppHandle, Emitter, Manager};

use crate::{memory::{instance::InstanceIDState, users::Users}, monitoring::VrcLogEntry};

pub fn handle_joined_instance(app: AppHandle, line: &VrcLogEntry) -> Result<bool, tauri::Error> {
    // Determine if this is an instance join line
    let message = &line.message;
    if let Some(rest) = message.strip_prefix("[Behaviour] Joined wrld_") {
        // Extract the instance ID from the log line
        if let Some(space_pos) = rest.find("wrld_") {
            let instance_id = &rest[..space_pos];
            println!("Joined instance: {}", instance_id);
            let state = app.state::<Mutex<InstanceIDState>>();
            let mut state = state.lock().unwrap();
            state.inner = Some(instance_id.to_string());
            // Clear the user list when joining a new instance
            let users_state = app.state::<Mutex<Users>>();
            let mut users_state = users_state.lock().unwrap();
            users_state.inner.clear();
            // Emit an event
            app.emit("vrcmrd:instance", instance_id.to_string())?;
            return Ok(true);
        }
    }
    Ok(false)
}