use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{models::ActivityLogEntry, paths::activity_log_path};

pub(crate) fn current_unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

fn read_activity_log_file() -> Vec<ActivityLogEntry> {
    fs::read_to_string(activity_log_path())
        .ok()
        .and_then(|data| serde_json::from_str(&data).ok())
        .unwrap_or_default()
}

pub(crate) fn record_activity(level: &str, message: impl Into<String>) {
    let mut entries = read_activity_log_file();
    entries.push(ActivityLogEntry {
        timestamp: current_unix_seconds(),
        level: level.into(),
        message: message.into(),
    });

    if entries.len() > 80 {
        entries = entries.split_off(entries.len() - 80);
    }

    let path = activity_log_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(data) = serde_json::to_string_pretty(&entries) {
        let _ = fs::write(path, data);
    }
}

#[tauri::command]
pub(crate) fn read_activity_log() -> Vec<ActivityLogEntry> {
    let mut entries = read_activity_log_file();
    entries.reverse();
    entries
}
