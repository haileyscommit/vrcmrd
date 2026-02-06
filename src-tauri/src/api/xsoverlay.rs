use std::sync::{Arc, LazyLock};

use futures_util::{stream::TryStreamExt, SinkExt};
use reqwest::Client;
use reqwest_websocket::{Message, RequestBuilderExt};
use tauri::{Emitter, EventTarget, Wry};

pub static XSOVERLAY_SOCKET: LazyLock<
    Arc<tokio::sync::Mutex<Option<reqwest_websocket::WebSocket>>>,
> = LazyLock::new(|| Arc::new(tokio::sync::Mutex::new(None)));

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
        let mut socket_guard = socket_lock.lock().await;
        *socket_guard = Some(websocket);
    }
    println!("Connected to XSOverlay WebSocket.");
    // Start listening for messages
    tauri::async_runtime::spawn(async move {
        let socket_lock = XSOVERLAY_SOCKET.clone();
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await; // Prevent busy loop
            let timeout_result =
                tokio::time::timeout(std::time::Duration::from_millis(500), socket_lock.lock())
                    .await;

            let mut socket_guard = match timeout_result {
                Ok(guard) => guard,
                Err(_) => {
                    static LOCK_TIMEOUT_ATTEMPTS: std::sync::atomic::AtomicU32 =
                        std::sync::atomic::AtomicU32::new(0);
                    let attempts =
                        LOCK_TIMEOUT_ATTEMPTS.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
                    if attempts >= 10 {
                        eprintln!("XSOverlay WebSocket lock timeout threshold exceeded (10 attempts). Stopping socket.");
                        break;
                    }
                    continue;
                }
            };
            let timeout = tokio::time::timeout(
                std::time::Duration::from_millis(100),
                socket_guard.as_mut().unwrap().try_next(),
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
                        let _ = socket_guard
                            .as_mut()
                            .unwrap()
                            .send(Message::Pong(bong))
                            .await;
                        socket_guard.as_mut().unwrap().flush().await.unwrap();
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
                        *socket_guard = None;
                        break;
                    } // _ => {
                      //     // Ignore other message types for now
                      //     println!("Received non-text message from XSOverlay, ignoring.");
                      // }
                }
            }
        }
    });
    Ok(())
}

pub async fn with_xsoverlay_socket<F, T>(func: F) -> Result<T, Box<dyn std::error::Error>>
where
    F: FnOnce(&mut reqwest_websocket::WebSocket) -> T,
{
    let socket_lock = XSOVERLAY_SOCKET.clone();
    let mut socket_guard = socket_lock.lock().await;
    if let Some(socket) = socket_guard.as_mut() {
        let result = func(socket);
        Ok(result)
    } else {
        Err("XSOverlay WebSocket is not connected.".into())
    }
}
