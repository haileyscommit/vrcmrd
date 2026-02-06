#[cfg(target_os = "windows")]
use std::collections::HashMap;


mod api;
mod memory;
mod monitoring;
mod types;
mod window;
mod settings;
mod advisories;
mod notices;

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
        .plugin(memory::advisories::advisory_memory_plugin())
        .plugin(api::vrchat_api_plugin())
        .invoke_handler(tauri::generate_handler![
            greet,
            memory::users::get_users,
            api::submit_2fa_token,
            api::cancel_login,
            memory::instance::get_instance_id,
            memory::instance::get_instance_id_info,
            memory::instance::get_instance_info,
            window::user::show_user_details,
            memory::users::get_user_info,
            api::groups::get_all_groups,
            window::show_settings_window,
            window::show_advisories_window,
            settings::update_config,
            settings::get_config,
            settings::secret::update_credentials,
            api::logout,
            // Advisories CRUD
            advisories::generate_advisory_id,
            advisories::add_advisory,
            advisories::get_advisories,
            advisories::get_advisory,
            advisories::update_advisory,
            advisories::remove_advisory,
            notices::get_all_notices,
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
            let appclone = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let _ = api::xsoverlay::start_xsoverlay_socket(appclone.clone()).await;
            });
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run_return(|_, e| {
            match e {
                tauri::RunEvent::ExitRequested {  .. } => {
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
