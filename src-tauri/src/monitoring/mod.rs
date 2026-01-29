pub mod instance;
mod avatars;
mod join_leave;
mod path;

use std::{
    env,
    fs::File,
    io::{Read, Seek},
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver},
        Arc,
    },
    thread::{self, JoinHandle},
    time::Duration,
    vec,
};

use tauri::{
    plugin::{Builder, TauriPlugin},
    EventLoopMessage, Manager, Runtime, Wry,
};

use crate::monitoring::path::get_monitor_path;

pub struct VrcLogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
}

/// Handle for stopping the background monitor thread.
pub struct MonitorHandle {
    stop_flag: Arc<AtomicBool>,
    join: Option<JoinHandle<()>>,
}

impl MonitorHandle {
    /// Signal the thread to stop and wait for it to finish.
    pub fn stop(mut self) {
        self.stop_flag.store(true, Ordering::SeqCst);
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
    }
}

fn is_timestamped_line(b: &[u8]) -> bool {
    if b.len() < 34 {
        false
    } else {
        // Quick checks for separators and digit presence
        let sep_ok = b[4] == b'.'
            && b[7] == b'.'
            && b[10] == b' '
            && b[13] == b':'
            && b[16] == b':'
            && b[19] == b' ';
        let digits_ok = b[0].is_ascii_digit()
            && b[1].is_ascii_digit()
            && b[2].is_ascii_digit()
            && b[3].is_ascii_digit()
            && b[5].is_ascii_digit()
            && b[6].is_ascii_digit()
            && b[8].is_ascii_digit()
            && b[9].is_ascii_digit()
            && b[11].is_ascii_digit()
            && b[12].is_ascii_digit()
            && b[14].is_ascii_digit()
            && b[15].is_ascii_digit()
            && b[17].is_ascii_digit()
            && b[18].is_ascii_digit();
        if !sep_ok || !digits_ok {
            false
        } else {
            // Parse numeric fields and check ranges
            let month = ((b[5] - b'0') as u32) * 10 + ((b[6] - b'0') as u32);
            let day = ((b[8] - b'0') as u32) * 10 + ((b[9] - b'0') as u32);
            let hour = ((b[11] - b'0') as u32) * 10 + ((b[12] - b'0') as u32);
            let minute = ((b[14] - b'0') as u32) * 10 + ((b[15] - b'0') as u32);
            let second = ((b[17] - b'0') as u32) * 10 + ((b[18] - b'0') as u32);
            let ranges_ok = (1..=12).contains(&month)
                && (1..=31).contains(&day)
                && hour <= 23
                && minute <= 59
                && second <= 59;
            if !ranges_ok {
                false
            } else {
                // Ensure a '-' exists in the expected area (log level / separator zone)
                if let Some(pos) = b.iter().position(|&c| c == b'-') {
                    // Expect '-' to be roughly after the level area (allow some slack)
                    (30..=40).contains(&pos) && pos > 0 && b[pos - 1] == b' '
                } else {
                    false
                }
            }
        }
    }
}

/// Start monitoring `path` on a background thread. Returns a receiver of events and a handle
/// to stop the monitor. The monitor polls the file every `interval`.
fn start_logfile_monitor(
    path: impl Into<PathBuf>,
    interval: Duration,
) -> (Receiver<VrcLogEntry>, MonitorHandle) {
    // this is `mut` so that it can be updated if a new log file appears
    #[allow(unused_mut)]
    let mut path = path.into();
    let (tx, rx) = mpsc::channel();
    let stop_flag = Arc::new(AtomicBool::new(false));
    let thread_stop = stop_flag.clone();

    let join = thread::spawn(move || {
        let mut last_len: u64 = 0;

        while !thread_stop.load(Ordering::SeqCst) {
            // Check for new rotated log files in the same directory and switch to the newest.
            // Make sure we operate on the directory: if `path` is a file, use its parent directory.
            if let Some(success) = get_monitor_path(&mut path) {
                if success {
                    // Reset read offset so we process the new file from start.
                    last_len = 0;
                }
            }
            let result = File::open(&path);
            if result.is_err() {
                // File might not exist yet
                eprintln!("Failed to open file: {:?}", result.err());
                continue;
            }
            let mut file = result.unwrap();

            // Read new data from the file
            let mut new_bytes: Vec<u8> = vec![];
            match file.seek(std::io::SeekFrom::Start(last_len)) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Failed to seek file: {:?}", e);
                    continue;
                }
            }
            match file.read_to_end(&mut new_bytes) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Failed to read file: {:?}", e);
                    continue;
                }
            }
            if !new_bytes.is_empty() {
                // Convert bytes to string lines
                // TODO: support multi-line entries (which don't start with a timestamp, level, and `-`)
                // Example line format:
                // 2026.01.24 01:53:54 Warning    -  Some warning message here
                // (everything before - has the same length)
                // Split new_bytes into lines
                let lines = new_bytes.split(|&b| b == b'\n').collect::<Vec<&[u8]>>();
                for (i, line) in lines.iter().enumerate() {
                    // Complete "line" by joining with next line repeatedly if not last line and the next line is not a timestamped line
                    let line = if i + 1 < lines.len() {
                        let mut line = line.to_vec();
                        let mut j = i + 1;
                        while j < lines.len() && !is_timestamped_line(lines[j]) {
                            // Append this line to the previous
                            let mut combined = line.to_vec();
                            combined.push(b'\n');
                            combined.extend_from_slice(lines[j]);
                            line = combined;
                            j += 1;
                        }
                        line
                    } else {
                        line.to_vec()
                    };
                    if line.len() < 34 {
                        continue;
                    }
                    let timestamp_str = String::from_utf8_lossy(&line[..19])
                        .to_string()
                        .trim()
                        .to_string();
                    let level_str = String::from_utf8_lossy(&line[20..30])
                        .to_string()
                        .trim()
                        .to_string();
                    let message_str = String::from_utf8_lossy(&line[34..])
                        .to_string()
                        .trim()
                        .to_string();
                    //eprintln!("Processing line: {:?}, {:?}, {:?}", timestamp_str, level_str, message_str);
                    let log_entry = VrcLogEntry {
                        timestamp: timestamp_str,
                        level: level_str,
                        message: message_str,
                    };
                    if tx.send(log_entry).is_err() {
                        eprintln!("Receiver has been dropped, stopping monitor.");
                        return;
                    }
                }
                //tx.send(vec![...]).ok();
            }

            // Get the new length after reading
            last_len = file.metadata().map(|m| m.len()).unwrap_or(last_len);

            thread::sleep(interval);
        }

        // When stopping, drop the sender so receiver gets an end-of-stream.
        drop(tx);
    });

    (
        rx,
        MonitorHandle {
            stop_flag,
            join: Some(join),
        },
    )
}

pub fn start_monitoring_logfiles(app: tauri::AppHandle) {
    // Example usage
    let file_to_watch =
        env::var("APPDATA").unwrap_or_default() + "\\..\\LocalLow\\VRChat\\VRChat\\";
    let (rx, handle) = start_logfile_monitor(file_to_watch, Duration::from_millis(200));

    // Spawn a thread to print events (main thread could also handle them).
    let printer = thread::spawn(move || {
        println!("Monitoring VRChat logs.");
        for evt in rx {
            //println!("{:?}", evt);
            match instance::handle_joined_instance(app.clone(), &evt) {
                Ok(true) => continue, // handled
                Ok(false) => {}
                Err(e) => eprintln!("Error handling joined instance: {:?}", e),
            }
            match avatars::handle_switched_avatar(app.clone(), &evt) {
                Ok(true) => continue, // handled
                Ok(false) => {}
                Err(e) => eprintln!("Error handling switched avatar: {:?}", e),
            }
            match join_leave::handle_join_leave(app.clone(), &evt) {
                Ok(true) => continue, // handled
                Ok(false) => {}
                Err(e) => eprintln!("Error handling join/leave: {:?}", e),
            }
        }
        println!("Monitor stopped and channel closed.");
    });

    // Let it run for 10 seconds in this example, then stop.
    // thread::sleep(Duration::from_secs(10));
    // handle.stop();

    //let _ = printer.join();
}

pub fn monitoring_plugin() -> TauriPlugin<Wry> {
    Builder::new("vrc-logfile-monitor")
        .setup(|app, _api| {
            start_monitoring_logfiles(app.clone());
            Ok(())
        })
        .build()
}
