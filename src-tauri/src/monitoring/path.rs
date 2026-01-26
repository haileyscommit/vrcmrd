use std::{fs::File, path::PathBuf};

pub fn get_monitor_path(path: &mut PathBuf) -> Option<bool> {
    let scan_dir = if path.is_dir() {
        path.clone()
    } else if let Some(parent) = path.parent() {
        parent.to_path_buf()
    } else {
        path.clone()
    };

    if let Ok(entries) = std::fs::read_dir(&scan_dir) {
        let mut latest_path: Option<PathBuf> = None;
        let mut latest_time: Option<std::time::SystemTime> = None;

        for entry in entries.filter_map(|e| e.ok()) {
            let p = entry.path();
            if !p.is_file() {
                continue;
            }
            let name = match p.file_name().and_then(|s| s.to_str()) {
                Some(n) => n,
                None => continue,
            };
            if !(name.starts_with("output_log_") && name.ends_with(".txt")) {
                continue;
            }

            if let Ok(meta) = entry.metadata() {
                // Prefer creation time, fall back to modified time
                let maybe_time = meta.created().or_else(|_| meta.modified());
                if let Ok(t) = maybe_time {
                    let take = match latest_time {
                        Some(cur) => t > cur,
                        None => true,
                    };
                    if take {
                        latest_time = Some(t);
                        latest_path = Some(p);
                    }
                } else if latest_path.is_none() {
                    // If timestamps aren't available, pick the first matching file
                    latest_path = Some(p);
                }
            }
        }

        if let Some(new_path) = latest_path {
            let current_name = path.file_name().and_then(|s| s.to_str()).map(|s| s.to_string());
            let latest_name = new_path.file_name().and_then(|s| s.to_str()).map(|s| s.to_string());
            if current_name.as_deref() != latest_name.as_deref() {
                eprintln!("Switching to latest log file: {:?}", new_path);
                *path = new_path;
                return Some(true);
            }
        }
        
        // let mut latest_name: Option<String> = None;
        // let mut latest_path: Option<PathBuf> = None;

        // for entry in entries.filter_map(|e| e.ok()) {
        //     let p = entry.path();
        //     if let Some(name) = p.file_name().and_then(|s| s.to_str()) {
        //         if name.starts_with("output_log_") && name.ends_with(".txt") {
        //             match &latest_name {
        //                 Some(cur) => {
        //                     if name.len() > cur.len() {
        //                         latest_name = Some(name.to_string());
        //                         latest_path = Some(p);
        //                     }
        //                 }
        //                 None => {
        //                     latest_name = Some(name.to_string());
        //                     latest_path = Some(p);
        //                 }
        //             }
        //         }
        //     }
        // }

        // if let Some(new_path) = latest_path {
        //     let current_name = path.file_name().and_then(|s| s.to_str()).map(|s| s.to_string());
        //     if current_name.as_deref() != latest_name.as_deref() {
        //         eprintln!("Switching to latest log file: {:?}", new_path);
        //         *path = new_path;
        //         return Some(true);
        //     }
        // }
        None
    } else {
        eprintln!("Failed to read directory {:?}", scan_dir);
        None
    }
}