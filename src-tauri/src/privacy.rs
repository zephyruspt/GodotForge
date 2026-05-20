use std::fs;

use crate::{
    activity::record_activity,
    models::{HubState, PrivacyReport},
    paths::{activity_log_path, config_dir},
    releases::{clear_release_cache_files, release_cache_info_for, RELEASE_CATALOG_PAGE_SIZE},
    state::read_state,
};

fn sanitized_state(mut state: HubState) -> HubState {
    if !state.settings.github_token.is_empty() {
        state.settings.github_token = "[configured redacted]".into();
    }
    state.settings.github_token_configured = crate::secrets::github_token_configured();
    state
}

#[tauri::command]
pub(crate) fn export_privacy_report() -> Result<PrivacyReport, String> {
    let state = read_state()?;
    let repositories = crate::releases::current_release_repositories(&state.settings)?;
    let activity_entries = fs::read_to_string(activity_log_path())
        .ok()
        .and_then(|data| serde_json::from_str(&data).ok())
        .unwrap_or_default();

    Ok(PrivacyReport {
        app_data_dir: config_dir().to_string_lossy().to_string(),
        state: sanitized_state(state),
        release_cache: release_cache_info_for(&repositories, RELEASE_CATALOG_PAGE_SIZE, 1),
        activity_entries,
        notes: vec![
            "GitHub tokens are redacted from this report.".into(),
            "Project paths, editor paths, repository URLs, and activity messages may still identify local workspaces.".into(),
            "Godot Forge stores data locally and does not sell or share personal data.".into(),
        ],
    })
}

#[tauri::command]
pub(crate) fn clear_auxiliary_privacy_data() -> Result<PrivacyReport, String> {
    let _ = fs::remove_file(activity_log_path());
    clear_release_cache_files();
    record_activity("info", "Auxiliary privacy data cleared.");
    export_privacy_report()
}
