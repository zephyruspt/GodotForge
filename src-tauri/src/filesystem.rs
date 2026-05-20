use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::models::HubState;

pub(crate) fn copy_dir_recursive(source: &Path, destination: &Path) -> Result<(), String> {
    fs::create_dir_all(destination).map_err(|error| error.to_string())?;

    for entry in fs::read_dir(source).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let entry_path = entry.path();
        let destination_path = destination.join(entry.file_name());

        if entry_path.is_dir() {
            copy_dir_recursive(&entry_path, &destination_path)?;
        } else {
            fs::copy(&entry_path, &destination_path).map_err(|error| error.to_string())?;
        }
    }

    Ok(())
}

pub(crate) fn move_dir_with_fallback(source: &Path, destination: &Path) -> Result<(), String> {
    if !source.exists() {
        return Ok(());
    }

    if destination.exists() {
        return Err(format!(
            "The destination path already exists: {}",
            destination.to_string_lossy()
        ));
    }

    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }

    match fs::rename(source, destination) {
        Ok(_) => Ok(()),
        Err(_) => {
            copy_dir_recursive(source, destination)?;
            fs::remove_dir_all(source).map_err(|error| error.to_string())
        }
    }
}

fn rebase_path(path: &Path, old_root: &Path, new_root: &Path) -> Option<PathBuf> {
    path.strip_prefix(old_root)
        .ok()
        .map(|relative_path| new_root.join(relative_path))
}

pub(crate) fn migrate_registered_paths(
    state: &mut HubState,
    old_install_root: &Path,
    new_install_root: &Path,
    old_project_root: &Path,
    new_project_root: &Path,
) -> Result<(), String> {
    if old_install_root != new_install_root {
        for editor in &mut state.editors {
            let install_path = PathBuf::from(&editor.install_path);
            let Some(next_install_path) =
                rebase_path(&install_path, old_install_root, new_install_root)
            else {
                continue;
            };
            let executable_path = PathBuf::from(&editor.executable_path);
            let next_executable_path = executable_path
                .strip_prefix(&install_path)
                .ok()
                .map(|relative_path| next_install_path.join(relative_path))
                .unwrap_or_else(|| executable_path.clone());

            move_dir_with_fallback(&install_path, &next_install_path)?;
            editor.install_path = next_install_path.to_string_lossy().to_string();
            editor.executable_path = next_executable_path.to_string_lossy().to_string();
        }
    }

    if old_project_root != new_project_root {
        for project in &mut state.projects {
            let project_path = PathBuf::from(&project.path);
            let Some(next_project_path) =
                rebase_path(&project_path, old_project_root, new_project_root)
            else {
                continue;
            };

            move_dir_with_fallback(&project_path, &next_project_path)?;
            project.path = next_project_path.to_string_lossy().to_string();
        }
    }

    Ok(())
}
