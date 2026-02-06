use std::ops::Deref;

use reqwest::cookie::CookieStore;
use tauri::{async_runtime::Mutex, AppHandle, Emitter, Listener, Manager, Wry};
use vrchatapi::{apis::configuration::Configuration, models::RegisterUserAccount200Response};

#[macro_use]
mod request;

pub mod user;
pub mod groups;
pub mod xsoverlay;

pub const VRCHAT_AUTH_COOKIE_CREDENTIAL_KEY: &str = "VRChatCookies";
pub const VRCHAT_API_USERNAME_CREDENTIAL_KEY: &str = "VRC_USERNAME";
pub const VRCHAT_API_PASSWORD_CREDENTIAL_KEY: &str = "VRC_PASSWORD";

#[derive(Debug, Clone)]
pub struct VrchatApiState {
    pub mode: VrchatApiMode,
    pub cookies: Option<std::sync::Arc<reqwest::cookie::Jar>>,
    pub config: Option<Configuration>,
}
impl VrchatApiState {
    pub fn not_ready() -> VrchatApiState {
        VrchatApiState {
            mode: VrchatApiMode::NotReady,
            cookies: None,
            config: None,
        }
    }
    // pub fn new_preparing(config: Configuration) -> VrchatApiState {
    //     VrchatApiState {
    //         mode: VrchatApiMode::Preparing,
    //         cookies: None,
    //         config: Some(config),
    //     }
    // }
    // pub fn new_mode(mode: VrchatApiMode) -> VrchatApiState {
    //     VrchatApiState {
    //         mode,
    //         cookies: None,
    //         config: None,
    //     }
    // }
}


#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum VrchatApiMode {
    /// The API is not initializing. This may be because no credentials were supplied.
    #[default]
    NotReady,
    /// The API is in the process of initializing.
    Preparing,
    /// The API has been provided a two-factor authentication code and is waiting for verification.
    TwoFactorCode(String),
    /// The API is fully initialized and ready to use
    /// with the given configuration.
    Ready,
}
pub type VrchatApiStateMutex = Mutex<VrchatApiState>;

pub async fn initialize_api(app: AppHandle) -> Result<(), ()> {
    let mut config = Configuration::default();
    // Get existing config from state if available
    {
        let state = app.state::<VrchatApiStateMutex>();
        let guard = state.lock().await;
        if let VrchatApiState {
            config: Some(existing_config),
            ..
        } = &*guard
        {
            config = existing_config.clone();
        }
    }
    let cookie_jar = {
        let state = app.state::<VrchatApiStateMutex>();
        let guard = state.lock().await;
        if let VrchatApiState {
            cookies: Some(existing_cookies),
            ..
        } = &*guard
        {
            std::sync::Arc::clone(existing_cookies)
        } else {
            std::sync::Arc::new(reqwest::cookie::Jar::default())
        }
    };
    let username = {
        let entry = keyring_core::Entry::new(VRCHAT_API_USERNAME_CREDENTIAL_KEY, "default");
        if let Ok(e) = entry {
            match e.get_password() {
                Ok(username) => username,
                Err(e) => {
                    eprintln!("Failed to get stored username from keyring: {:?}", e);
                    return Err(());
                }
            }
        } else {
            eprintln!("No stored username found in keyring.");
            return Err(());
        }
    };
    config.client = reqwest::Client::builder()
        .cookie_provider(std::sync::Arc::clone(&cookie_jar))
        .build()
        .unwrap();
    let api_key_entry = keyring_core::Entry::new(VRCHAT_AUTH_COOKIE_CREDENTIAL_KEY, username.as_str());
    if let Err(e) = &api_key_entry {
        eprintln!("No stored auth cookie found in keyring: {:?}", e);
        return Err(());
    }
    if let Ok(stored_cookie) = api_key_entry.unwrap().get_password() {
        let cookie = stored_cookie;
        let url = reqwest::Url::parse(&config.base_path).unwrap();
        cookie_jar.add_cookie_str(&cookie, &url);
        println!("Using stored auth cookie for VRChat API.");
    } else {
        println!("No stored auth cookie found; logging in with username and password!");
        config.basic_auth = Some((
            username.clone(),
            {
                let entry = keyring_core::Entry::new(VRCHAT_API_PASSWORD_CREDENTIAL_KEY, "default");
                if let Ok(e) = entry {
                    match e.get_password() {
                        Ok(password) => Some(password),
                        Err(e) => {
                            eprintln!("Failed to get stored password from keyring: {:?}", e);
                            return Err(());
                        }
                    }
                } else {
                    eprintln!("No stored password found in keyring.");
                    return Err(());
                }
            },
        ));
    }
    #[cfg(debug_assertions)]
    if config.basic_auth.is_some() {
        println!(
            "Using VRChat username: {} and password: {}",
            config.basic_auth.as_ref().unwrap().0,
            config
                .basic_auth
                .as_ref()
                .unwrap()
                .1
                .as_ref()
                .unwrap_or(&"<empty>".to_string())
        );
    }
    config.user_agent = Some("vrcmrd/0.1.0 vrcmrd.by.haileyscommit@fastmail.com".to_string());

    // If there's a 2FA code, verify the session with it.
    let two_factor_code = {
        let mutex = app.state::<VrchatApiStateMutex>();
        let guard = mutex.lock().await;
        match guard.clone().mode {
            VrchatApiMode::TwoFactorCode(code) => Some(code),
            _ => None,
        }
    };
    if let Some(code) = two_factor_code {
        match vrchatapi::apis::authentication_api::verify2_fa(
            &config,
            vrchatapi::models::TwoFactorAuthCode::new(code.clone()),
        )
        .await
        {
            Ok(response) => {
                if !response.verified {
                    eprintln!("Two-factor authentication code was not verified.");
                    // Change the state back to NotReady to force re-initialization
                    *app.state::<VrchatApiStateMutex>().lock().await = VrchatApiState::not_ready();
                    return Err(());
                }
                println!("Two-factor authentication code verified successfully.");
            }
            Err(e) => {
                eprintln!("Failed to verify two-factor authentication code: {:?}", e);
                // Change the state back to NotReady to force re-initialization
                *app.state::<VrchatApiStateMutex>().lock().await = VrchatApiState::not_ready();
                return Err(());
            }
        }
    }

    // Log in
    match vrchatapi::apis::authentication_api::get_current_user(&config).await {
        Ok(user) => {
            match user {
                RegisterUserAccount200Response::CurrentUser(user_info) => {
                    println!("Logged in as {} ({})", user_info.display_name, user_info.id);
                    // TODO: save auth cookie in secure storage for later use
                    cookie_jar.cookies(&reqwest::Url::parse(format!("{}/{}", config.base_path, "auth/user").as_str()).unwrap()).map(|c| {
                        c.to_str().unwrap_or_default().split(';').for_each(|cookie| {
                            let kv = cookie.splitn(2, '=').collect::<Vec<&str>>();
                            if kv[0].trim() == "auth" {
                                println!("Auth cookie received!");
                                keyring_core::Entry::new(VRCHAT_AUTH_COOKIE_CREDENTIAL_KEY, user_info.username.as_deref().unwrap_or(&user_info.display_name))
                                    .and_then(|entry| entry.set_password(cookie))
                                    .unwrap_or_else(|e| eprintln!("Failed to store auth cookie in keyring: {:?}", e));
                            }
                        });
                    });
                    config.basic_auth = None; // Clear basic auth to use cookie-based auth
                    *app.state::<VrchatApiStateMutex>().lock().await = VrchatApiState {
                        mode: VrchatApiMode::Ready,
                        cookies: Some(std::sync::Arc::clone(&cookie_jar)),
                        config: Some(config.clone()),
                    };
                    Ok(())
                }
                RegisterUserAccount200Response::RequiresTwoFactorAuth(info) => {
                    // Request two-factor auth token from user via UI
                    // (the UI should then call a command to provide the token and continue initialization)
                    let _ = app.emit(
                        "vrcmrd:auth-token-needed",
                        info.clone().requires_two_factor_auth,
                    );
                    eprintln!("Two-factor authentication required. Requesting token from user. Info: {:?}", info);
                    //app.exit(401);
                    *app.state::<VrchatApiStateMutex>().lock().await = VrchatApiState {
                        mode: VrchatApiMode::Preparing,
                        cookies: Some(std::sync::Arc::clone(&cookie_jar)),
                        config: Some(config.clone()),
                    };
                    Ok(())
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to log in: {:?}", e);
            //app.exit(403);
            Err(())
        }
    }
}

#[tauri::command]
pub async fn logout(app: tauri::AppHandle<Wry>) -> Result<(), String> {
    try_request!(app.app_handle().clone(), |config| {
        vrchatapi::apis::authentication_api::logout(config)
    }, {});
    // Clear stored credentials
    let username_entry = keyring_core::Entry::new(VRCHAT_API_USERNAME_CREDENTIAL_KEY, "default");
    if let Ok(u) = username_entry {
        match u.get_password() {
            Ok(username) => {
                keyring_core::Entry::new(VRCHAT_AUTH_COOKIE_CREDENTIAL_KEY, &username)
                    .and_then(|entry| entry.delete_credential())
                    .map_err(|e| e.to_string())?;
                u.delete_credential().map_err(|e| e.to_string())?;
            }
            Err(keyring_core::Error::NoEntry) => {
                // No existing entry; nothing to delete
            }
            Err(e) => {
                eprintln!("Failed to access existing username entry: {}", e);
                //return Err(e.to_string());
            }
        };
    } else {
        eprintln!("Failed to create username entry: {}", username_entry.err().unwrap());
        //return Err("Failed to create username entry".into());
    };
    let password_entry = keyring_core::Entry::new(VRCHAT_API_PASSWORD_CREDENTIAL_KEY, "default");
    if let Ok(p) = password_entry {
        p.delete_credential().map_err(|e| e.to_string())?;
    } else {
        eprintln!("Failed to find password entry: {}", password_entry.err().unwrap());
        //return Err("Failed to create password entry".into());
    };
    // Reset API state
    let api_state_mutex = app.state::<VrchatApiStateMutex>().clone();
    let mut api_state = api_state_mutex.lock().await;
    *api_state = VrchatApiState::not_ready();
    Ok(())
}

#[tauri::command]
pub async fn submit_2fa_token(
    app: tauri::AppHandle<Wry>,
    token: String,
    config_state: tauri::State<'_, VrchatApiStateMutex>,
) -> Result<(), String> {
    println!("Received 2FA token from UI.");
    let (config, cookies) = {
        let mut guard = config_state.deref().lock().await;
        match guard.clone() {
            VrchatApiState {
                mode: VrchatApiMode::Preparing,
                cookies: Some(cookies),
                config: Some(cfg),
            } => (cfg, cookies),
            VrchatApiState {
                mode: VrchatApiMode::TwoFactorCode(_),
                cookies: Some(cookies),
                config: Some(cfg),
            } => (cfg, cookies),
            other => {
                *guard = other.clone();
                eprintln!("API state not ready for 2FA token submission: {:?}", other);
                return Err("API not ready".into());
            }
        }
    };
    let _ = {
        let mut api_state = config_state.deref().lock().await;
        *api_state = VrchatApiState {
            mode: VrchatApiMode::TwoFactorCode(token),
            cookies: Some(cookies),
            config: Some(config.clone()),
        };
    };

    #[cfg(debug_assertions)]
    {
        let api_state = config_state.deref().lock().await;
        if api_state.mode != VrchatApiMode::Ready {
            eprintln!("DEBUG: Status: {:?}", api_state);
        }
    }

    // Now re-initialize the API with the provided token
    let app_clone = app.clone();
    println!("Submitting 2FA token and re-initializing VRChat API.");
    if let Err(_) = initialize_api(app_clone.clone()).await {
        eprintln!("Failed to initialize VRChat API after submitting 2FA token.");
        *app_clone.state::<VrchatApiStateMutex>().lock().await = VrchatApiState::not_ready();
        return Err("Failed to initialize VRChat API after submitting 2FA token.".into());
    }

    {
        let api_state = config_state.deref().lock().await;
        if api_state.mode != VrchatApiMode::Ready {
            eprintln!(
                "VRChat API not ready after submitting 2FA token. Status: {:?}",
                api_state
            );
            return Err("Failed to initialize VRChat API after submitting 2FA token.".into());
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn cancel_login(app: tauri::AppHandle<Wry>) -> Result<(), String> {
    // User cancelled the login process (like a broken log-out).
    let api_state_mutex = app.state::<VrchatApiStateMutex>().clone();
    let mut api_state = api_state_mutex.lock().await;
    *api_state = VrchatApiState::not_ready();
    Ok(())
}

/// Initializes the VRChat API and manages its configuration (authentication, mostly).
/// To use the API:
/// ```rust
/// //app is a tauri::AppHandle
/// let api_state = app.state::<VrchatApiStateMutex>().lock().unwrap();
/// if let Some(VrchatApiState { mode: VrchatApiMode::Ready, config: Some(config) }) = &*api_state {
///     // Pass `config` to vrchatapi::apis methods
/// }
/// // You can also check for Preparing and NotReady states as needed.
/// ```
pub fn vrchat_api_plugin() -> tauri::plugin::TauriPlugin<Wry> {
    tauri::plugin::Builder::new("instance_id_memory")
        .setup(|app, _api| {
            app.manage::<VrchatApiStateMutex>(Mutex::new(VrchatApiState {
                mode: VrchatApiMode::NotReady,
                config: None,
                cookies: None,
            }));
            // These two clones are needed to prevent borrowing issues.
            let app_handle = app.clone();
            let app_handle_for_once = app_handle.clone();
            app_handle.once("vrcmrd:ui-ready", move |_e| {
                tauri::async_runtime::spawn(async move {
                    println!("Initializing VRChat API...");
                    *app_handle_for_once
                        .state::<VrchatApiStateMutex>()
                        .lock().await = VrchatApiState {
                        mode: VrchatApiMode::Preparing,
                        cookies: None,
                        config: None,
                    };
                    // These clones are also needed to prevent borrowing issues. Yay rust!
                    let task_handle = app_handle_for_once.clone();
                    let app_handle_reset_state = app_handle_for_once.clone();
                    match initialize_api(task_handle).await {
                        Ok(_) => (), // The API is ready and the configuration state has been updated.
                        Err(_) => {
                            *app_handle_reset_state
                                .state::<VrchatApiStateMutex>()
                                .lock().await = VrchatApiState::not_ready();
                        }
                    }
                    drop(app_handle_for_once);
                    drop(app_handle_reset_state);
                });
            });
            drop(app_handle);
            Ok(())
        })
        .build()
}
