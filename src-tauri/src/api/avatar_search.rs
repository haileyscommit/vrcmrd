
use itertools::Itertools;
use parking_lot::Mutex;
use tauri::{AppHandle, Emitter, Manager};

/// Searches for avatars using third-party APIs (same as VRCX uses).
use crate::{api::VrchatApiStateMutex, memory::users::Users, types::{VrcMrdUser, avatar::{AvatarBundleFileMetadata, GetWorstRank, PerfRank, VrcxAvatarSearchResult}}};

// TODO: use a setting to get the list, and use this hardcoded list as the default instead
const AVATAR_API_BASE_URLS: [&str; 4] = [
    //"https://api.avtrdb.com/v2/avatar/search/vrcx", // seems to be blocked by cloudflare
    "https://api.avatarrecovery.com/Avatar/vrcx",
    "https://vrcx.vrcdb.com/avatars/Avatar/VRCX",
    "https://vrcx.avtr.zip/",
    "https://avatarwbvrcxsearch.worldbalancer.com/vrcx_search",
];
const AVATAR_API_BASE_URLS_SUPPORTING_FILEID: [&str; 1] = [
    // "https://api.avtrdb.com/v2/avatar/search/vrcx", // seems to be blocked by cloudflare
    "https://api.avatarrecovery.com/Avatar/vrcx",
    //"https://vrcx.vrcdb.com/avatars/Avatar/VRCX",
];
const AVATAR_API_BASE_URLS_SUPPORTING_AUTHORID: [&str; 2] = [
    "https://avatarwbvrcxsearch.worldbalancer.com/vrcx_search",
    // "https://api.avtrdb.com/v2/avatar/search/vrcx", // seems to be blocked by cloudflare
    "https://api.avatarrecovery.com/Avatar/vrcx",
    //"https://vrcx.vrcdb.com/avatars/Avatar/VRCX",
];

const AVATAR_SEARCH_SKIP_FILES: [&str; 1] = [
    "file_0e8c4e32-7444-44ea-ade4-313c010d4bae", // default robot avatar. Probably a VRC+ user.
];

pub fn get_file_id_from_image_url(image_url: &str) -> Option<String> {
    image_url.split(['/', '?', '&', '=']).find(|segment| segment.starts_with("file_")).map(|s| s.to_string())
}

pub async fn get_owner_id_from_file(file_id: &str, app: AppHandle) -> Option<String> {
    let response = try_request!(app.clone(), |config| {
        vrchatapi::apis::files_api::get_file(config, file_id)
    }, { wait_for_api_ready: true }).await;
    match response {
        Ok(Some(file_info)) => Some(file_info.owner_id),
        Ok(None) => {
            #[cfg(debug_assertions)]
            eprintln!("File with ID '{}' not found on VRChat API", file_id);
            None
        },
        Err(e) => {
            eprintln!("Failed to get file info from VRChat API for file ID '{}': {}", file_id, e);
            #[cfg(debug_assertions)]
            eprintln!("Error details: {:?}", e);
            None
        },
    }
}

/// Searches for an avatar by a file ID known to correspond to it (usually the banner image).
pub async fn search_avatar_by_file(file_id: &str, client: reqwest_middleware::ClientWithMiddleware) -> Result<Option<VrcxAvatarSearchResult>, String> {
    let mut lastError = None;
    if AVATAR_SEARCH_SKIP_FILES.contains(&file_id) {
        #[cfg(debug_assertions)]
        eprintln!("File ID '{}' is skipped for avatar search (likely a placeholder file ID)", file_id);
        return Ok(None);
    }
    // TODO: only check unique strings
    for base_url in AVATAR_API_BASE_URLS_SUPPORTING_FILEID.iter() {
        let url = format!("{}?fileId={}", base_url, file_id);
        println!("Searching for avatar with file ID '{}' at '{}'", file_id, url);
        match client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<Vec<VrcxAvatarSearchResult>>().await {
                        Ok(result) => {
                            if let Some(result) = result.first().cloned() {
                                return Ok(Some(result));
                            }
                        },
                        Err(e) => {
                            eprintln!("Failed to parse response from {}: {}", url, e);
                            lastError = Some(e.to_string());
                        }
                    }
                } else {
                    let errorMessage = format!("HTTP {}", response.status());
                    eprintln!("Failed to search avatar at {}: {}", base_url, errorMessage);
                    #[cfg(debug_assertions)]
                    eprintln!("Response body: {:?}", response.text().await);
                    lastError = Some(errorMessage);
                }
            },
            Err(e) => {
                eprintln!("Failed to send request to {}: {}", base_url, e);
                lastError = Some(e.to_string());
            },
        }
    }
    // TODO: look up the file ID on the VRChat API and get its owner, if possible
    // return search_avatar_by_author(owner_id).await?;
    if lastError.is_some() {
        Err(lastError.unwrap())
    } else {
        Ok(None)
    }
}

pub async fn search_avatar_by_name(query: &str, client: reqwest_middleware::ClientWithMiddleware) -> Result<Vec<VrcxAvatarSearchResult>, String> {
    for base_url in AVATAR_API_BASE_URLS.iter() {
        let url = format!("{}?search={}", base_url, query);
        match client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<Vec<VrcxAvatarSearchResult>>().await {
                        Ok(result) => return Ok(result),
                        Err(e) => eprintln!("Failed to parse response from {}: {}", base_url, e),
                    }
                } else {
                    eprintln!("Failed to search avatar at {}: HTTP {}", base_url, response.status());
                }
            },
            Err(e) => eprintln!("Failed to send request to {}: {}", base_url, e),
        }
    }
    Err("All avatar search attempts failed".to_string())
}

pub async fn search_avatar_by_author(owner_id: &str, query: Option<&str>, client: reqwest_middleware::ClientWithMiddleware) -> Result<Vec<VrcxAvatarSearchResult>, String> {
    //let urls = Iterator::chain(AVATAR_API_BASE_URLS.iter(), AVATAR_API_BASE_URLS_SUPPORTING_AUTHORID.iter()).unique();
    for base_url in AVATAR_API_BASE_URLS_SUPPORTING_AUTHORID.iter() {
        // let url = {
        //     if AVATAR_API_BASE_URLS_SUPPORTING_AUTHORID.contains(&base_url) {
        //         format!("{}?authorId={}", base_url, owner_id)
        //     } else {
        //         format!("{}?search={}", base_url, owner_id)
        //     }
        // };
        let query = {
            if let Some(query) = query {
                if query.is_empty() {
                    None
                } else {
                    Some(query)
                }
            } else {
                None
            }
        };
        let url = if let Some(query) = query {
            format!("{}?authorId={}&search={}", base_url, owner_id, query)
        } else {
            format!("{}?authorId={}", base_url, owner_id)
        };
        #[cfg(debug_assertions)]
        println!("Searching for avatar with author ID '{}' at '{}'", owner_id, url);
        match client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<Vec<VrcxAvatarSearchResult>>().await {
                        Ok(result) => return Ok(result),
                        Err(e) => eprintln!("Failed to parse response from {}: {}", base_url, e),
                    }
                } else {
                    eprintln!("Failed to search avatar at {}: HTTP {}", base_url, response.status());
                }
            },
            Err(e) => eprintln!("Failed to send request to {}: {}", base_url, e),
        }
    }
    Err("All avatar search attempts failed".to_string())
}

pub fn update_avatar(user: VrcMrdUser, app: AppHandle) {
    let app_clone = app.clone();
    // TODO: set up a unique username+tags+aviname disk cache
    tauri::async_runtime::spawn(async move {
        let user = user.clone();
        #[cfg(debug_assertions)]
        println!("Looking up avatar for user '{}'", user.username);
        let mut avatar_id: Option<String> = None;
        let mut found_performance_info = false;
        let client = {
            // Use the regular API client (throttled/debounced and cached)
            let config_state = app.state::<VrchatApiStateMutex>();
            let config_state = config_state.lock();
            let config = &config_state.config;
            config.as_ref().unwrap().client.clone()
        };
        'image: for image in user.avatar_images.iter().unique() {
            if AVATAR_SEARCH_SKIP_FILES.iter().any(|skipped| image.contains(skipped)) {
                #[cfg(debug_assertions)]
                eprintln!("Skipping avatar search by file or author ID for user '{}' because their avatar image URL contains a skipped file ID (usually placeholder)", user.username);
                continue;
            }
            let avatar_file_id = get_file_id_from_image_url(image);
            if avatar_file_id.is_none() {
                eprintln!("Failed to extract file ID from avatar image URL for user '{}'", user.username);
                return;
            }
            let file_id = avatar_file_id.unwrap();
            match crate::api::avatar_search::search_avatar_by_file(&file_id, client.clone()).await {
                Ok(Some(avatar_result)) => {
                    let avatar_name = avatar_result.name.clone().unwrap_or_else(|| "Unknown Avatar".to_string());
                    println!("Found avatar '{}' for user '{}'", avatar_name, user.username);
                    let (returned_avatar_id, has_perf_info) = update_avatar_from_search(&user, avatar_result, app.clone());
                    if has_perf_info {
                        found_performance_info = true;
                    }
                    if returned_avatar_id.is_some() {
                        avatar_id = returned_avatar_id;
                    }
                    break; // Stop after the first successful avatar lookup
                },
                // TODO: look up with other approaches here
                Ok(None) => {
                    #[cfg(debug_assertions)]
                    eprintln!("No avatar found for file ID '{}' when updating avatar for user '{}'", file_id, user.username);
                    // Get the owner of the file and try searching by author ID
                    if let Some(owner_id) = get_owner_id_from_file(&file_id, app_clone.clone()).await {
                        match crate::api::avatar_search::search_avatar_by_author(&owner_id, Some(&user.avatar_name), client.clone()).await {
                            Ok(results) => {
                                if results.is_empty() {
                                    #[cfg(debug_assertions)]
                                    eprintln!("No avatars found for author ID '{}' when updating avatar for user '{}'", owner_id, user.username);
                                } else {
                                    println!("Found {} possible avatars for author ID '{}' when updating avatar for user '{}'", results.len(), owner_id, user.username);
                                    for avatar_result in results {
                                        if avatar_result.name.is_some() && avatar_result.name.as_ref().unwrap().clone() == user.avatar_name {
                                            let avatar_name = avatar_result.name.clone().unwrap();
                                            println!("Found matching avatar '{}' for user '{}' by author ID search", avatar_name, user.username);
                                            let (returned_avatar_id, has_perf_info) = update_avatar_from_search(&user, avatar_result, app.clone());
                                            if has_perf_info {
                                                found_performance_info = true;
                                            }
                                            if returned_avatar_id.is_some() {
                                                avatar_id = returned_avatar_id;
                                            }
                                            break 'image; // Stop after the first successful avatar lookup
                                        }
                                    }
                                }
                            },
                            Err(e) => eprintln!("Failed to search avatar by author ID '{}' for user '{}': {}", owner_id, user.username, e),
                        }
                    } else {
                        eprintln!("Could not get owner ID from file ID '{}' for user '{}'; cannot search by author", file_id, user.username);
                    }
                }
                Err(e) => eprintln!("Failed to search avatar by file ID '{}': {}", file_id, e),
            }
        }
        let settled = {
            let instance_state = app_clone.state::<crate::memory::instance::InstanceStateMutex>();
            let instance_state = instance_state.lock();
            if !instance_state.settled {
                #[cfg(debug_assertions)]
                eprintln!("Instance is not settled yet while looking up avatar for user '{}', waiting for instance to settle before trying to use file metadata to look up avatar", user.clone().username);
            }
            instance_state.settled
        };
        let mut file_meta: Option<AvatarBundleFileMetadata> = None;
        if avatar_id.is_none() && settled {
            // TODO: use found-file metadata for avatar bundles, get the owner, look it up again by name (to hopefully more reliably get the cross-platform perf rank)
            let mut attempts = 0;
            loop {
                // Wait for all file metadata lookups to complete so we can scan them
                let state = app_clone.state::<crate::memory::users::avatar::AvatarsStateMutex>();
                let pending_lookups = {
                    let state = state.lock();
                    state.pending_file_metadata_lookups.clone()
                };
                if pending_lookups.is_empty() {
                    break;
                } else {
                    #[cfg(debug_assertions)]
                    println!("Waiting for {} pending file metadata lookups to complete before scanning for avatar bundles for user '{}'", pending_lookups.len(), user.clone().username);
                    attempts += 1;
                    if attempts > 20 {
                        eprintln!("Waited too long for file metadata lookups to complete while looking up avatar for user '{}', giving up on waiting", user.clone().username);
                        break;
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                }
            }
            const MIN_COEF: f32 = 0.4;
            file_meta = {
                let state = app_clone.state::<crate::memory::users::avatar::AvatarsStateMutex>();
                let state = state.lock();
                let user = user.clone();
                let filtered = fuzzy_cmp::search_filter(state.possible_avatar_files.as_slice(), &user.avatar_name, MIN_COEF, true, |f| {
                    f.avatar_name.as_ref().unwrap()
                });
                #[cfg(debug_assertions)]
                {
                    if filtered.is_empty() {
                        eprintln!("[DEBUG] No file metadata found with fuzzy match for avatar name '{}' for user '{}'", user.clone().avatar_name, user.clone().username);
                    } else {
                        println!("[DEBUG] Found {} possible file metadata matches for avatar name '{}' for user '{}'", filtered.len(), user.clone().avatar_name, user.clone().username);
                        for (coef, meta) in filtered.iter() {
                            eprintln!("[DEBUG] {:>6.2}%: file ID '{}', file avatar name '{}', user avatar name '{}'", (coef*100.0), meta.file_id, meta.avatar_name.as_ref().unwrap_or(&"Unknown".to_string()), user.clone().avatar_name);
                        }
                    }
                }
                filtered.first().cloned().and_then(|f| Some(f.1))
            };
            if file_meta.is_none() {
                eprintln!("No matching file metadata found for avatar '{}' worn by user '{}'", user.clone().avatar_name, user.clone().username);
            } else {
                let file_meta = file_meta.clone().unwrap();
                // look up this avatar owner on avatar search APIs to try to get the avatar ID and performance info
                #[cfg(debug_assertions)]
                println!("[DEBUG] Found possible avatar bundle file metadata for avatar '{}' worn by user '{}', looking up avatar info by author ID '{}' to try to get performance info", file_meta.avatar_name.as_ref().unwrap(), user.clone().username, file_meta.author_id);
                match crate::api::avatar_search::search_avatar_by_author(&file_meta.author_id, None, client.clone()).await {
                    Ok(results) => {
                        if results.is_empty() {
                            #[cfg(debug_assertions)]
                            eprintln!("[DEBUG] No avatars found for author ID '{}' when looking up avatar info for user '{}'", file_meta.author_id, user.clone().username);
                        } else {
                            println!("Found {} possible avatars for author ID '{}' when looking up avatar info for user '{}'", results.len(), file_meta.author_id, user.clone().username);
                            for avatar_result in results {
                                if avatar_result.name.is_some() && avatar_result.name.as_ref().unwrap() == file_meta.avatar_name.as_ref().unwrap() {
                                    let avatar_name = avatar_result.name.clone().unwrap();
                                    println!("Found matching avatar '{}' for user '{}' by author ID search (using file metadata)", avatar_name, user.clone().username);
                                    // Update the user in the user list with the new avatar data
                                    let (returned_avatar_id, has_perf_info) = update_avatar_from_search(&user, avatar_result, app.clone());
                                    if has_perf_info {
                                        found_performance_info = true;
                                    }
                                    if returned_avatar_id.is_some() {
                                        avatar_id = returned_avatar_id;
                                    }
                                    break; // Stop after the first successful avatar lookup
                                }
                            }
                        }
                    }
                    Err(e) => eprintln!("Failed to search avatar by author ID '{}' for user '{}': {}", file_meta.author_id, user.clone().username, e),
                }
            }
        }
        // TODO: if that doesn't work, get the analysis for the bundle itself to get the perf rank for that; which is only going to show one platform but it's better than nothing
        if avatar_id.is_none() && file_meta.is_some() && !found_performance_info {
            let file_meta = file_meta.clone().unwrap();
            #[cfg(debug_assertions)]
            println!("[DEBUG] Attempting to look up file analysis for file ID '{}' and version '{}' to get performance info for avatar '{}' worn by user '{}'", file_meta.file_id, file_meta.file_version, file_meta.avatar_name.as_ref().unwrap(), user.clone().username);
            // Fall back to looking up the file bundle analysis
            let response = try_request!(app_clone.clone(), |config| {
                vrchatapi::apis::files_api::get_file_analysis(config, &file_meta.file_id, file_meta.file_version.cast_signed())
            }, { wait_for_api_ready: true }).await;
            match response {
                Ok(Some(analysis)) => {
                    if let Some(rating) = analysis.performance_rating {
                        let users_state = app_clone.state::<Mutex<Users>>();
                        let users_state = users_state.try_lock_for(std::time::Duration::from_millis(100));
                        if users_state.is_none() {
                            eprintln!("Failed to acquire lock on users state to update performance info for user '{}'", user.clone().username);
                            return;
                        }
                        let mut users_state = users_state.unwrap();
                        if let Some(user) = users_state
                            .inner
                            .iter_mut()
                            .find(|u| u.username == user.clone().username)
                        {
                            user.perf_rank = PerfRank::from_string(&rating);
                            // Emit an event with the updated performance info
                            if let Err(e) = app_clone.emit("vrcmrd:update-user", user.clone()) {
                                eprintln!("Failed to emit update-user event with performance info from file analysis: {}", e);
                            }
                        }
                    } else {
                        #[cfg(debug_assertions)]
                        eprintln!("[DEBUG] File analysis for file ID '{}' does not contain performance rating when looking up avatar info for user '{}'", file_meta.file_id, user.clone().username);
                    }
                },
                Ok(None) => eprintln!("File analysis for file ID '{}' not found on VRChat API when looking up avatar info for user '{}'", file_meta.file_id, user.clone().username),
                Err(e) => eprintln!("Failed to get file analysis from VRChat API for file ID '{}' when looking up avatar info for user '{}': {}", file_meta.file_id, user.clone().username, e)
            }
        }
        if avatar_id.is_some() && !found_performance_info {
            println!("Looking up avatar performance info from VRChat API for avatar ID '{}' (worn by '{}')", avatar_id.clone().unwrap(), user.clone().username);
            let avatar_id = avatar_id.clone().unwrap();
            let response = try_request!(app_clone.clone(), |config| {
                vrchatapi::apis::avatars_api::get_avatar(config, avatar_id.as_str())
            }, { max_attempts: 1 }).await;
            match response {
                Ok(Some(avatar_info)) => {
                    let performance = avatar_info.performance.clone();
                    let users_state = app_clone.state::<Mutex<Users>>();
                    let users_state = users_state.try_lock_for(std::time::Duration::from_millis(100));
                    if users_state.is_none() {
                        eprintln!("Failed to acquire lock on users state to update performance info for user '{}'", user.clone().username);
                        return;
                    }
                    let mut users_state = users_state.unwrap();
                    if let Some(user) = users_state
                        .inner
                        .iter_mut()
                        .find(|u| u.username == user.clone().username)
                    {
                        user.perf_rank = performance.get_worst_rank();
                    }
                    // Emit an event with the updated performance info
                    if let Err(e) = app_clone.emit("vrcmrd:update-user", avatar_info) {
                        eprintln!("Failed to emit update-user event with performance info: {}", e);
                    }
                },
                Ok(None) => eprintln!("Avatar with ID '{}' not found on VRChat API when looking up performance info for user '{}'", avatar_id, user.clone().username),
                Err(e) => eprintln!("Failed to get avatar info from VRChat API for avatar ID '{}' when looking up performance info for user '{}': {}", avatar_id, user.clone().username, e),
            }
        }
    });
}

fn update_avatar_from_search(user: &VrcMrdUser, result: VrcxAvatarSearchResult, app: AppHandle) -> (Option<String>, bool) {
    let mut found_performance_info = false;
    // Update the user in the user list with the new avatar data
    let users_state = app.state::<Mutex<Users>>();
    let users_state = users_state.try_lock_for(std::time::Duration::from_millis(100));
    if users_state.is_none() {
        eprintln!("Failed to acquire lock on users state to update avatar for user '{}'", user.username);
        return (None, false);
    }
    let mut users_state = users_state.unwrap();
    if let Some(user) = users_state
        .inner
        .iter_mut()
        .find(|u| u.username == user.username)
    {
        user.avatar_id = Some(result.id.clone());
        // if let Some(name) = result.name.clone() {
        //     user.avatar_name = name;
        // }
        if let Some(performance) = result.performance.clone() {
            user.perf_rank = performance.get_worst_rank();
            found_performance_info = true;
        }
    }
    // Emit an event
    if let Err(e) = app.emit("vrcmrd:update-user", result) {
        eprintln!("Failed to emit update-user event: {}", e);
    };
    (user.avatar_id.clone(), found_performance_info)
}