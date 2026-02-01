use tauri::{utils::config::WindowConfig, Manager, Runtime};

#[tauri::command]
pub async fn show_user_details<R: Runtime>(
    app: tauri::AppHandle<R>,
    app_window: tauri::Window<R>,
    user: String
) -> Result<(), String> {
    let window = tauri::WebviewWindowBuilder::from_config(
        app.app_handle(),
        &WindowConfig {
            title: format!("User Details - {}", user).into(),
            label: format!("user_details_{}", user),
            parent: Some(app_window.label().to_string()),
            url: tauri::WebviewUrl::App(format!("src/entrypoints/user.html#{}", user).into()),
            ..Default::default()
        },
    )
    .unwrap()
    .build()
    .unwrap();
    window.show().unwrap();
    Ok(())
}
