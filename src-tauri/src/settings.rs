use std::path::PathBuf;

use crate::{
    activity::record_activity,
    filesystem::migrate_registered_paths,
    godot::normalize_release_repositories,
    models::{HubState, UpdateSettingsRequest},
    secrets,
    state::{default_settings, read_state, write_state},
};

#[tauri::command]
pub(crate) fn save_settings(request: UpdateSettingsRequest) -> Result<HubState, String> {
    let mut state = read_state()?;
    let old_install_root = PathBuf::from(&state.settings.default_install_path);
    let old_project_root = PathBuf::from(&state.settings.default_project_path);
    let new_install_path = request.default_install_path.trim();
    let new_project_path = request.default_project_path.trim();

    if new_install_path.is_empty() || new_project_path.is_empty() {
        return Err("Default editor and project paths are required.".into());
    }

    let new_install_root = PathBuf::from(new_install_path);
    let new_project_root = PathBuf::from(new_project_path);

    if request.migrate_existing_paths {
        migrate_registered_paths(
            &mut state,
            &old_install_root,
            &new_install_root,
            &old_project_root,
            &new_project_root,
        )?;
    }

    state.settings.default_install_path = new_install_root.to_string_lossy().to_string();
    state.settings.default_project_path = new_project_root.to_string_lossy().to_string();
    state.settings.release_repositories =
        normalize_release_repositories(&request.release_repositories)?;
    let github_token = request.github_token.trim();
    if !github_token.is_empty() {
        secrets::save_github_token(github_token)?;
    } else if request.clear_github_token {
        secrets::clear_github_token()?;
    }
    state.settings.github_token = String::new();
    state.settings.github_token_configured = secrets::github_token_configured();
    state.settings.release_repository = None;

    write_state(&state)?;
    record_activity("info", "Workspace settings saved.");
    Ok(state)
}

#[tauri::command]
pub(crate) fn restore_default_settings() -> Result<HubState, String> {
    let mut state = read_state()?;
    let _ = secrets::clear_github_token();
    state.settings = default_settings();
    state.settings.github_token_configured = secrets::github_token_configured();

    write_state(&state)?;
    record_activity("info", "Workspace settings restored to defaults.");
    Ok(state)
}
