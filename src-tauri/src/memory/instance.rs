use std::{
    ops::Deref,
    sync::{Arc, OnceLock},
};

use parking_lot::Mutex;
use tauri::{Emitter, Listener, Manager, Runtime, Wry};
use vrchatapi::models::Instance;

use crate::monitoring::instance::query_instance_info;
use crate::{memory::users::Users, types::VrcMrdInstanceId};
use std::time::Duration;

#[derive(Clone, Default)]
pub struct InstanceState {
    pub id: Option<String>,
    pub id_info: Option<VrcMrdInstanceId>,
    pub info: Option<vrchatapi::models::Instance>,
    pub settled: bool,
    /// Whether the monitor has finished reading the initial state of the file.
    /// This prevents settling instances before we've finished loading the file, since a
    /// log file may contain logs from multiple instances.
    pub isCaughtUp: bool,
}
pub type InstanceStateMutex = Mutex<InstanceState>;

pub fn instance_memory_plugin() -> tauri::plugin::TauriPlugin<Wry> {
    let listener = Arc::new(Mutex::new(None::<u32>));
    let listener2 = Arc::new(Mutex::new(None::<u32>));
    let listener3 = Arc::new(Mutex::new(None::<u32>));
    let listener4 = Arc::new(Mutex::new(None::<u32>));
    tauri::plugin::Builder::new("instance_memory")
    .setup({
        let listener = listener.clone();
        let listener2 = listener2.clone();
        let listener3 = listener3.clone();
        let listener4 = listener4.clone();
        move |app, _api| {
            let instance_state = InstanceStateMutex::new(InstanceState::default());
            app.manage::<InstanceStateMutex>(instance_state);

            // cache_refresh listener
            {
                let app_clone = app.app_handle().clone();
                let id = app.listen("vrcmrd:cache_refresh", move |_| {
                    let state = app_clone.state::<InstanceStateMutex>();
                    let state = state.lock();
                    if state.id_info.is_none() {
                        return;
                    }
                    crate::monitoring::instance::query_instance_info(app_clone.clone(), state.id_info.as_ref().unwrap());
                });
                *listener.lock() = Some(id);
            }

            // settling timer listeners (join and instance change)
            {
                let app_clone = app.app_handle().clone();
                static SETTLE_TX: OnceLock<Mutex<Option<std::sync::mpsc::Sender<()>>>> = OnceLock::new();
                let tx_mutex = SETTLE_TX.get_or_init(|| Mutex::new(None));
                let id2 = app.listen("vrcmrd:join", move |_| {
                    let mut guard = tx_mutex.lock();

                    // spawn the timer thread if not already running
                    if guard.is_none() {
                        let (tx, rx) = std::sync::mpsc::channel::<()>();
                        let app_for_thread = app_clone.clone();
                        *guard = Some(tx.clone());

                        std::thread::spawn(move || {
                            let instance_id = {
                                let state = app_for_thread.state::<InstanceStateMutex>();
                                let state = state.lock();
                                state.id.clone()
                            };
                            if instance_id.is_none() {
                                println!("Not attempting to settle instance because instance ID is not available.");
                                return;
                            }
                            loop {
                                let new_instance_id = {
                                    let state = app_for_thread.state::<InstanceStateMutex>();
                                    let state = state.lock();
                                    state.id.clone()
                                };
                                if new_instance_id.is_none() {
                                    println!("Instance ID became unavailable while waiting to settle, aborting settle process.");
                                    break;
                                }
                                let new_instance_id = new_instance_id.unwrap();
                                if new_instance_id != instance_id.clone().unwrap() {
                                    eprintln!("Instance ID changed while waiting to settle! Not settling this instance.");
                                    println!("Original instance ID: {}, new instance ID: {}", instance_id.clone().unwrap(), new_instance_id);
                                    break;
                                }
                                match rx.recv_timeout(Duration::from_secs(1)) {
                                    Ok(_) => continue, // reset timer on each join signal
                                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                                        // TODO: make sure the list isn't empty
                                        let app_for_users = app_for_thread.clone();
                                        let users = app_for_users.state::<Mutex<Users>>();
                                        let users = users.lock();
                                        if users.inner.is_empty() {
                                            println!("Not marking instance as settled yet; user list is still empty.");
                                            continue;
                                        }
                                        let is_caught_up = {
                                            let state = app_for_thread.state::<InstanceStateMutex>();
                                            let state = state.lock();
                                            state.isCaughtUp
                                        };
                                        if !is_caught_up {
                                            println!("Not marking instance as settled yet; still catching up on initial logs.");
                                            continue;
                                        }
                                        let _ = app_for_thread.emit("vrcmrd:settled", ());
                                        break;
                                    }
                                    Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
                                }
                            }
                            // clear the sender so next join will create a new thread
                            if let Some(m) = SETTLE_TX.get() {
                                let mut g = m.lock();
                                *g = None;
                            }
                        });
                    }

                    // signal the timer to restart
                    if let Some(tx) = guard.as_ref() {
                        let _ = tx.send(());
                    }
                });
                *listener2.lock() = Some(id2);
                let id4 = app.listen("vrcmrd:instance", move |_| {
                    if let Some(m) = SETTLE_TX.get() {
                        println!("Instance changed, resetting settle timer.");
                        let mut g = m.lock();
                        *g = None;
                    }
                });
                *listener4.lock() = Some(id4);
            }

            // settled listener
            {
                let app_clone = app.app_handle().clone();
                let id3 = app.listen("vrcmrd:settled", move |_| {
                    let state = app_clone.state::<InstanceStateMutex>();
                    let mut state = state.lock();
                    state.settled = true;
                    let id_info = state.id_info.clone();
                    drop(state); // Release the lock early
                    println!("Instance marked as settled.");
                    // Fetch instance info now that instance is settled
                    if let Some(info) = id_info {
                        let handle = app_clone.clone();
                        query_instance_info(handle, &info);
                    } else {
                        eprintln!("Instance ID info not available when marking instance as settled.");
                    }
                });
                *listener3.lock() = Some(id3);
            }

            Ok(())
        }
    })
    .on_drop(move |app| {
        if let Some(listener_id) = *listener.lock() {
            app.unlisten(listener_id);
        }
        if let Some(listener_id) = *listener2.lock() {
            app.unlisten(listener_id);
        }
        if let Some(listener_id) = *listener3.lock() {
            app.unlisten(listener_id);
        }
        if let Some(listener_id) = *listener4.lock() {
            app.unlisten(listener_id);
        }
    })
    .build()
}

#[tauri::command]
pub async fn get_instance_id<R: Runtime>(
    app: tauri::AppHandle<R>,
    _window: tauri::Window<R>,
) -> Result<Option<String>, String> {
    let instance_state = app.state::<InstanceStateMutex>();
    let instance_id_mutex = instance_state.lock();
    let instance_id = instance_id_mutex.deref().id.clone();
    Ok(instance_id)
}

#[tauri::command]
pub async fn get_instance_id_info<R: Runtime>(
    app: tauri::AppHandle<R>,
    _window: tauri::Window<R>,
) -> Result<Option<VrcMrdInstanceId>, String> {
    let instance_state = app.state::<InstanceStateMutex>();
    let instance_id_mutex = instance_state.lock();
    if let Some(ref instance_id_str) = instance_id_mutex.deref().id {
        let instance_info = VrcMrdInstanceId::from(instance_id_str);
        Ok(Some(instance_info))
    } else {
        Ok(None)
    }
}

// Instance info doesn't benefit from long-term caching
#[tauri::command]
pub async fn get_instance_info<R: Runtime>(
    app: tauri::AppHandle<R>,
    _window: tauri::Window<R>,
) -> Result<Option<Instance>, String> {
    let instance_state = app.state::<InstanceStateMutex>();
    let instance_id_mutex = instance_state.lock();
    if let Some(ref instance_info) = instance_id_mutex.deref().info {
        Ok(Some(instance_info.clone()))
    } else {
        Ok(None)
    }
}
