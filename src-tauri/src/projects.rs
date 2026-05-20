use std::{fs, path::PathBuf, process::Command};

use crate::{
    activity::{current_unix_seconds, record_activity},
    filesystem::copy_dir_recursive,
    godot::{ensure_editor_installed, project_name_from_path},
    models::{
        CreateProjectRequest, GodotProject, HubState, ImportProjectRequest, MoveProjectRequest,
    },
    paths::now_id,
    state::{read_state, write_state},
};

#[tauri::command]
pub(crate) fn import_project(request: ImportProjectRequest) -> Result<HubState, String> {
    let mut state = read_state()?;
    ensure_editor_installed(&state)?;
    let path = PathBuf::from(request.path.trim());

    if !path.join("project.godot").exists() {
        return Err("Could not find project.godot at this path.".into());
    }

    let clean_path = path.to_string_lossy().to_string();
    if state
        .projects
        .iter()
        .any(|project| project.path == clean_path)
    {
        return Err("This project is already registered.".into());
    }

    state.projects.push(GodotProject {
        id: now_id("project"),
        name: request
            .name
            .filter(|name| !name.trim().is_empty())
            .unwrap_or_else(|| project_name_from_path(&path)),
        path: clean_path,
        editor_id: request.editor_id,
        favorite: false,
        last_opened: None,
    });

    write_state(&state)?;
    record_activity("success", "Project imported.");
    Ok(state)
}

#[tauri::command]
pub(crate) fn create_project(request: CreateProjectRequest) -> Result<HubState, String> {
    let mut state = read_state()?;
    ensure_editor_installed(&state)?;
    let project_name = request.name.trim();

    if project_name.is_empty() {
        return Err("Enter a project name.".into());
    }

    let root_path = PathBuf::from(request.root_path.trim());
    let project_path = root_path.join(project_name);

    if project_path.exists() {
        return Err("A folder with this name already exists.".into());
    }

    fs::create_dir_all(&project_path).map_err(|error| error.to_string())?;
    fs::write(
        project_path.join("Main.tscn"),
        "[gd_scene format=3]\n\n[node name=\"Main\" type=\"Node2D\"]\n",
    )
    .map_err(|error| error.to_string())?;
    fs::write(
        project_path.join("project.godot"),
        format!(
            "; Engine configuration file.\n; Created by Godot Forge.\n\nconfig_version=5\n\n[application]\n\nconfig/name=\"{}\"\nrun/main_scene=\"res://Main.tscn\"\n",
            project_name.replace('"', "\\\"")
        ),
    )
    .map_err(|error| error.to_string())?;

    state.projects.push(GodotProject {
        id: now_id("project"),
        name: project_name.to_string(),
        path: project_path.to_string_lossy().to_string(),
        editor_id: request.editor_id,
        favorite: false,
        last_opened: None,
    });

    write_state(&state)?;
    record_activity("success", format!("Project {project_name} created."));
    Ok(state)
}

#[tauri::command]
pub(crate) fn remove_project(project_id: String) -> Result<HubState, String> {
    let mut state = read_state()?;
    state.projects.retain(|project| project.id != project_id);
    write_state(&state)?;
    record_activity("info", "Project removed from library.");
    Ok(state)
}

#[tauri::command]
pub(crate) fn move_project(request: MoveProjectRequest) -> Result<HubState, String> {
    let mut state = read_state()?;
    ensure_editor_installed(&state)?;
    let project = state
        .projects
        .iter()
        .find(|project| project.id == request.project_id)
        .cloned()
        .ok_or_else(|| "Project not found.".to_string())?;

    let source = PathBuf::from(&project.path);
    let destination = PathBuf::from(request.destination_path.trim());

    if !source.exists() {
        return Err("The current project folder does not exist.".into());
    }

    if destination.exists() {
        return Err("The destination path already exists.".into());
    }

    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }

    match fs::rename(&source, &destination) {
        Ok(_) => {}
        Err(_) => {
            copy_dir_recursive(&source, &destination)?;
            fs::remove_dir_all(&source).map_err(|error| error.to_string())?;
        }
    }

    if !destination.join("project.godot").exists() {
        return Err("Project moved, but project.godot was not found at the destination.".into());
    }

    if let Some(project) = state
        .projects
        .iter_mut()
        .find(|project| project.id == request.project_id)
    {
        project.path = destination.to_string_lossy().to_string();
    }

    write_state(&state)?;
    record_activity("success", "Project moved.");
    Ok(state)
}

#[tauri::command]
pub(crate) fn toggle_project_favorite(project_id: String) -> Result<HubState, String> {
    let mut state = read_state()?;

    if let Some(project) = state
        .projects
        .iter_mut()
        .find(|project| project.id == project_id)
    {
        project.favorite = !project.favorite;
    }

    write_state(&state)?;
    record_activity("info", "Project favorite state changed.");
    Ok(state)
}

#[tauri::command]
pub(crate) fn launch_project(project_id: String) -> Result<HubState, String> {
    let mut state = read_state()?;
    let project = state
        .projects
        .iter()
        .find(|project| project.id == project_id)
        .cloned()
        .ok_or_else(|| "Project not found.".to_string())?;

    let editor = project
        .editor_id
        .as_ref()
        .and_then(|editor_id| state.editors.iter().find(|editor| &editor.id == editor_id))
        .or_else(|| state.editors.iter().find(|editor| editor.is_default))
        .cloned()
        .ok_or_else(|| "Register a Godot editor before opening projects.".to_string())?;

    Command::new(&editor.executable_path)
        .arg("--editor")
        .arg("--path")
        .arg(&project.path)
        .spawn()
        .map_err(|error| error.to_string())?;

    if let Some(project) = state.projects.iter_mut().find(|item| item.id == project_id) {
        project.last_opened = Some(current_unix_seconds().to_string());
    }

    write_state(&state)?;
    record_activity("success", format!("Launched project {}.", project.name));
    Ok(state)
}
