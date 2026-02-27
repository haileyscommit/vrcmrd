#[cfg(target_os = "windows")]
use std::collections::HashMap;

use tauri::Listener;

use crate::api::xsoverlay::XSOVERLAY_SOCKET;

mod advisories;
mod api;
mod memory;
mod monitoring;
mod notices;
mod settings;
mod types;
mod window;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // TODO: set the title to reflect the current instance, group, or world
    tauri::Builder::default()
        .plugin(tauri_plugin_tts::init())
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
            keyring_core::set_default_store(
                windows_native_keyring_store::Store::new_with_configuration(&HashMap::from([(
                    "prefix",
                    "vrcmrd:".into(),
                )]))
                .unwrap(),
            );
            #[cfg(target_os = "android")]
            keyring_core::set_default_store(android_native_keyring_store::Store::new().unwrap());
            // TODO: properly configure these other platforms
            //#[cfg(target_os = "macos")]
            //keyring_core::set_default_store(apple_native_keyring_store::Store::new().unwrap());
            //#[cfg(target_os = "linux")]
            //keyring_core::set_default_store(dbus_secret_service_keyring_store::Store::new().unwrap());
            //app.handle().plugin(tauri_plugin_stronghold::Builder::with_argon2(&salt_path).build())?;

            // XSOverlay WebSocket connection setup
            let appclone = app.handle().clone();
            let stophandle = tauri::async_runtime::spawn(async move {
                let _ = api::xsoverlay::start_xsoverlay_socket(appclone.clone()).await;
                println!("XSOverlay initial socket connection quit.");
            });
            let stophandle = std::sync::Arc::new(parking_lot::Mutex::new(stophandle));
            let appclone = app.handle().clone();
            let stophandle_clone = stophandle.clone();
            let xso_refresh_handler = move |_| {
                // This soft-refresh handler restarts the XSOverlay connection if it needs to
                #[cfg(debug_assertions)]
                println!("Received refresh event, checking XSOverlay connection...");
                let appclone = appclone.clone();
                let socket = XSOVERLAY_SOCKET.try_lock_for(std::time::Duration::from_secs(1));
                if socket.is_none() || socket.as_ref().unwrap().is_none() {
                    if stophandle_clone.lock().inner().is_finished() {
                        println!("XSOverlay socket connection is not active, restarting...");
                    } else if socket.is_none() {
                        println!("Could not acquire lock on XSOverlay socket, it may be stuck! Not restarting for now...");
                        return;
                    } else {
                        #[cfg(debug_assertions)]
                        println!("XSOverlay socket is still connected.");
                        return;
                    }
                    // If we can't acquire the lock or the socket is not available, try to restart the socket connection
                    let mut handle = stophandle_clone.lock();
                    println!("Restarting XSOverlay socket connection...");
                    handle.abort();
                    *handle = tauri::async_runtime::spawn(async move {
                        let _ = api::xsoverlay::start_xsoverlay_socket(appclone.clone()).await;
                        println!("XSOverlay socket connection quit.");
                    });
                } else {
                    #[cfg(debug_assertions)]
                    println!("XSOverlay socket connection is healthy, no need to restart.");
                }
            };
            app.listen_any("vrcmrd:cache_refresh", xso_refresh_handler.clone());
            app.listen_any("vrcmrd:settled", xso_refresh_handler.clone());
            
            // Deadlock detection
            #[cfg(debug_assertions)]
            tauri::async_runtime::spawn(async {
                loop {
                    tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                    for deadlock in parking_lot::deadlock::check_deadlock() {
                        println!("Deadlock detected:");
                        for thread in deadlock {
                            println!("Thread Id {:#?}", thread.thread_id());
                            println!("{:#?}", thread.backtrace());
                        }
                    }
                };
            });
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run_return(|_, e| match e {
            tauri::RunEvent::ExitRequested { .. } => {
                keyring_core::unset_default_store();
            }
            tauri::RunEvent::Exit => {
                keyring_core::unset_default_store();
            }
            _ => {}
        });
    //.expect("error while running tauri application");
}
