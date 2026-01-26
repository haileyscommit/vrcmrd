use std::{ops::Deref, sync::Mutex};

use tauri::{Emitter, Manager, Runtime};

use crate::types::VrcMrdUser;

#[derive(Default)]
pub struct InstanceIDState {
    pub inner: Option<String>,
}

pub fn instance_id_memory_plugin<R: Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("instance_id_memory")
        .setup(|app, _api| {
            let instance_id_state = Mutex::new(InstanceIDState::default());
            app.manage(instance_id_state);
            Ok(())
        })
        .build()
}

#[tauri::command]
pub async fn get_instance_id<R: Runtime>(app: tauri::AppHandle<R>, _window: tauri::Window<R>) -> Result<Option<String>, String> {
  match app.state::<Mutex<InstanceIDState>>().lock() {
    Ok(instance_id_mutex) => {
      let instance_id = instance_id_mutex.deref().inner.clone();
      Ok(instance_id)
    },
    Err(e) => {
      eprintln!("Failed to lock instance ID mutex: {:?}", e);
      Err(e.to_string())
    }
  }
}