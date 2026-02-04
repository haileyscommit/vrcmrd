use std::ops::{Deref, DerefMut};
use std::sync::Mutex;

use nid::Nanoid;
use nid::alphabet::Base58Alphabet;
use tauri::{Emitter, Manager, Wry};

use crate::memory::advisories::AdvisoryMemory;
use crate::types::advisories::{Advisory};
use crate::settings::{get_config, update_config};

pub const ADVISORIES_CONFIG_KEY: &str = "my_advisories";

#[tauri::command]
pub async fn generate_advisory_id() -> String {
    format!("vrcmrd_adv_{}", Nanoid::<12, Base58Alphabet>::new().to_string())
}

#[tauri::command]
pub async fn add_advisory(app: tauri::AppHandle<Wry>, advisory: Advisory) -> Result<(), String> {
    let adv = match get_config(app.clone(), ADVISORIES_CONFIG_KEY.to_string()).await? {
        Some(existing) => existing,
        None => "[]".to_string(),
    };
    let mut adv = serde_json::from_str(&adv).unwrap_or_else(|_| Vec::new());
    if let Some(_) = adv.iter().position(|v: &Advisory| v.id == advisory.id) {
        // Can't use this command to update existing advisories
        return Err("Advisory with this ID already exists".to_string());
    }
    // TODO: validate advisory, i.e. ensure no recursive Not conditions, log line must be by itself, etc.
    adv.push(advisory);
    {
        app.state::<Mutex<AdvisoryMemory>>()
            .lock()
            .unwrap()
            .deref_mut()
            .set(adv.clone());
    }
    let adv = serde_json::to_string(&adv).map_err(|e| e.to_string())?;
    update_config(app.clone(), ADVISORIES_CONFIG_KEY.to_string(), adv).await?;
    app.emit("vrcmrd:advisories_updated", {}).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn get_advisories(app: tauri::AppHandle<Wry>) -> Result<Vec<Advisory>, String> {
    // gonna move that to the memory plugin later
    // let adv = match get_config(app.clone(), ADVISORIES_CONFIG_KEY.to_string()).await? {
    //     Some(existing) => existing,
    //     None => "[]".to_string(),
    // };
    //let adv: Vec<Advisory> = serde_json::from_str(&adv).map_err(|e| e.to_string())?;
    let advisory_memory = app.state::<Mutex<AdvisoryMemory>>();
    let advisory_memory = advisory_memory.lock().unwrap();
    return Ok(advisory_memory.deref().all_advisories.clone());
    //Ok(adv)
}

pub async fn get_active_advisories(app: tauri::AppHandle<Wry>) -> Result<Vec<Advisory>, String> {
    let advisory_memory = app.state::<Mutex<AdvisoryMemory>>();
    let advisory_memory = advisory_memory.lock().unwrap();
    return Ok(advisory_memory.deref().active_advisories.clone());
}

#[tauri::command]
pub async fn get_advisory(app: tauri::AppHandle<Wry>, advisory_id: &str) -> Result<Option<Advisory>, String> {
    // let adv = match get_config(app.clone(), ADVISORIES_CONFIG_KEY.to_string()).await? {
    //     Some(existing) => existing,
    //     None => "[]".to_string(),
    // };
    // let adv: Vec<Advisory> = serde_json::from_str(&adv).map_err(|e| e.to_string())?;
    // Ok(adv.into_iter().find(|a| a.id == advisory_id))
    let advisory_memory = app.state::<Mutex<AdvisoryMemory>>();
    let advisory_memory = advisory_memory.lock().unwrap();
    Ok(advisory_memory.deref().all_advisories.iter().find(|a| a.id == advisory_id).cloned())
}

#[tauri::command]
pub async fn remove_advisory(app: tauri::AppHandle<Wry>, advisory_id: &str) -> Result<(), String> {
    let adv = match get_config(app.clone(), ADVISORIES_CONFIG_KEY.to_string()).await? {
        Some(existing) => existing,
        None => "[]".to_string(),
    };
    let mut adv: Vec<Advisory> = serde_json::from_str(&adv).map_err(|e| e.to_string())?;
    if let Some(pos) = adv.iter().position(|v: &Advisory| v.id == advisory_id) {
        adv.remove(pos);
    } else {
        return Err("Advisory with this ID does not exist".to_string());
    }
    {
        app.state::<Mutex<AdvisoryMemory>>()
            .lock()
            .unwrap()
            .deref_mut()
            .set(adv.clone());
    }
    let adv = serde_json::to_string(&adv).map_err(|e| e.to_string())?;
    update_config(app.clone(), ADVISORIES_CONFIG_KEY.to_string(), adv).await?;
    app.emit("vrcmrd:advisories_updated", {}).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn update_advisory(app: tauri::AppHandle<Wry>, advisory: Advisory) -> Result<(), String> {
    let adv = match get_config(app.clone(), ADVISORIES_CONFIG_KEY.to_string()).await? {
        Some(existing) => existing,
        None => "[]".to_string(),
    };
    let mut adv: Vec<Advisory> = serde_json::from_str(&adv).map_err(|e| e.to_string())?;
    if let Some(pos) = adv.iter().position(|v: &Advisory| v.id == advisory.id) {
        adv[pos] = advisory;
    } else {
        return Err("Advisory with this ID does not exist".to_string());
    }
    {
        app.state::<Mutex<AdvisoryMemory>>()
            .lock()
            .unwrap()
            .deref_mut()
            .set(adv.clone());
    }
    let adv = serde_json::to_string(&adv).map_err(|e| e.to_string())?;
    update_config(app.clone(), ADVISORIES_CONFIG_KEY.to_string(), adv).await?;
    app.emit("vrcmrd:advisories_updated", {}).map_err(|e| e.to_string())?;
    Ok(())
}