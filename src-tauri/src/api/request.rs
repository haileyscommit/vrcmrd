pub struct VrchatTryRequestOptions {
    pub initial_backoff_secs: Option<u64>,
    pub max_attempts: Option<u8>,
    pub max_backoff_secs: Option<u64>,
    pub wait_for_api_ready: bool,
}
impl Default for VrchatTryRequestOptions {
    fn default() -> Self {
        VrchatTryRequestOptions {
            initial_backoff_secs: Some(5),
            max_attempts: Some(5),
            max_backoff_secs: Some(300),
            wait_for_api_ready: false,
        }
    }
}
impl VrchatTryRequestOptions {
    pub fn unlimited() -> Self {
        VrchatTryRequestOptions {
            initial_backoff_secs: None,
            max_attempts: None,
            max_backoff_secs: None,
            wait_for_api_ready: false,
        }
    }
    pub fn wait_for_api_ready(&self) -> Self {
        VrchatTryRequestOptions {
            wait_for_api_ready: true,
            ..*self
        }
    }
}

/// Macro to perform an API request with retry logic on rate limiting (HTTP 429).
/// The macro takes a guard to the API state, a closure representing the API request,
/// and an optional struct-like block of options to customize the retry behavior.
///
/// Example:
/// ```
/// let response = try_request!(guard, |config| {
///     vrchatapi::apis::instances_api::get_instance(config, &instance_id.world, &instance_id.id)
/// }, { wait_for_api_ready: true }).await;
/// match response {
///     Ok(Some(instance_info)) => {
///        ...
///     }
///     Ok(None) => {
///        // API not ready
///     }
///     Err(e) => {
///        eprintln!("Failed to fetch instance info: {:?}", e);
///     }
/// }
/// ```
#[macro_export]
macro_rules! try_request {
    // no-options form -> use defaults
    ($handle:expr, $f:expr $(,)?) => {{
        async {
            let initial_backoff_secs: Option<u64> = Some(5);
            let mut secs = initial_backoff_secs.unwrap_or(5);
            let max_attempts: Option<u8> = Some(5);
            let max_backoff_secs: Option<u64> = Some(300);
            let wait_for_api_ready: bool = false;

            let handle: tauri::AppHandle = &$handle;
            let mut state = {
                let state_lock = handle.state::<crate::api::VrchatApiStateMutex>();
                state_lock.lock().await
            };

            if wait_for_api_ready {
                println!("Waiting for API to become ready...");
                let mut i = 0;
                while !(state.mode == crate::api::VrchatApiMode::Ready && state.config.is_some()) {
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    state = {
                        let state_lock = handle.state::<crate::api::VrchatApiStateMutex>();
                        state_lock.lock().await
                    };
                    i += 1;
                    if i % 10 == 0 {
                        println!("...still waiting for API to become ready...");
                    }
                    if i >= 30 {
                        eprintln!("Timed out waiting for API to become ready after 30 seconds");
                        return Ok(None);
                    }
                }
            }

            if state.mode == crate::api::VrchatApiMode::Ready && state.config.is_some() {
                match $f(&state.config.clone().unwrap()).await {
                    Ok(result) => Ok(Some(result)),
                    Err(e) => {
                        eprintln!("API request failed: {:?}", e);
                        if format!("{:?}", e).contains("429") {
                            let mut secs = initial_backoff_secs.unwrap_or(5);
                            let mut attempts: u8 = 1;
                            loop {
                                tokio::time::sleep(std::time::Duration::from_secs(secs)).await;
                                match $f(&state.config.clone().unwrap()).await {
                                    Ok(result) => return Ok(Some(result)),
                                    Err(e) => {
                                        eprintln!("API request failed after {}-second backoff: {:?}", secs, e);
                                        if format!("{:?}", e).contains("429") {
                                            secs *= 2;
                                            attempts += 1;
                                            if let Some(max) = max_attempts {
                                                if attempts >= max {
                                                    eprintln!("Max API request attempts reached ({}), giving up", max);
                                                    return Err(e);
                                                }
                                            }
                                            if let Some(max_backoff) = max_backoff_secs {
                                                if secs >= max_backoff {
                                                    secs = max_backoff;
                                                }
                                            }
                                        } else {
                                            return Err(e);
                                        }
                                    }
                                }
                            }
                        }
                        Err(e)
                    }
                }
            } else {
                Ok(None)
            }
        }
    }};

    // options form with an inline struct-like block of optional fields
    ($handle:expr, $f:expr, { $( initial_backoff_secs : $initial_backoff_secs:tt )? $(,)? $( max_attempts : $max_attempts:tt )? $(,)? $( max_backoff_secs : $max_backoff_secs:tt )? $(,)? $( wait_for_api_ready : $wait_for_api_ready:tt )? $(,)? } ) => {{
        #[allow(unused_mut)]
        async {
            let initial_backoff_secs: Option<u64> = None;
            $( let initial_backoff_secs = Some($initial_backoff_secs); )?
            let mut secs = initial_backoff_secs.unwrap_or(5);

            let mut max_attempts: Option<u8> = None;
            $( let max_attempts = Some($max_attempts); )?

            let mut max_backoff_secs: Option<u64> = None;
            $( let max_backoff_secs = Some($max_backoff_secs); )?
            
            #[allow(unused_variables)]
            let mut wait_for_api_ready: bool = false;
            $( let wait_for_api_ready = $wait_for_api_ready; )?

            let handle: tauri::AppHandle = $handle.clone();
            let state_lock = handle.state::<crate::api::VrchatApiStateMutex>();
            let mut state = state_lock.try_lock().ok();

            if state.is_none() || (wait_for_api_ready && !(state.as_ref().unwrap().mode == crate::api::VrchatApiMode::Ready && state.as_ref().unwrap().config.is_some())) {
                println!("Waiting for API to become ready...");
                let mut i = 0;
                loop {
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    state = state_lock.try_lock().ok();
                    if state.is_some() && state.as_ref().unwrap().mode == crate::api::VrchatApiMode::Ready && state.as_ref().unwrap().config.is_some() {
                        break;
                    }
                    i += 1;
                    if i % 10 == 0 {
                        println!("...still waiting for API to become ready...");
                    }
                    if i >= 30 {
                        eprintln!("Timed out waiting for API to become ready after 30 seconds");
                        return Ok(None);
                    }
                }
            } else {
                println!("API is ready, proceeding with request...");
            }

            let state = state.unwrap();

            if state.mode == crate::api::VrchatApiMode::Ready && state.config.is_some() {
                match $f(&state.config.clone().unwrap()).await {
                    Ok(result) => Ok(Some(result)),
                    Err(e) => {
                        eprintln!("API request failed: {:?}", e);
                        if format!("{:?}", e).contains("429") {
                            let mut secs = secs;
                            let mut attempts: u8 = 1;
                            loop {
                                tokio::time::sleep(std::time::Duration::from_secs(secs)).await;
                                match $f(&state.config.clone().unwrap()).await {
                                    Ok(result) => return Ok(Some(result)),
                                    Err(e) => {
                                        eprintln!("API request failed after {}-second backoff: {:?}", secs, e);
                                        if format!("{:?}", e).contains("429") {
                                            secs *= 2;
                                            attempts += 1;
                                            if let Some(max) = max_attempts {
                                                if attempts >= max {
                                                    eprintln!("Max API request attempts reached ({}), giving up", max);
                                                    return Err(e);
                                                }
                                            }
                                            if let Some(max_backoff) = max_backoff_secs {
                                                if secs >= max_backoff {
                                                    secs = max_backoff;
                                                }
                                            }
                                        } else {
                                            return Err(e);
                                        }
                                    }
                                }
                            }
                        }
                        Err(e)
                    }
                }
            } else {
                Ok(None)
            }
        }
    }};
}