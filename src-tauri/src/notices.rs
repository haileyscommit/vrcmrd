use crate::{
    api::xsoverlay::XSOVERLAY_SOCKET,
    memory::advisories::AdvisoryMemory,
    types::{
        advisories::{AdvisoryLevel, Notice},
        xsoverlay::{XSOverlayCommand, XSOverlayNotificationObject},
    },
};
use futures_util::{FutureExt, SinkExt};
use notify_rust::{Hint, Notification};
use regex::Split;
use tauri_plugin_tts::{GetVoicesRequest, SpeakRequest, TtsExt};
use std::{
    fmt::format, ops::{Deref, DerefMut}, sync::Mutex
};
use tauri::{Emitter, Manager, Wry};
#[cfg(target_os = "windows")]
use tauri_winrt_notification::{Scenario, Toast};

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
        let advisory_memory = app.state::<Mutex<AdvisoryMemory>>();
        let mut advisory_memory = advisory_memory.lock().map_err(|e| e.to_string())?;
        advisory_memory.deref_mut().notices.push(notice.clone());
    }
    // Emit an event so the UI can react immediately (show toast, update notice list)
    app.emit("vrcmrd:notice", notice.clone())
        .map_err(|e| e.to_string())?;
    // TODO: don't spawn the send-notification task if the notification-settle hasn't been reached
    if notice.send_notification {
        let notice = notice.clone();
        let app = app.clone();
        tauri::async_runtime::spawn(async move {
            let mut socket_guard = XSOVERLAY_SOCKET.lock().await;
            println!("Sending XSOverlay notification");
            if socket_guard.is_some() {
                let notification = XSOverlayNotificationObject {
                    title: notice.title.clone(),
                    content: Some(notice.message.clone()),
                    timeout: Some(3.0 + (notice.message.len() as f32 * 0.065)),
                    height: Some(75.0 + (wrapped_lines_count(&notice.message) as f32 * 25.0)),
                    audio_path: Some("default".to_string()),
                    source_app: Some("vrcmrd".to_string()),
                    ..Default::default()
                };
                let command = XSOverlayCommand {
                    sender: "vrcmrd".to_string(),
                    target: "xsoverlay".to_string(),
                    command: XSOverlayNotificationObject::COMMAND.to_string(),
                    json_data: serde_json::to_string(&notification).unwrap(),
                    raw_data: "".to_string(),
                };
                // #[cfg(debug_assertions)]
                // println!("[DEBUG] Sending XSOverlay notification: {:?}", serde_json::to_string(&command).unwrap());
                let _ = socket_guard
                    .as_mut()
                    .unwrap()
                    .send(reqwest_websocket::Message::Text(
                        serde_json::to_string(&command).unwrap(),
                    ))
                    .await
                    .map_err(|e| eprintln!("Could not send XSOverlay notification: {:?}", e));
                let _ = socket_guard
                    .as_mut()
                    .unwrap()
                    .flush()
                    .await
                    .map_err(|e| eprintln!("Could not flush XSOverlay notifications: {:?}", e));
                // #[cfg(debug_assertions)]
                // println!("[DEBUG] Sent!");
                return;
            } else {
                drop(socket_guard);
                // TODO: send OVR Toolkit notification if applicable
                // TODO: send desktop notification if neither of the above are available
                // Desktop notification
                #[cfg(not(target_os = "windows"))]
                Notification::new()
                    .summary(&notice.title.unwrap_or("New notice".to_owned()))
                    .body(&notice.message)
                    .auto_icon()
                    .appname("VRCMRD")
                    .show()
                    .map_err(|e| eprintln!("Could not send desktop notification: {:?}", e))
                    .ok();
                #[cfg(target_os = "windows")]
                Toast::new(Toast::POWERSHELL_APP_ID)
                    .title(&notice.title.unwrap_or("New notice".to_owned()))
                    .text1(&notice.message)
                    .duration(tauri_winrt_notification::Duration::Short)
                    .show()
                    .map_err(|e| eprintln!("Could not send Windows toast notification: {:?}", e))
                    .ok();
            }
            // TODO: send TTS if applicable (TTS has its own thread, so dispatch to that)
        });
    }
    if notice.send_tts {
        let notice = notice.clone();
        let tts = app.tts();
        // let voices = tts.get_voices(GetVoicesRequest {
        //     language: Some("en_US".to_string())
        // }).map_err(|e| e.to_string())?;
        // TODO: configurable voice selection

        tts.speak(SpeakRequest {
            text: format!("{}. {}.", notice.title.unwrap_or(String::new()), notice.message).to_string(),
            voice_id: None,
            rate: 0.9,
            language: Some("en_US".to_string()),
            volume: 1.0,
            pitch: 1.0,
            queue_mode: match notice.level {
                AdvisoryLevel::Maximum => tauri_plugin_tts::QueueMode::Flush,
                _ => tauri_plugin_tts::QueueMode::Add,
            }
        }).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn wrapped_lines_count(s: &str) -> usize {
    let max_line_length = 50;
    s.lines()
        .map(|line| {
            let mut chunks: Vec<&str> = vec![];
            let mut line = line;
            while line.len() > max_line_length {
                let split = line.split_at_checked(max_line_length);
                if let Some((a, b)) = split {
                    chunks.push(a);
                    line = b.trim_start();
                } else {
                    break;
                }
            }
            if !line.is_empty() {
                chunks.push(line);
            }
            chunks.len()
        })
        .sum()
}
