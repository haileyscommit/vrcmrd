use parking_lot::Mutex;

use tauri::{AppHandle, Manager};

use crate::{api::avatar_search::update_avatar, memory::users::Users, monitoring::VrcLogEntry};

pub fn handle_switched_avatar(app: AppHandle, line: &VrcLogEntry) -> Result<bool, tauri::Error> {
    // Determine if this is an instance join line
    let message = &line.message;
    if match_switching_avatar(message) {
        // Extract the username and avatar name from the log line
        // Example log line:
        // [Behaviour] Switching User name to avatar Avatar name
        let parts: Vec<&str> = message["[Behaviour] Switching ".len()..]
            .split(" to avatar ")
            .collect();
        if parts.len() == 2 {
            let username = parts[0].trim().to_string();
            let avatar_name = parts[1].trim().to_string();
            println!("User '{}' switched to avatar '{}'", username, avatar_name);
            // If user is not in user list yet, mark them as pending with this avatar name
            let users_state = app.state::<Mutex<Users>>();
            let users_state = users_state.lock();
            let user_exists = users_state
                .inner
                .iter()
                .any(|user| user.username == username);
            drop(users_state); // Release the lock early
            if !user_exists {
                println!(
                    "User '{}' not found in user list yet, marking avatar '{}' as pending",
                    username, avatar_name
                );
                let avatars_state = app.state::<crate::memory::users::avatar::AvatarsStateMutex>();
                let mut avatars_state = avatars_state.lock();
                (*avatars_state)
                    .pending_avatar_names
                    .push((username.clone(), avatar_name.clone()));
            } else {
                // User exists, lookup and update avatar data
                {
                    // Set avatar name immediately
                    let users_state = app.state::<Mutex<Users>>();
                    let mut users_state = users_state.lock();
                    if let Some(user) = users_state
                        .inner
                        .iter_mut()
                        .find(|user| user.username == username) {
                        user.avatar_name = avatar_name.clone();
                    } else {
                        eprintln!("User '{}' not found in user list after initial check", username);
                        return Ok(false);
                    }
                }
                // Do network lookups and other stuff
                let users_state = app.state::<Mutex<Users>>();
                let users_state = users_state.lock();
                let user = users_state
                    .inner
                    .iter()
                    .find(|user| user.username == username)
                    .unwrap()
                    .clone();
                let instance_state = app.state::<crate::memory::instance::InstanceStateMutex>();
                let instance_state = instance_state.lock();
                if users_state.joined_before_settled.contains(&user.id) || !instance_state.settled {
                    #[cfg(debug_assertions)]
                    eprintln!("User '{}' switched avatar before instance settled, not attempting avatar search", username);
                } else {
                    drop(users_state); // Release the lock before doing async work
                    drop(instance_state);
                    update_avatar(user.clone(), app.clone());
                }
                // if let Some(user) = found_user {
                //     // Emit an event
                //     app.emit("vrcmrd:update-user", user)?;
                // }
            }
            return Ok(true);
        }
    }
    Ok(false)
}

fn match_switching_avatar(message: &str) -> bool {
    message.starts_with("[Behaviour] Switching ") && message.contains(" to avatar ")
}
