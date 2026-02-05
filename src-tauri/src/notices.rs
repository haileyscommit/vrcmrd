use std::{ops::{Deref, DerefMut}, sync::Mutex};
use tauri::{Emitter, Manager, Wry};
use crate::{memory::advisories::AdvisoryMemory, types::advisories::Notice};

// TODO: chunk notice list, so the UI doesn't have to load them all at once when it opens the notices tab
#[tauri::command]
pub async fn get_all_notices(app: tauri::AppHandle<Wry>) -> Result<Vec<Notice>, String> {
    let notices = app
        .state::<Mutex<AdvisoryMemory>>()
        .lock()
        .map_err(|e| e.to_string())?
        .deref()
        .notices
        .clone();
    Ok(notices)
}

pub fn publish_notice(app: tauri::AppHandle<Wry>, notice: Notice) -> Result<(), String> {
    {
        // Add the notice to memory (where the UI can find it)
        let advisory_memory = app
            .state::<Mutex<AdvisoryMemory>>();
        let mut advisory_memory = advisory_memory
            .lock()
            .map_err(|e| e.to_string())?;
        advisory_memory.deref_mut().notices.push(notice.clone());
    }
    // Emit an event so the UI can react immediately (show toast, update notice list)
    app.emit("vrcmrd:notice", notice.clone()).map_err(|e| e.to_string())?;
    // TODO: send XSOverlay notification if applicable
    // TODO: send OVR Toolkit notification if applicable
    // TODO: send desktop notification if neither of the above are available
    // TODO: send TTS if applicable
    Ok(())
}