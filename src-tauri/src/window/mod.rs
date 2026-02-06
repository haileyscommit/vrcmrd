use std::str::FromStr;

use tauri::{Emitter, Listener, Manager, Runtime};

pub mod user;

pub async fn show_window_by_label<R: Runtime>(
    app: tauri::AppHandle<R>,
    window: Option<tauri::Window<R>>,
    label: &str,
) -> Result<(), String> {
    let winconfig = app
        .config()
        .app
        .windows
        .iter()
        .find(|w| w.label == label)
        .cloned();
    if let Some(mut cfg) = winconfig {
        if let Some(window) = window {
            cfg.parent = Some(window.label().to_string());
        }
        if let Some(window) = app.get_webview_window(label) {
            window.show().unwrap();
            window.set_focus().unwrap();
            return Ok(());
        }
        let window = tauri::WebviewWindowBuilder::from_config(app.app_handle(), &cfg)
            .unwrap()
            .build()
            .unwrap();
        window.show().unwrap();
        window.set_focus().unwrap();
        Ok(())
    } else {
        return Err("could not find window configuration".into());
    }
}

#[tauri::command]
pub async fn show_settings_window<R: Runtime>(
    app: tauri::AppHandle<R>,
    window: tauri::Window<R>,
) -> Result<(), String> {
    show_window_by_label(app, Some(window), "settings").await
}
#[tauri::command]
pub async fn show_advisories_window<R: Runtime>(
    app: tauri::AppHandle<R>,
    window: tauri::Window<R>,
    advisory: Option<String>,
) -> Result<(), String> {
    show_window_by_label(app.clone(), Some(window), "advisories").await?;
    let advisories_window = app
        .get_webview_window("advisories")
        .ok_or("could not get advisories window")?;
    let winclone = advisories_window.clone();
    advisories_window.once("ready", move |_| {
        if let Some(advisory_id) = advisory {
            winclone.emit("open-advisory", advisory_id).unwrap();
        }
    });
    Ok(())
}
