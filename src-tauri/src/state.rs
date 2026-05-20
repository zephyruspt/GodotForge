use std::fs;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use crate::{
    godot::normalize_release_repositories,
    models::{HubSettings, HubState},
    paths::{config_path, default_install_path, default_project_path},
    secrets,
};

fn default_state() -> HubState {
    HubState {
        editors: Vec::new(),
        projects: Vec::new(),
        settings: HubSettings {
            default_install_path: default_install_path().to_string_lossy().to_string(),
            default_project_path: default_project_path().to_string_lossy().to_string(),
            release_repositories: Vec::new(),
            github_token: String::new(),
            github_token_configured: secrets::github_token_configured(),
            release_repository: None,
        },
    }
}

fn normalize_settings(settings: &mut HubSettings) -> Result<(), String> {
    if settings.release_repositories.is_empty() {
        settings.release_repositories = settings
            .release_repository
            .take()
            .map(|repository| vec![repository])
            .unwrap_or_default();
    }

    settings.release_repositories = normalize_release_repositories(&settings.release_repositories)?;
    let legacy_token = settings.github_token.trim().to_string();
    if !legacy_token.is_empty() {
        let _ = secrets::save_github_token(&legacy_token);
    }
    settings.github_token = String::new();
    settings.github_token_configured = secrets::github_token_configured();
    settings.release_repository = None;
    Ok(())
}

pub(crate) fn default_settings() -> HubSettings {
    default_state().settings
}

pub(crate) fn read_state() -> Result<HubState, String> {
    let path = config_path();

    if !path.exists() {
        return Ok(default_state());
    }

    let data = fs::read_to_string(path).map_err(|error| error.to_string())?;
    let mut state: HubState = serde_json::from_str(&data).map_err(|error| error.to_string())?;
    normalize_settings(&mut state.settings)?;
    Ok(state)
}

pub(crate) fn write_state(state: &HubState) -> Result<(), String> {
    let path = config_path();

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }

    let data = serde_json::to_string_pretty(state).map_err(|error| error.to_string())?;
    fs::write(&path, data).map_err(|error| error.to_string())?;
    restrict_file_permissions(&path);
    Ok(())
}

#[cfg(unix)]
fn restrict_file_permissions(path: &std::path::Path) {
    let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o600));
}

#[cfg(not(unix))]
fn restrict_file_permissions(_path: &std::path::Path) {}

#[tauri::command]
pub(crate) fn load_hub_state() -> Result<HubState, String> {
    read_state()
}
