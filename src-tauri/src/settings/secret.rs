use tauri::{Manager, Wry};

use crate::api::{VRCHAT_API_PASSWORD_CREDENTIAL_KEY, VRCHAT_API_USERNAME_CREDENTIAL_KEY, VRCHAT_AUTH_COOKIE_CREDENTIAL_KEY, VrchatApiStateMutex, initialize_api};

#[tauri::command]
pub async fn update_credentials(app: tauri::AppHandle<Wry>, username: String, password: String) -> Result<(), String> {
    let user_entry = keyring_core::Entry::new(VRCHAT_API_USERNAME_CREDENTIAL_KEY, "default");
    if let Ok(u) = user_entry {
        // Unready the API
        println!("Setting API to NotReady state to update credentials");
        let api = app.state::<VrchatApiStateMutex>();
        let mut api_lock = api.lock().await;
        api_lock.mode = crate::api::VrchatApiMode::NotReady;
        api_lock.cookies = None;
        api_lock.config = None;
        drop(api_lock);
        // Username exists; clear its API key
        match u.get_password() {
            Ok(username) => {
                keyring_core::Entry::new(VRCHAT_AUTH_COOKIE_CREDENTIAL_KEY, &username)
                    .and_then(|entry| entry.delete_credential())
                    .map_err(|e| e.to_string())?;
            }
            Err(keyring_core::Error::NoEntry) => {
                // No existing entry; nothing to delete
            }
            Err(e) => {
                eprintln!("Failed to access existing username entry: {}", e);
                return Err(e.to_string());
            }
        };
        u.set_password(&username).map_err(|e| e.to_string())?;
    } else {
        eprintln!("Failed to create username entry: {}", user_entry.err().unwrap());
        return Err("Failed to create username entry".into());
    };
    let password_entry = keyring_core::Entry::new(VRCHAT_API_PASSWORD_CREDENTIAL_KEY, "default");
    if let Ok(p) = password_entry {
        p.set_password(&password).map_err(|e| e.to_string())?;
    } else {
        eprintln!("Failed to create password entry: {}", password_entry.err().unwrap());
        return Err("Failed to create password entry".into());
    };
    initialize_api(app.clone()).await.map_err(|_| {
        eprintln!("Failed to re-initialize API after updating credentials");
        "Failed to re-initialize API".to_string()
    })?;
    Ok(())
}