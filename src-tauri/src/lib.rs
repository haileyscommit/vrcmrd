mod monitoring;
mod types;
mod memory;
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
        .plugin(monitoring::monitoring_plugin())
        .plugin(memory::users::user_memory_plugin())
        .plugin(memory::instance::instance_id_memory_plugin())
        .invoke_handler(tauri::generate_handler![
            greet,
            memory::users::get_users
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
