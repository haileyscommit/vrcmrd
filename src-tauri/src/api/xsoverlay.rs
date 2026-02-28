use std::{os::windows::thread, sync::{Arc, LazyLock}};

use futures_util::{stream::TryStreamExt, SinkExt};
use parking_lot::Mutex;
use reqwest::{Client, retry};
use reqwest_websocket::{Message, RequestBuilderExt};
use tauri::{Emitter, EventTarget, Wry, async_runtime::JoinHandle};

use crate::types::xsoverlay::XSOverlayCommand;

pub static XSOVERLAY_SOCKET: LazyLock<
    Arc<Mutex<Option<reqwest_websocket::WebSocket>>>,
> = LazyLock::new(|| Arc::new(Mutex::new(None)));

static XSO_QUEUED_COMMANDS: LazyLock<Arc<Mutex<Vec<XSOverlayCommand>>>> =
    LazyLock::new(|| Arc::new(Mutex::new(Vec::new())));

pub async fn start_xsoverlay_socket(
    app: tauri::AppHandle<Wry>,
) -> Result<(), Box<dyn std::error::Error>> {
    let response = Client::default()
        .get("ws://localhost:42070?client=vrcmrd") // TODO: configurable port (this is the default XSOverlay port)
        .upgrade() // Prepares the WebSocket upgrade.
        .send()
        .await?;
    let websocket = response.into_websocket().await?;
    {
        let socket_lock = XSOVERLAY_SOCKET.clone();
        let mut socket_guard = socket_lock.lock();
        *socket_guard = Some(websocket);
    }
    println!("Connected to XSOverlay WebSocket.");
    // Start listening for messages
    // let thread_handle = tauri::async_runtime::spawn(async move {
        let socket_lock = XSOVERLAY_SOCKET.clone();
        let mut retry_count: u8 = 0;
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(100-(retry_count*10).max(0) as u64)).await; // Prevent busy loop
            let mut websocket = {
                let mut socket_guard = socket_lock.lock();
                socket_guard.take()
            };

            if websocket.is_none() {
                // If the socket is not available, wait and try again
                retry_count += 1;
                if retry_count > 0 {
                    println!("XSOverlay WebSocket is not available, retrying (attempt {})...", retry_count);
                    continue;
                }
                if retry_count > 10 {
                    eprintln!("Failed to connect to XSOverlay WebSocket after {} attempts, giving up.", retry_count);
                    break;
                }
                continue;
            }
            if retry_count > 0 {
                println!("Successfully connected to XSOverlay WebSocket after {} retries.", retry_count);
            }
            retry_count = 0; // Reset retry count on successful connection

            let timeout = tokio::time::timeout(
                std::time::Duration::from_millis(100),
                websocket.as_mut().unwrap().try_next(),
            )
            .await;
            if let Some(message) = timeout.ok().and_then(|res| res.ok()).and_then(|opt| opt) {
                match message {
                    Message::Text(text) => {
                        app.emit_to(EventTarget::App, "xsoverlay-response", text.clone())
                            .unwrap();
                        println!("Received XSOverlay response: {:?}", text);
                    }
                    Message::Binary(bin) => {
                        // Ignore binary messages for now
                        println!(
                            "Received {}-byte binary message from XSOverlay, ignoring.",
                            bin.len()
                        );
                    }
                    Message::Ping(bong) => {
                        // Respond to pings
                        let _ = websocket
                            .as_mut()
                            .unwrap()
                            .send(Message::Pong(bong))
                            .await;
                        websocket.as_mut().unwrap().flush().await.unwrap();
                    }
                    Message::Pong(bong) => {
                        println!("Received pong from XSOverlay: {:?}", bong);
                        // Ignore pongs
                    }
                    Message::Close { code, reason } => {
                        eprintln!(
                            "XSOverlay WebSocket connection closed: code={:?}, reason={:?}",
                            code, reason
                        );
                        break;
                    } // _ => {
                      //     // Ignore other message types for now
                      //     println!("Received non-text message from XSOverlay, ignoring.");
                      // }
                }
            }

            let commands = {
                let mut queued_commands_guard = XSO_QUEUED_COMMANDS.lock();
                let commands = queued_commands_guard.drain(..).collect::<Vec<_>>();
                commands
            };
            let commands_empty = commands.clone().is_empty();
            for command in commands {
                let _ = websocket
                    .as_mut()
                    .unwrap()
                    .send(Message::Text(serde_json::to_string(&command).unwrap()))
                    .await
                    .map_err(|e| eprintln!("Could not send queued XSOverlay command: {:?}", e));
                let _ = {
                    let mut queued_commands_guard = XSO_QUEUED_COMMANDS.lock();
                    queued_commands_guard.retain(|c| c != &command);
                };
            }
            if !commands_empty {
                let _ = websocket
                    .as_mut()
                    .unwrap()
                    .flush()
                    .await
                    .map_err(|e| eprintln!("Could not flush XSOverlay commands: {:?}", e));
            }

            {
                let mut socket_guard = socket_lock.lock();
                *socket_guard = websocket;
            }
        }
    // });
    Ok(())
}

pub async fn with_xsoverlay_socket<F, T>(func: F) -> Result<T, Box<dyn std::error::Error>>
where
    F: FnOnce(&mut reqwest_websocket::WebSocket) -> T,
{
    let socket_lock = XSOVERLAY_SOCKET.clone();
    let mut socket_guard = socket_lock.lock();
    if let Some(socket) = socket_guard.as_mut() {
        let result = func(socket);
        Ok(result)
    } else {
        Err("XSOverlay WebSocket is not connected.".into())
    }
}

/// Adds an XSOverlay command to the queue to be sent on the next available opportunity. 
/// This is used to send commands from other threads without needing to worry about locking the socket directly.
pub fn queue_xsoverlay_command(command: XSOverlayCommand) {
    XSO_QUEUED_COMMANDS.lock().push(command);
}