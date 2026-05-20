use std::path::{Path, PathBuf};

use crate::{
    activity::record_activity,
    godot::{
        architecture_from_asset, collect_godot_executables, collect_project_dirs,
        editor_is_registered, editor_name_from_asset, ensure_editor_installed,
        find_godot_executable, immediate_child_dirs, mark_executable, project_is_registered,
        project_name_from_path, version_from_executable_path,
    },
    models::{
        DiscoveredEditor, DiscoveredProject, GodotEditor, GodotProject, HubState,
        RegisterDiscoveredEditorRequest, RegisterDiscoveredProjectRequest, WorkspaceScan,
    },
    paths::now_id,
    state::{read_state, write_state},
};

#[tauri::command]
pub(crate) fn scan_workspace() -> Result<WorkspaceScan, String> {
    let state = read_state()?;
    let install_root = PathBuf::from(&state.settings.default_install_path);
    let project_root = PathBuf::from(&state.settings.default_project_path);
    let mut editors = Vec::new();
    let mut projects = Vec::new();

    if install_root.exists() {
        let mut executables = Vec::new();
        collect_godot_executables(&install_root, 5, &mut executables);
        executables.sort();
        executables.dedup();

        for executable_path in executables {
            let install_path = executable_path
                .parent()
                .map(Path::to_path_buf)
                .unwrap_or_else(|| install_root.clone());
            let executable_name = executable_path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("Godot");

            editors.push(DiscoveredEditor {
                name: editor_name_from_asset(executable_name),
                version: version_from_executable_path(&executable_path),
                architecture: architecture_from_asset(executable_name),
                executable_path: executable_path.to_string_lossy().to_string(),
                install_path: install_path.to_string_lossy().to_string(),
                registered: editor_is_registered(&state, &executable_path),
                corrupt: false,
                reason: None,
            });
        }

        for directory in immediate_child_dirs(&install_root) {
            let has_candidate = editors
                .iter()
                .any(|editor| PathBuf::from(&editor.install_path).starts_with(&directory));
            if !has_candidate && find_godot_executable(&directory).is_none() {
                editors.push(DiscoveredEditor {
                    name: directory
                        .file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or("Unknown editor")
                        .to_string(),
                    version: "unknown".into(),
                    architecture: "unknown".into(),
                    executable_path: String::new(),
                    install_path: directory.to_string_lossy().to_string(),
                    registered: false,
                    corrupt: true,
                    reason: Some(
                        "No Godot executable was found in this installation folder.".into(),
                    ),
                });
            }
        }
    }

    if project_root.exists() {
        let mut project_dirs = Vec::new();
        collect_project_dirs(&project_root, 6, &mut project_dirs);
        project_dirs.sort();
        project_dirs.dedup();

        for project_path in &project_dirs {
            projects.push(DiscoveredProject {
                name: project_name_from_path(project_path),
                path: project_path.to_string_lossy().to_string(),
                registered: project_is_registered(&state, project_path),
                corrupt: false,
                reason: None,
            });
        }

        for directory in immediate_child_dirs(&project_root) {
            let contains_project = project_dirs
                .iter()
                .any(|project| project.starts_with(&directory));
            if !contains_project {
                projects.push(DiscoveredProject {
                    name: project_name_from_path(&directory),
                    path: directory.to_string_lossy().to_string(),
                    registered: false,
                    corrupt: true,
                    reason: Some("No project.godot file was found in this project folder.".into()),
                });
            }
        }
    }

    Ok(WorkspaceScan { editors, projects })
}

#[tauri::command]
pub(crate) fn register_discovered_editor(
    request: RegisterDiscoveredEditorRequest,
) -> Result<HubState, String> {
    let mut state = read_state()?;
    let executable = PathBuf::from(request.executable_path.trim());
    let install_path = PathBuf::from(request.install_path.trim());

    if !executable.exists() {
        return Err("The discovered editor executable does not exist.".into());
    }

    if !install_path.exists() {
        return Err("The discovered editor installation folder does not exist.".into());
    }

    if editor_is_registered(&state, &executable) {
        return Ok(state);
    }

    mark_executable(&executable)?;

    let should_make_default = state.editors.is_empty();
    state.editors.push(GodotEditor {
        id: now_id("editor"),
        name: request.name.trim().to_string(),
        version: request.version.trim().to_string(),
        executable_path: executable.to_string_lossy().to_string(),
        install_path: install_path.to_string_lossy().to_string(),
        architecture: request.architecture.trim().to_string(),
        is_default: should_make_default,
    });

    write_state(&state)?;
    record_activity(
        "success",
        format!(
            "Registered discovered editor {} {}.",
            request.name, request.version
        ),
    );
    Ok(state)
}

#[tauri::command]
pub(crate) fn register_discovered_project(
    request: RegisterDiscoveredProjectRequest,
) -> Result<HubState, String> {
    let mut state = read_state()?;
    ensure_editor_installed(&state)?;
    let path = PathBuf::from(request.path.trim());

    if !path.join("project.godot").exists() {
        return Err("Could not find project.godot at this path.".into());
    }

    if project_is_registered(&state, &path) {
        return Ok(state);
    }

    state.projects.push(GodotProject {
        id: now_id("project"),
        name: request.name.trim().to_string(),
        path: path.to_string_lossy().to_string(),
        editor_id: None,
        favorite: false,
        last_opened: None,
    });

    write_state(&state)?;
    record_activity(
        "success",
        format!("Registered discovered project {}.", request.name),
    );
    Ok(state)
}
