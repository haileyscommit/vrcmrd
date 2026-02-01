use std::sync::Mutex;

use tauri::{AppHandle, Emitter, Manager};

use crate::{memory::users::Users, monitoring::VrcLogEntry};

pub fn handle_switched_avatar(app: AppHandle, line: &VrcLogEntry) -> Result<bool, tauri::Error> {
    // Determine if this is an instance join line
    let message = &line.message;
    if match_switching_avatar(message) {
        // Extract the username and avatar name from the log line
        // Example log line:
        // [Behaviour] Switching User name to avatar Avatar name
        let parts: Vec<&str> = message["[Behaviour] Switching ".len()..].split(" to avatar ").collect();
        if parts.len() == 2 {
            let username = parts[0].trim().to_string();
            let avatar_name = parts[1].trim().to_string();
            println!("User '{}' switched to avatar '{}'", username, avatar_name);
            // If user is not in user list yet, mark them as pending with this avatar name
            let users_state = app.state::<Mutex<Users>>();
            let users_state = users_state.lock().unwrap();
            let user_exists = users_state.inner.iter().any(|user| user.username == username);
            drop(users_state); // Release the lock early
            if !user_exists {
                println!("User '{}' not found in user list yet, marking avatar '{}' as pending", username, avatar_name);
                let avatars_state = app.state::<crate::memory::users::avatar::AvatarsStateMutex>();
                let mut avatars_state = avatars_state.lock().unwrap();
                (*avatars_state).pending_avatar_names.push((username.clone(), avatar_name.clone()));
            } else {
                //println!("User '{}' found in user list, updating avatar to '{}'", username, avatar_name);
                // Update the user's avatar name directly
                let users_state = app.state::<Mutex<Users>>();
                let mut users_state = users_state.lock().unwrap();
                let mut found_user = Option::None;
                for user in users_state.inner.iter_mut() {
                    if user.username == username {
                        user.avatar_name = avatar_name.clone();
                        found_user = Some(user.clone());
                        break;
                    }
                }
                drop(users_state); // Release the lock early
                // Emit an event
                if let Some(user) = found_user {
                    app.emit("vrcmrd:update-user", user)?;
                }
            }
            return Ok(true);
        }
    }
    Ok(false)
}

fn match_switching_avatar(message: &str) -> bool {
    message.starts_with("[Behaviour] Switching ") && message.contains(" to avatar ")
}
