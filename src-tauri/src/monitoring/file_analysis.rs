// Context: the log line is:
// [API] Requesting Get analysis/file_e97b6623-bc25-4689-84c1-ba4dce26bd9a/2/security {{}} retryCount: 2

use tauri::{AppHandle, Emitter, Manager};

use crate::{monitoring::VrcLogEntry, try_request};

const AVATAR_FILE_ANALYSIS_PREFIX: &str = "[API] Requesting Get analysis/file_";

/// Handles a file analysis request log line by looking up the file metadata.
pub fn handle_file_analysis_request(app: AppHandle, line: &VrcLogEntry) -> Result<bool, tauri::Error> {
    let message = &line.message;
    if message.starts_with(AVATAR_FILE_ANALYSIS_PREFIX) {
        let url = message[AVATAR_FILE_ANALYSIS_PREFIX.len()..]
            .split_whitespace()
            .next()
            .unwrap_or("")
            .to_string();
        let file_id = message.split('/').find(|s| s.starts_with("file_")).unwrap_or("").to_string();
        if file_id.is_empty() {
            #[cfg(debug_assertions)]
            eprintln!("[DEBUG] Could not extract file ID from log line '{}'", message);
            return Ok(true);
        }
        // check if the file ID is already in the list
        {
            let state = app.state::<crate::memory::users::avatar::AvatarsStateMutex>();
            let state = state.lock();
            if state.possible_avatar_files.iter().any(|f| f.file_id == file_id) {
                #[cfg(debug_assertions)]
                println!("[DEBUG] File ID '{}' is already in possible_avatar_files, skipping lookup", file_id);
                return Ok(true);
            }
        }
        println!("Requesting file metadata for file ID '{}'", file_id);
        let mut already_pending = false;
        {
            // Add to pending_file_metadata_lookups if not already there
            let state = app.state::<crate::memory::users::avatar::AvatarsStateMutex>();
            let mut state = state.lock();
            if !state.pending_file_metadata_lookups.contains(&file_id) {
                state.pending_file_metadata_lookups.push(file_id.clone());
            } else {
                already_pending = true;
            }
        }
        let already_pending = already_pending;
        if already_pending {
            #[cfg(debug_assertions)]
            println!("[DEBUG] File ID '{}' is already pending lookup", file_id);
            return Ok(true);
        }
        let app_clone = app.clone();
        tauri::async_runtime::spawn(async move {
            let app = app_clone.clone();
            // wait until the instance is settled before looking these up, for good measure
            loop {
                {
                    let state = app.state::<crate::memory::instance::InstanceStateMutex>();
                    let state = state.lock();
                    if state.settled {
                        let state = app.state::<crate::memory::users::avatar::AvatarsStateMutex>();
                        let state = state.lock();
                        if state.pending_file_metadata_lookups.contains(&file_id) {
                            #[cfg(debug_assertions)]
                            println!("[DEBUG] Instance is settled, proceeding with file metadata lookup for file ID '{}'", file_id);
                            break;
                        } else {
                            #[cfg(debug_assertions)]
                            println!("[DEBUG] File ID '{}' is no longer pending after instance settled, skipping lookup", file_id);
                            return;
                        }
                    }
                }
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            }
            let response = try_request!(app.clone(), |config| {
                vrchatapi::apis::files_api::get_file(config, &file_id)
            }, { wait_for_api_ready: true }).await;
            match response {
                Ok(Some(file)) => {
                    println!("Received file metadata response for file ID '{}'", file_id);
                    let avatar_name = get_avatar_name_from_file_name(&file.name);
                    let state = app.state::<crate::memory::users::avatar::AvatarsStateMutex>();
                    let mut state = state.lock();
                    // Remove from pending_file_metadata_lookups
                    state.pending_file_metadata_lookups.retain(|id| id != &file_id);
                    if avatar_name.is_none() {
                        #[cfg(debug_assertions)]
                        eprintln!("[DEBUG] Could not extract avatar name from file name '{}'; likely not an avatar bundle", file.name);
                        return;
                    }
                    // Add to possible_avatar_files if not already there
                    if !state.possible_avatar_files.iter().any(|f| f.file_id == file_id) {
                        state.possible_avatar_files.push(crate::types::avatar::AvatarBundleFileMetadata {
                            // file.name == Avatar - AVATAR NAME - Asset Bundle - IRRELEVANT STUFF
                            file_id: file_id.clone(),
                            file_version: file.versions.last().unwrap().version.unsigned_abs(),
                            avatar_name: avatar_name,
                            author_id: file.owner_id,
                            analysis_type: {
                                if url.ends_with("/security") {
                                    crate::types::avatar::AvatarBundleAnalysisType::Security
                                } else if url.ends_with("/standard") {
                                    crate::types::avatar::AvatarBundleAnalysisType::Standard
                                } else {
                                    crate::types::avatar::AvatarBundleAnalysisType::Basic
                                }
                            }
                        });
                    } else {
                        #[cfg(debug_assertions)]
                        println!("[DEBUG] File ID '{}' is already in possible_avatar_files after metadata response! What happened!?", file_id);
                    }
                    let _ = app.emit("vrcmrd:file_metadata_lookup_complete", file_id.clone());
                },
                Ok(None) => eprintln!("File metadata request for file ID '{}' returned no data", file_id),
                Err(e) => eprintln!("Error requesting file metadata for file ID '{}': {:?}", file_id, e),
            }
        });
        return Ok(true);
    }
    Ok(false)
}

fn get_avatar_name_from_file_name(file_name: &str) -> Option<String> {
    // file_name == Avatar - AVATAR NAME - Asset bundle - IRRELEVANT STUFF
    // AVATAR NAME can contain dashes, and in theory "Asset Bundle" as well, so we need to be careful when splitting
    let trimmed = file_name.trim_start_matches("Avatar - ").split(" - Asset bundle").next()?.to_string();
    eprintln!("[DEBUG] Extracted avatar name '{}' from file name '{}'", trimmed, file_name);
    Some(trimmed)
}