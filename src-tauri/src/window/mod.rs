use tauri::{Manager, Runtime};

pub mod user;

#[tauri::command]
pub async fn show_settings_window<R: Runtime>(app: tauri::AppHandle<R>, window: tauri::Window<R>) -> Result<(), String> {
    let winconfig = app.config().app.windows.iter().find(|w| w.label == "settings").cloned();
    if let Some(mut cfg) = winconfig {
        cfg.parent = Some(window.label().to_string());
        if let Some(window) = app.get_webview_window("settings") {
            //window.parent = Some(window.label().to_string());
            //window.dialog().unwrap();
            window.show().unwrap();
            window.set_focus().unwrap();
            return Ok(());
        }
        let window = tauri::WebviewWindowBuilder::from_config(
            app.app_handle(),
            &cfg,
        )
        .unwrap()
        .build()
        .unwrap();
        window.show().unwrap();
        window.set_focus().unwrap();
        Ok(())
    } else {
        return Err("could not find settings window configuration".into());
    }
}