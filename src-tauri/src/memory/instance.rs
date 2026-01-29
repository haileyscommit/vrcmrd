use std::{ops::Deref, sync::Mutex};

use tauri::{Listener, Manager, Runtime, Wry};
use vrchatapi::models::Instance;

use crate::types::VrcMrdInstanceId;

#[derive(Clone, Default)]
pub struct InstanceState {
    pub id: Option<String>,
    pub id_info: Option<VrcMrdInstanceId>,
    pub info: Option<vrchatapi::models::Instance>,
}
pub type InstanceStateMutex = Mutex<InstanceState>;

pub fn instance_memory_plugin() -> tauri::plugin::TauriPlugin<Wry> {
    let mut listener: Option<u32> = None;
    tauri::plugin::Builder::new("instance_memory")
        .setup(move |app, _api| {
            let instance_state = InstanceStateMutex::new(InstanceState::default());
            app.manage::<InstanceStateMutex>(instance_state);
            let app_clone = app.app_handle().clone();
            listener = Some(app.listen("vrcmrd:cache_refresh", move |_| {
                let state = app_clone.state::<InstanceStateMutex>();
                let state = state.lock().unwrap();
                if state.id_info.is_none() {
                    return;
                }
                crate::monitoring::instance::query_instance_info(app_clone.clone(), state.id_info.as_ref().unwrap());
            }));
            Ok(())
        })
        .on_drop(move |app| {
            if let Some(listener_id) = listener {
                app.unlisten(listener_id);
            }
        })
        .build()
}

#[tauri::command]
pub async fn get_instance_id<R: Runtime>(
    app: tauri::AppHandle<R>,
    _window: tauri::Window<R>,
) -> Result<Option<String>, String> {
    match app.state::<InstanceStateMutex>().lock() {
        Ok(instance_id_mutex) => {
            let instance_id = instance_id_mutex.deref().id.clone();
            Ok(instance_id)
        }
        Err(e) => {
            eprintln!("Failed to lock instance ID mutex: {:?}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn get_instance_id_info<R: Runtime>(
    app: tauri::AppHandle<R>,
    _window: tauri::Window<R>,
) -> Result<Option<VrcMrdInstanceId>, String> {
    match app.state::<InstanceStateMutex>().lock() {
        Ok(instance_id_mutex) => {
            if let Some(ref instance_id_str) = instance_id_mutex.deref().id {
                let instance_info = VrcMrdInstanceId::from(instance_id_str);
                Ok(Some(instance_info))
            } else {
                Ok(None)
            }
        }
        Err(e) => {
            eprintln!("Failed to lock instance state mutex: {:?}", e);
            Err(e.to_string())
        }
    }
}

// Instance info doesn't benefit from long-term caching
#[tauri::command]
pub async fn get_instance_info<R: Runtime>(
    app: tauri::AppHandle<R>,
    _window: tauri::Window<R>,
) -> Result<Option<Instance>, String> {
    match app.state::<InstanceStateMutex>().lock() {
        Ok(instance_id_mutex) => {
            if let Some(ref instance_info) = instance_id_mutex.deref().info {
                Ok(Some(instance_info.clone()))
            } else {
                Ok(None)
            }
        }
        Err(e) => {
            eprintln!("Failed to lock instance state mutex: {:?}", e);
            Err(e.to_string())
        }
    }
}