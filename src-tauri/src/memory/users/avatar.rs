use parking_lot::Mutex;

use tauri::{Manager, Wry};

pub struct AvatarsState {
    pub pending_avatar_names: Vec<(String, String)>, // (username, avatar name)
}
pub type AvatarsStateMutex = Mutex<AvatarsState>;

pub fn avatar_memory_plugin() -> tauri::plugin::TauriPlugin<Wry> {
    //let mut listener: Option<u32> = None;
    //let mut listener2: Option<u32> = None;
    tauri::plugin::Builder::new("avatars_memory")
        .setup(move |app, _api| {
            let state = AvatarsState {
                pending_avatar_names: Vec::new(),
            };
            app.manage::<AvatarsStateMutex>(Mutex::new(state));
            // TODO: watch for vrcmrd:cache_refresh to clear the avatar info cache
            Ok(())
        })
        .build()
}
