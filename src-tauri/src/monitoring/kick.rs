// [ModerationManager] Juiceworld17 2117 has been kicked
// (where Juiceworld17 2117 is the username)

use parking_lot::Mutex;
use tauri::{AppHandle, Manager};

use crate::{memory::users::Users, monitoring::VrcLogEntry};

pub fn handle_kick(app: AppHandle, line: &VrcLogEntry) -> Result<bool, tauri::Error> {
    let message = &line.message;
    if let Some(rest) = message.strip_prefix("[ModerationManager] ") {
        // TODO: also support "has been banned"?
        if let Some(kick_info) = rest.strip_suffix(" has been kicked") {
            let username = kick_info.trim();
            println!("User kicked: {}", username);
            // Mark the user as recently kicked in the memory
            let state = app.state::<Mutex<Users>>();
            let mut state = state.lock();
            if let Some(user) = state.inner.iter_mut().find(|u| u.username == username) {
                user.recently_kicked = true;
                println!("Marked user '{}' as recently kicked", username);
            } else {
                println!("Could not find user '{}' to mark as recently kicked", username);
            }
            return Ok(true);
        } else if let Some(ban_info) = rest.strip_suffix(" has been banned") {
            // NOTE: this is untested because I haven't personally banned anyone lol
            let username = ban_info.trim();
            println!("User banned: {}", username);
            // Mark the user as recently kicked in the memory
            let state = app.state::<Mutex<Users>>();
            let mut state = state.lock();
            if let Some(user) = state.inner.iter_mut().find(|u| u.username == username) {
                user.recently_kicked = true;
                println!("Marked user '{}' as recently banned", username);
            } else {
                println!("Could not find user '{}' to mark as recently banned", username);
            }
            return Ok(true);
        }
    }
    Ok(false)
}