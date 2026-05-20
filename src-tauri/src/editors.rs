use std::{fs, path::PathBuf};

use crate::{
    activity::record_activity,
    godot::{
        architecture_from_asset, editor_name_from_asset, extract_zip, find_godot_executable,
        mark_executable, safe_folder_name, version_from_tag,
    },
    models::{DownloadEditorRequest, GodotEditor, HubState},
    paths::now_id,
    releases::http_client,
    state::{read_state, write_state},
};

#[tauri::command]
pub(crate) fn download_godot_editor(request: DownloadEditorRequest) -> Result<HubState, String> {
    let mut state = read_state()?;
    let install_root = request
        .install_path
        .filter(|path| !path.trim().is_empty())
        .unwrap_or_else(|| state.settings.default_install_path.clone());
    let install_root = PathBuf::from(install_root);
    let source_folder = request
        .release_repository
        .as_deref()
        .map(safe_folder_name)
        .unwrap_or_else(|| "custom".to_string());
    let release_folder = install_root
        .join(source_folder)
        .join(request.release_tag.replace('/', "-"));
    let archive_path = release_folder.join(&request.asset_name);

    fs::create_dir_all(&release_folder).map_err(|error| error.to_string())?;

    let mut response = http_client()?
        .get(&request.asset_url)
        .header("Accept", "application/octet-stream")
        .send()
        .map_err(|error| error.to_string())?
        .error_for_status()
        .map_err(|error| error.to_string())?;
    let mut archive_file = fs::File::create(&archive_path).map_err(|error| error.to_string())?;
    response
        .copy_to(&mut archive_file)
        .map_err(|error| error.to_string())?;

    let executable_path = if request.asset_name.to_lowercase().ends_with(".zip") {
        extract_zip(&archive_path, &release_folder)?;
        find_godot_executable(&release_folder).ok_or_else(|| {
            "Download completed, but the Godot executable could not be found.".to_string()
        })?
    } else {
        archive_path
    };

    mark_executable(&executable_path)?;

    if request.make_default || state.editors.is_empty() {
        for editor in &mut state.editors {
            editor.is_default = false;
        }
    }

    state.editors.push(GodotEditor {
        id: now_id("editor"),
        name: editor_name_from_asset(&request.asset_name),
        version: version_from_tag(&request.release_tag),
        executable_path: executable_path.to_string_lossy().to_string(),
        install_path: release_folder.to_string_lossy().to_string(),
        architecture: architecture_from_asset(&request.asset_name),
        is_default: request.make_default || state.editors.is_empty(),
    });

    write_state(&state)?;
    record_activity(
        "success",
        format!(
            "Installed editor {}.",
            version_from_tag(&request.release_tag)
        ),
    );
    Ok(state)
}

#[tauri::command]
pub(crate) fn remove_editor(editor_id: String) -> Result<HubState, String> {
    let mut state = read_state()?;
    let removed_default = state
        .editors
        .iter()
        .any(|editor| editor.id == editor_id && editor.is_default);

    state.editors.retain(|editor| editor.id != editor_id);

    if removed_default {
        if let Some(first_editor) = state.editors.first_mut() {
            first_editor.is_default = true;
        }
    }

    for project in &mut state.projects {
        if project.editor_id.as_deref() == Some(editor_id.as_str()) {
            project.editor_id = None;
        }
    }

    write_state(&state)?;
    record_activity("info", "Editor removed from Forge.");
    Ok(state)
}

#[tauri::command]
pub(crate) fn set_default_editor(editor_id: String) -> Result<HubState, String> {
    let mut state = read_state()?;

    if !state.editors.iter().any(|editor| editor.id == editor_id) {
        return Err("Editor not found.".into());
    }

    for editor in &mut state.editors {
        editor.is_default = editor.id == editor_id;
    }

    write_state(&state)?;
    record_activity("info", "Default editor changed.");
    Ok(state)
}
