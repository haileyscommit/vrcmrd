use std::sync::{Arc, LazyLock};

use futures_util::{stream::TryStreamExt, SinkExt};
use parking_lot::Mutex;
use reqwest::Client;
use reqwest_websocket::{Message, RequestBuilderExt};
use tauri::{Emitter, EventTarget, Wry};

pub static XSOVERLAY_SOCKET: LazyLock<
    Arc<Mutex<Option<reqwest_websocket::WebSocket>>>,
> = LazyLock::new(|| Arc::new(Mutex::new(None)));

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
    tauri::async_runtime::spawn(async move {
        let socket_lock = XSOVERLAY_SOCKET.clone();
        // let mut retry_count = 0;
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await; // Prevent busy loop
            let mut websocket = {
                let mut socket_guard = socket_lock.lock();
                socket_guard.take()
            };

            if websocket.is_none() {
                break;
                // // If the socket is not available, wait and try again
                // println!("XSOverlay WebSocket is not available, retrying...");
                // retry_count += 1;
                // if retry_count > 10 {
                //     eprintln!("Failed to connect to XSOverlay WebSocket after {} attempts, giving up.", retry_count);
                //     break;
                // }
                // continue;
            }
            // retry_count = 0; // Reset retry count on successful connection

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

            {
                let mut socket_guard = socket_lock.lock();
                *socket_guard = websocket;
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
    let mut socket_guard = socket_lock.lock();
    if let Some(socket) = socket_guard.as_mut() {
        let result = func(socket);
        Ok(result)
    } else {
        Err("XSOverlay WebSocket is not connected.".into())
    }
}
