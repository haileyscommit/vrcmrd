use std::sync::Mutex;

use tauri::{Listener, Manager, Wry};

pub struct AvatarsState {
    pub pendingAvatarNames: Vec<(String, String)>, // (username, avatar name)
}
pub type AvatarsStateMutex = Mutex<AvatarsState>;

pub fn avatar_memory_plugin() -> tauri::plugin::TauriPlugin<Wry> {
    let mut listener: Option<u32> = None;
    let mut listener2: Option<u32> = None;
    tauri::plugin::Builder::new("avatars_memory")
        .setup(move |app, _api| {
            let state = AvatarsState {
                pendingAvatarNames: Vec::new(),
            };
            app.manage::<AvatarsStateMutex>(Mutex::new(state));
            // listener = Some(app.listen("vrcmrd:", |event| {
            //     let instance_id = event.payload().to_string();
            //     //vrchatapi::apis::instances_api::get_instance_by_short_name(&Configuration::default(), &instance_id).await;
            //     // TODO: look up, cache, and emit instance info here
            // }));
            // let app_clone = app.app_handle().clone();
            // listener2 = Some(app.listen("vrcmrd:cache_refresh", move |_| {
            //     let state = app_clone.state::<AvatarsStateMutex>();
            //     let state = state.lock().unwrap();
            //     if state.id_info.is_none() {
            //         return;
            //     }
            //     crate::monitoring::instance::query_instance_info(app_clone.clone(), state.id_info.as_ref().unwrap());
            // }));
            Ok(())
        })
        // .on_drop(move |app| {
        //     if let Some(listener_id) = listener {
        //         app.unlisten(listener_id);
        //     }
        //     if let Some(listener_id) = listener2 {
        //         app.unlisten(listener_id);
        //     }
        // })
        .build()
}