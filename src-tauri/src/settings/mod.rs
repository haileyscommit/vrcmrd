use tauri::{Emitter, Runtime};
use tauri_plugin_store::StoreExt;

pub mod secret;

#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct ConfigEntry {
    key: String,
    value: String,
}

#[tauri::command]
pub async fn update_config<R: Runtime>(
    app: tauri::AppHandle<R>,
    key: String,
    value: String,
) -> Result<(), String> {
    match app.store("vrcmrd-config.json") {
        Ok(store) => {
            store.set(&key, serde_json::Value::String(value.clone()));
            match store.save() {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Failed to save store: {}", e);
                    return Err(e.to_string());
                }
            }
            app.emit("vrcmrd:config_updated", ConfigEntry { key, value })
                .map_err(|e| e.to_string())?;
        }
        Err(e) => {
            eprintln!("Failed to access store: {}", e);
            return Err(e.to_string());
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn get_config<R: Runtime>(
    app: tauri::AppHandle<R>,
    key: String,
) -> Result<Option<String>, String> {
    // TODO: cache config in memory to avoid repeated store access
    match app.store("vrcmrd-config.json") {
        Ok(store) => match store.get(&key) {
            Some(value) => {
                if let Some(s) = value.as_str() {
                    Ok(Some(s.to_string()))
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
        },
        Err(e) => {
            eprintln!("Failed to access store: {}", e);
            Err(e.to_string())
        }
    }
}
