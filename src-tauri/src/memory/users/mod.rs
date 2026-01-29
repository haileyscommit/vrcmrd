use std::{ops::Deref, sync::Mutex};

use tauri::{Emitter, Manager, Runtime};

use crate::types::VrcMrdUser;
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
