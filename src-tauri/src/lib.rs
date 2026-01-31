#[cfg(target_os = "windows")]
use std::collections::HashMap;

use tauri::Manager;

mod api;
mod memory;
mod monitoring;
mod types;
mod window;
mod settings;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // TODO: set the title to reflect the current instance, group, or world
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_prevent_default::debug())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(monitoring::monitoring_plugin())
        .plugin(memory::users::user_memory_plugin())
        .plugin(memory::instance::instance_memory_plugin())
        .plugin(memory::users::avatar::avatar_memory_plugin())
        .plugin(api::vrchat_api_plugin())
        .invoke_handler(tauri::generate_handler![
            greet,
            memory::users::get_users,
            api::submit_2fa_token,
            api::cancel_login,
            window::user::show_user_details,
            window::show_settings_window,
            settings::update_config,
            settings::secret::update_credentials,
            api::logout,
        ])
        .setup(|app| {
            // let salt_path = app
            //     .path()
            //     .app_local_data_dir()
            //     .expect("could not resolve app local data path")
            //     .join("salt.txt");
            #[cfg(target_os = "windows")]
            keyring_core::set_default_store(windows_native_keyring_store::Store::new_with_configuration(&HashMap::from([
                ("prefix", "vrcmrd:".into()),
            ])).unwrap());
            #[cfg(target_os = "android")]
            keyring_core::set_default_store(android_native_keyring_store::Store::new().unwrap());
            // TODO: properly configure these other platforms
            //#[cfg(target_os = "macos")]
            //keyring_core::set_default_store(apple_native_keyring_store::Store::new().unwrap());
            //#[cfg(target_os = "linux")]
            //keyring_core::set_default_store(dbus_secret_service_keyring_store::Store::new().unwrap());
            //app.handle().plugin(tauri_plugin_stronghold::Builder::with_argon2(&salt_path).build())?;
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run_return(|_, e| {
            match e {
                tauri::RunEvent::ExitRequested { api, .. } => {
                    keyring_core::unset_default_store();
                }
                tauri::RunEvent::Exit => {
                    keyring_core::unset_default_store();
                }
                _ => {}
            }
        });
        //.expect("error while running tauri application");
}
