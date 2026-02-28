use crate::{
    api::xsoverlay::{XSO_CONNECTED, queue_xsoverlay_command}, memory::advisories::AdvisoryMemory, settings::get_config, types::{
        advisories::{AdvisoryLevel, Notice},
        xsoverlay::{XSOverlayCommand, XSOverlayNotificationObject},
    }
};
use parking_lot::Mutex;
use tauri_plugin_tts::{SpeakRequest, TtsExt};
use std::{
    ops::{Deref, DerefMut}
};
use tauri::{Emitter, Manager, Wry};
#[cfg(target_os = "windows")]
use tauri_winrt_notification::Toast;

// TODO: chunk notice list, so the UI doesn't have to load them all at once when it opens the notices tab
#[tauri::command]
pub async fn get_all_notices(app: tauri::AppHandle<Wry>) -> Result<Vec<Notice>, String> {
    let notices = app
        .state::<Mutex<AdvisoryMemory>>()
        .lock()
        .deref()
        .notices
        .clone();
    Ok(notices)
}

pub fn publish_notice(app: tauri::AppHandle<Wry>, notice: Notice) -> Result<(), String> {
    {
        // Add the notice to memory (where the UI can find it)
        let advisory_memory = app.state::<Mutex<AdvisoryMemory>>();
        let mut advisory_memory = advisory_memory.lock();
        advisory_memory.deref_mut().notices.push(notice.clone());
    }
    // Emit an event so the UI can react immediately (show toast, update notice list)
    app.emit("vrcmrd:notice", notice.clone())
        .map_err(|e| e.to_string())?;
    // TODO: 1 and 2 are different in join mode: 1 shows for just the host (default), 2 shows for everyone.
    // The UI will only expose 1 and 2 for TTS, not notifications.
    
    let settled_for_user = {
        let notice = notice.clone();
        let users_state = app.state::<Mutex<crate::memory::users::Users>>();
        let users_state = users_state.try_lock_for(std::time::Duration::from_secs(2));
        if let Some(users_state) = users_state {
            if notice.relevant_user_id.is_some() && users_state.joined_before_settled.contains(&notice.clone().relevant_user_id.unwrap()) {
                println!("Skipping notification for user {} because they joined before me", notice.relevant_user_id.unwrap());
                drop(users_state);
                false
            } else {
                drop(users_state);
                true
            }
        } else {
            false
        }
    };

    if notice.send_notification && settled_for_user {
        let notice = notice.clone();
        let app = app.clone();
        tauri::async_runtime::spawn(async move {
            let notif_preference = get_config(app.clone(), "notification_preference".to_string())
                .await
                .unwrap()
                .unwrap_or("1".to_string())
                .parse::<u8>()
                .unwrap_or(1);
            if notif_preference == 0 {
                return;
            }
            println!("Sending notification for notice with title: {}", notice.title.clone().unwrap_or("No title".to_string()));
            // TODO: if notif_preference == 1 && inJoinMode { show to host only } else if notif_preference == 2 { always show me notifications }
            let socket_ready = {
                let connected = XSO_CONNECTED.lock();
                *connected
            };
            if socket_ready {
                println!("Sending XSOverlay notification");
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
                queue_xsoverlay_command(command);
                // let _ = socket
                //     .as_mut()
                //     .unwrap()
                //     .send(reqwest_websocket::Message::Text(
                //         serde_json::to_string(&command).unwrap(),
                //     ))
                //     .await
                //     .map_err(|e| eprintln!("Could not send XSOverlay notification: {:?}", e));
                // let _ = socket
                //     .as_mut()
                //     .unwrap()
                //     .flush()
                //     .await
                //     .map_err(|e| eprintln!("Could not flush XSOverlay notifications: {:?}", e));
                // {
                //     let mut socket_guard = XSOVERLAY_SOCKET.lock();
                //     *socket_guard = socket;
                // }
                // #[cfg(debug_assertions)]
                // println!("[DEBUG] Sent!");
                return;
            } else {
                // TODO: send OVR Toolkit notification if applicable
                // TODO: send desktop notification if neither of the above are available
                println!("Sending desktop notification");
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
    if notice.send_tts && settled_for_user {
        let notice = notice.clone();
        tauri::async_runtime::spawn(async move {
            let tts_preference = get_config(app.clone(), "tts_preference".to_string())
                .await.unwrap()
                .unwrap_or("1".to_string())
                .parse::<u8>()
                .unwrap_or(1);
            if tts_preference == 0 {
                return;
            }
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
            }).map_err(|e| e.to_string()).unwrap();
        });
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
