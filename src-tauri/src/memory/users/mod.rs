use std::{ops::Deref, sync::Mutex};

use tauri::{Manager, Runtime, Wry};
use vrchatapi::models::LimitedUserInstance;

use crate::{try_request, types::{VrcMrdUser, user::CommonUser}};
pub mod avatar;

#[derive(Default)]
pub struct Users {
    pub inner: Vec<VrcMrdUser>,
}

pub fn user_memory_plugin<R: Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("users_memory")
        .setup(|app, _api| {
            let users = Mutex::new(Users::default());
            app.manage(users);
            Ok(())
        })
        .build()
}

#[tauri::command]
pub async fn get_users<R: Runtime>(
    app: tauri::AppHandle<R>,
    _window: tauri::Window<R>,
) -> Result<Vec<VrcMrdUser>, String> {
    match app.state::<Mutex<Users>>().lock() {
        Ok(users_mutex) => {
            let users = users_mutex.deref().inner.clone();
            Ok(users)
        }
        Err(e) => {
            eprintln!("Failed to lock users mutex: {:?}", e);
            Err(e.to_string())
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct GetUserInfoResponse {
    pub local: Option<VrcMrdUser>,
    pub remote: Option<LimitedUserInstance>,
}

#[tauri::command]
pub async fn get_user_info(
    app: tauri::AppHandle<Wry>,
    user_id: &str,
) -> Result<GetUserInfoResponse, String> {
    let response = try_request!(app.clone(), |config| {
        vrchatapi::apis::users_api::get_user(config, user_id)
    }, { wait_for_api_ready: true }).await;
    match response {
        Ok(Some(user_info)) => {
            let base_user = {
                let users_state = app.state::<Mutex<Users>>();
                let users_state = users_state.lock().unwrap();
                users_state.inner.iter().find(|u| u.id == user_info.id).cloned()
            };
            let mut local_user = base_user;
            let remote_user = Some(user_info);
            {
                let users_state = app.state::<Mutex<Users>>();
                let mut users_state = users_state.lock().unwrap();
                if let Some(existing_user) = users_state.inner.iter_mut().find(|u| u.id == user_id) {
                    // Update existing user
                    if let Some(ref remote) = remote_user {
                        existing_user.update_from(app.clone(), &Into::<CommonUser>::into(remote.clone()), Vec::new());
                        local_user = Some(existing_user.clone());
                    }
                }
            }
            Ok(GetUserInfoResponse {
                local: local_user,
                remote: remote_user.map(CommonUser::from).map(|u| u.into()),
            })
        }
        Ok(None) => {
            Err(format!("User not found for ID: {}", user_id))
        }
        Err(e) => {
            Err(format!("Error fetching user info for ID {}: {:?}", user_id, e))
        }
    }
}
            
