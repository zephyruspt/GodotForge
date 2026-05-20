use std::{
    fs, io,
    path::{Path, PathBuf},
};

use crate::models::HubState;

pub(crate) const OFFICIAL_RELEASE_REPOSITORY: &str = "godotengine/godot";

pub(crate) fn project_name_from_path(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.trim().is_empty())
        .unwrap_or("Godot Project")
        .to_string()
}

fn normalize_release_repository(value: &str) -> Result<String, String> {
    let trimmed = value.trim().trim_end_matches('/').trim_end_matches(".git");
    let repository = trimmed
        .strip_prefix("git@github.com:")
        .or_else(|| trimmed.strip_prefix("ssh://git@github.com/"))
        .or_else(|| trimmed.strip_prefix("https://github.com/"))
        .or_else(|| trimmed.strip_prefix("http://github.com/"))
        .unwrap_or(trimmed);
    let parts = repository.split('/').collect::<Vec<_>>();

    if parts.len() != 2 || parts.iter().any(|part| part.trim().is_empty()) {
        return Err(
            "Automatic release downloads currently support GitHub repositories. Use owner/name, a GitHub URL, or a GitHub SSH URL.".into(),
        );
    }

    if parts.iter().any(|part| {
        part.chars().any(|character| {
            !(character.is_ascii_alphanumeric()
                || character == '-'
                || character == '_'
                || character == '.')
        })
    }) {
        return Err("The release repository contains invalid characters.".into());
    }

    Ok(format!("{}/{}", parts[0], parts[1]))
}

pub(crate) fn normalize_release_repositories(values: &[String]) -> Result<Vec<String>, String> {
    let mut repositories = Vec::new();

    for value in values {
        if value.trim().is_empty() {
            continue;
        }

        let repository = normalize_release_repository(value)?;
        if repository == OFFICIAL_RELEASE_REPOSITORY {
            continue;
        }

        if !repositories.contains(&repository) {
            repositories.push(repository);
        }
    }

    Ok(repositories)
}

pub(crate) fn safe_folder_name(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric()
                || character == '-'
                || character == '_'
                || character == '.'
            {
                character
            } else {
                '-'
            }
        })
        .collect()
}

pub(crate) fn ensure_editor_installed(state: &HubState) -> Result<(), String> {
    if state.editors.is_empty() {
        Err("Install a Godot editor before creating or editing projects.".into())
    } else {
        Ok(())
    }
}

pub(crate) fn version_from_tag(tag: &str) -> String {
    tag.trim_start_matches("godot-")
        .trim_start_matches('v')
        .to_string()
}

pub(crate) fn architecture_from_asset(name: &str) -> String {
    let lower = name.to_lowercase();

    if lower.contains("arm64") || lower.contains("aarch64") {
        "arm64".into()
    } else if lower.contains("x86_64") || lower.contains("64") {
        "x86_64".into()
    } else if lower.contains("x86_32") || lower.contains("32") {
        "x86".into()
    } else {
        "unknown".into()
    }
}

pub(crate) fn editor_name_from_asset(asset_name: &str) -> String {
    let lower = asset_name.to_lowercase();
    if lower.contains("mono") {
        "Godot .NET".into()
    } else {
        "Godot".into()
    }
}

pub(crate) fn extract_zip(zip_path: &Path, destination: &Path) -> Result<(), String> {
    let file = fs::File::open(zip_path).map_err(|error| error.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|error| error.to_string())?;

    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).map_err(|error| error.to_string())?;
        let entry_path = entry
            .enclosed_name()
            .ok_or_else(|| format!("Archive entry uses an unsafe path: {}", entry.name()))?;
        let output_path = destination.join(entry_path);

        if entry.is_dir() {
            fs::create_dir_all(&output_path).map_err(|error| error.to_string())?;
            continue;
        }

        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }

        let mut output_file = fs::File::create(&output_path).map_err(|error| error.to_string())?;
        io::copy(&mut entry, &mut output_file).map_err(|error| error.to_string())?;
    }

    Ok(())
}

pub(crate) fn find_godot_executable(path: &Path) -> Option<PathBuf> {
    let entries = fs::read_dir(path).ok()?;

    for entry in entries.flatten() {
        let entry_path = entry.path();
        if entry_path.is_dir() {
            if let Some(found) = find_godot_executable(&entry_path) {
                return Some(found);
            }
            continue;
        }

        if looks_like_godot_executable(&entry_path) {
            return Some(entry_path);
        }
    }

    None
}

fn looks_like_godot_executable(path: &Path) -> bool {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_lowercase();

    file_name.contains("godot")
        && !file_name.ends_with(".zip")
        && !file_name.ends_with(".txt")
        && !file_name.ends_with(".md")
}

pub(crate) fn collect_godot_executables(path: &Path, depth: usize, output: &mut Vec<PathBuf>) {
    if depth == 0 {
        return;
    }

    let Ok(entries) = fs::read_dir(path) else {
        return;
    };

    for entry in entries.flatten() {
        let entry_path = entry.path();
        if entry_path.is_dir() {
            collect_godot_executables(&entry_path, depth - 1, output);
        } else if looks_like_godot_executable(&entry_path) {
            output.push(entry_path);
        }
    }
}

pub(crate) fn collect_project_dirs(path: &Path, depth: usize, output: &mut Vec<PathBuf>) {
    if depth == 0 {
        return;
    }

    if path.join("project.godot").exists() {
        output.push(path.to_path_buf());
        return;
    }

    let Ok(entries) = fs::read_dir(path) else {
        return;
    };

    for entry in entries.flatten() {
        let entry_path = entry.path();
        if entry_path.is_dir() {
            collect_project_dirs(&entry_path, depth - 1, output);
        }
    }
}

pub(crate) fn immediate_child_dirs(path: &Path) -> Vec<PathBuf> {
    fs::read_dir(path)
        .ok()
        .into_iter()
        .flat_map(|entries| entries.flatten())
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .collect()
}

pub(crate) fn version_from_executable_path(path: &Path) -> String {
    let text = path.to_string_lossy().to_lowercase();
    text.split(['/', '\\', '_', '-'])
        .find(|part| {
            part.starts_with('v')
                || part
                    .chars()
                    .next()
                    .is_some_and(|character| character.is_ascii_digit())
        })
        .map(version_from_tag)
        .unwrap_or_else(|| "unknown".into())
}

pub(crate) fn project_is_registered(state: &HubState, path: &Path) -> bool {
    let path = path.to_string_lossy();
    state.projects.iter().any(|project| project.path == path)
}

pub(crate) fn editor_is_registered(state: &HubState, executable_path: &Path) -> bool {
    let executable_path = executable_path.to_string_lossy();
    state
        .editors
        .iter()
        .any(|editor| editor.executable_path == executable_path)
}

#[cfg(unix)]
pub(crate) fn mark_executable(path: &Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = fs::metadata(path)
        .map_err(|error| error.to_string())?
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).map_err(|error| error.to_string())
}

#[cfg(not(unix))]
pub(crate) fn mark_executable(_path: &Path) -> Result<(), String> {
    Ok(())
}
