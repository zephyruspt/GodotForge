use std::fs;

use crate::{
    godot::normalize_release_repositories,
    models::{HubSettings, HubState},
    paths::{config_path, default_install_path, default_project_path},
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
    settings.github_token = settings.github_token.trim().to_string();
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
    fs::write(path, data).map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn load_hub_state() -> Result<HubState, String> {
    read_state()
}
