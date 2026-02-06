use std::{ops::DerefMut, sync::Mutex};

use tauri::{Manager, Runtime};

use crate::{
    advisories::ADVISORIES_CONFIG_KEY,
    settings::get_config,
    types::advisories::{Advisory, AdvisoryCondition, Notice},
};

pub struct AdvisoryMemory {
    pub has_group_membership_advisory: bool,
    pub active_advisories: Vec<Advisory>,
    pub all_advisories: Vec<Advisory>,
    pub notices: Vec<Notice>,
}

impl AdvisoryMemory {
    pub fn new() -> Self {
        Self {
            has_group_membership_advisory: false,
            active_advisories: Vec::new(),
            all_advisories: Vec::new(),
            notices: Vec::new(),
        }
    }
    /// Set the advisories in the struct. Automatically updates active_advisories as well.
    pub fn set(&mut self, advisories: Vec<Advisory>) {
        self.active_advisories = advisories.iter().filter(|a| a.active).cloned().collect();
        self.all_advisories = advisories.clone();
        self.has_group_membership_advisory = self
            .active_advisories
            .iter()
            .any(|a| matches!(a.condition, AdvisoryCondition::IsGroupMember(_)));
    }
}

pub fn advisory_memory_plugin<R: Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("advisory_memory")
        .setup(|app, _api| {
            let advisories = Mutex::new(AdvisoryMemory::new());
            app.manage(advisories);
            let app_clone = app.clone();
            tauri::async_runtime::spawn(async move {
                // Load advisories from config on startup
                let adv = match get_config(app_clone.clone(), ADVISORIES_CONFIG_KEY.to_string())
                    .await
                    .unwrap()
                {
                    Some(existing) => existing,
                    None => "[]".to_string(),
                };
                let adv: Vec<Advisory> = serde_json::from_str(&adv)
                    .map_err(|e| e.to_string())
                    .unwrap();
                {
                    app_clone
                        .state::<Mutex<AdvisoryMemory>>()
                        .lock()
                        .unwrap()
                        .deref_mut()
                        .set(adv);
                }
            });
            Ok(())
        })
        .build()
}
