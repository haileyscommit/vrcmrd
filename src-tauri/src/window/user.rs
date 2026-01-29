use tauri::{utils::config::WindowConfig, Manager, Runtime};

use crate::api::VrchatApiStateMutex;

#[tauri::command]
pub async fn show_user_details<R: Runtime>(
    app: tauri::AppHandle<R>,
    appWindow: tauri::Window<R>,
    user: String,
    api_config: tauri::State<'_, VrchatApiStateMutex>,
) -> Result<(), String> {
    let window = tauri::WebviewWindowBuilder::from_config(
        app.app_handle(),
        &WindowConfig {
            title: "User Details".into(), // TODO: include user name
            label: format!("user_details_{}", user),
            parent: Some(appWindow.label().to_string()),
            ..Default::default()
        },
    )
    .unwrap()
    .build()
    .unwrap();
    window.show().unwrap();
    Ok(())
}
