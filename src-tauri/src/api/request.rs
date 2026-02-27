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
#[allow(unused_mut, unused_must_use)] // acknowledged. handled elsewhere.
macro_rules! try_request {
    // no-options form -> use defaults
    ($handle:expr, $f:expr $(,)?) => {{
        async {
            let initial_backoff_secs: Option<u64> = Some(5);
            let secs = initial_backoff_secs.unwrap_or(5);
            let max_attempts: Option<u8> = Some(5);
            let max_backoff_secs: Option<u64> = Some(300);
            let wait_for_api_ready: bool = false;

            let handle: tauri::AppHandle = &$handle;
            let mut waited = false;
            let mut i = 0;
            let config = loop {
                let state_snapshot = {
                    let state_lock = handle.state::<crate::api::VrchatApiStateMutex>();
                    let state = state_lock.lock();
                    (state.mode.clone(), state.config.clone())
                };

                if state_snapshot.0 == crate::api::VrchatApiMode::Ready && state_snapshot.1.is_some() {
                    break state_snapshot.1.unwrap();
                }

                if !wait_for_api_ready {
                    return Ok(None);
                }

                if !waited {
                    println!("Waiting for API to become ready...");
                    waited = true;
                }

                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                i += 1;
                if i % 10 == 0 {
                    println!("...still waiting for API to become ready...");
                }
                if i >= 30 {
                    eprintln!("Timed out waiting for API to become ready after 30 seconds");
                    return Ok(None);
                }
            };

            match $f(&config).await {
                    Ok(result) => Ok(Some(result)),
                    Err(e) => {
                        eprintln!("API request failed: {:?}", e);
                        if format!("{:?}", e).contains("429") {
                            let mut secs = initial_backoff_secs.unwrap_or(5);
                            let mut attempts: u8 = 1;
                            loop {
                                tokio::time::sleep(std::time::Duration::from_secs(secs)).await;
                                match $f(&config).await {
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
        }
    }};

    // options form with an inline struct-like block of optional fields
    ($handle:expr, $f:expr, { $( initial_backoff_secs : $initial_backoff_secs:tt )? $(,)? $( max_attempts : $max_attempts:tt )? $(,)? $( max_backoff_secs : $max_backoff_secs:tt )? $(,)? $( wait_for_api_ready : $wait_for_api_ready:tt )? $(,)? } ) => {{
        async {
            use tauri::{Manager};
            let initial_backoff_secs: Option<u64> = None;
            $( let initial_backoff_secs = Some($initial_backoff_secs); )?
            let secs = initial_backoff_secs.unwrap_or(5);

            let max_attempts: Option<u8> = None;
            $( let max_attempts = Some($max_attempts); )?

            let max_backoff_secs: Option<u64> = None;
            $( let max_backoff_secs = Some($max_backoff_secs); )?

            #[allow(unused_variables)]
            let wait_for_api_ready: bool = false;
            $( let wait_for_api_ready = $wait_for_api_ready; )?

            let handle: tauri::AppHandle = $handle.clone();
            let instance_id = {
                let state_lock = handle.state::<crate::memory::instance::InstanceStateMutex>();
                let state = state_lock.lock();
                state.id.clone()
            };
            let mut waited = false;
            let mut i = 0;
            let config = loop {
                let new_instance_id = {
                    let state_lock = handle.state::<crate::memory::instance::InstanceStateMutex>();
                    let state = state_lock.lock();
                    state.id.clone()
                };
                if new_instance_id.is_some() && instance_id.is_some() && new_instance_id != instance_id {
                    eprintln!("Instance ID changed while waiting for API ready! Original instance ID: {:?}, new instance ID: {:?}. Discarding response.", instance_id, new_instance_id);
                    return Ok(None);
                }
                
                let state = {
                    let state_lock = handle.state::<crate::api::VrchatApiStateMutex>();
                    state_lock
                        .try_lock()
                        .map(|state| (state.mode.clone(), state.config.clone()))
                };

                if let Some((mode, config)) = state {
                    if mode == crate::api::VrchatApiMode::Ready && config.is_some() {
                        if wait_for_api_ready && waited {
                            println!("API is ready, proceeding with request...");
                        }
                        break config.unwrap();
                    }
                }

                if !wait_for_api_ready {
                    eprintln!("API not initialized, cannot perform request yet");
                    return Ok(None);
                }

                if !waited {
                    println!("Waiting for API to become ready...");
                    waited = true;
                }

                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                i += 1;
                if i % 10 == 0 {
                    println!("...still waiting for API to become ready...");
                }
                if i >= max_attempts.unwrap_or(30) {
                    eprintln!("Timed out waiting for API to become ready after 30 seconds");
                    return Ok(None);
                }
            };

            match $f(&config).await {
                    Ok(result) => {
                        let new_instance_id = {
                            let state_lock = handle.state::<crate::memory::instance::InstanceStateMutex>();
                            let state = state_lock.lock();
                            state.id.clone()
                        };
                        if new_instance_id.is_some() && instance_id.is_some() && new_instance_id != instance_id {
                            eprintln!("Instance ID changed while waiting for API response! Original instance ID: {:?}, new instance ID: {:?}. Discarding response.", instance_id, new_instance_id);
                            return Ok(None);
                        }
                        Ok(Some(result))
                    },
                    Err(e) => {
                        eprintln!("API request failed: {:?}", e);
                        if format!("{:?}", e).contains("429") {
                            let mut secs = secs;
                            let mut attempts: u8 = 1;
                            loop {
                                tokio::time::sleep(std::time::Duration::from_secs(secs)).await;
                                let new_instance_id = {
                                    let state_lock = handle.state::<crate::memory::instance::InstanceStateMutex>();
                                    let state = state_lock.lock();
                                    state.id.clone()
                                };
                                if new_instance_id.is_some() && instance_id.is_some() && new_instance_id != instance_id {
                                    eprintln!("Instance ID changed while waiting for API ready! Original instance ID: {:?}, new instance ID: {:?}. Discarding response.", instance_id, new_instance_id);
                                    return Ok(None);
                                }
                                match $f(&config).await {
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
        }
    }};
}
