mod filesystem;
mod git;
mod godot;
mod models;
mod paths;

use filesystem::*;
use git::*;
use godot::*;
use models::*;
use paths::*;
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::Manager;

#[cfg(target_os = "macos")]
use tauri::{
    menu::{AboutMetadata, Menu, MenuItem, PredefinedMenuItem, Submenu},
    Emitter,
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

const OFFICIAL_RELEASE_REPOSITORY: &str = "godotengine/godot";
const RELEASE_CACHE_TTL_SECONDS: u64 = 60 * 60 * 6;

fn current_unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
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

fn write_state(state: &HubState) -> Result<(), String> {
    let path = config_path();

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }

    let data = serde_json::to_string_pretty(state).map_err(|error| error.to_string())?;
    fs::write(path, data).map_err(|error| error.to_string())
}

fn read_release_cache(repositories: &[String], limit: usize, page: usize) -> Option<ReleaseCache> {
    let data = fs::read_to_string(release_cache_path(repositories, limit, page)).ok()?;
    let cache: ReleaseCache = serde_json::from_str(&data).ok()?;

    if cache.repositories == repositories && cache.limit == limit && cache.page == page {
        Some(cache)
    } else {
        None
    }
}

fn fresh_cached_releases(
    repositories: &[String],
    limit: usize,
    page: usize,
) -> Option<Vec<GodotRelease>> {
    let cache = read_release_cache(repositories, limit, page)?;
    let age = current_unix_seconds().saturating_sub(cache.fetched_at);

    if age <= RELEASE_CACHE_TTL_SECONDS {
        Some(cache.releases)
    } else {
        None
    }
}

fn stale_cached_releases(
    repositories: &[String],
    limit: usize,
    page: usize,
) -> Option<Vec<GodotRelease>> {
    read_release_cache(repositories, limit, page).map(|cache| cache.releases)
}

fn write_release_cache(
    repositories: Vec<String>,
    limit: usize,
    page: usize,
    releases: Vec<GodotRelease>,
) -> Result<(), String> {
    let path = release_cache_path(&repositories, limit, page);

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }

    let cache = ReleaseCache {
        repositories,
        limit,
        page,
        fetched_at: current_unix_seconds(),
        releases,
    };
    let data = serde_json::to_string_pretty(&cache).map_err(|error| error.to_string())?;
    fs::write(path, data).map_err(|error| error.to_string())
}

fn http_client() -> Result<reqwest::blocking::Client, String> {
    reqwest::blocking::Client::builder()
        .user_agent("GodotForge/0.1")
        .build()
        .map_err(|error| error.to_string())
}

fn github_token(settings: &HubSettings) -> Option<String> {
    env::var("GITHUB_TOKEN")
        .or_else(|_| env::var("GH_TOKEN"))
        .ok()
        .or_else(|| Some(settings.github_token.clone()))
        .map(|token| token.trim().to_string())
        .filter(|token| !token.is_empty())
}

fn github_release_error(error: reqwest::Error) -> String {
    if error.status() == Some(reqwest::StatusCode::FORBIDDEN) {
        return "GitHub returned 403 Forbidden while loading releases. You may be rate limited on the public API. Wait for the limit to reset, configure a GitHub token in Settings, or start Godot Forge with GITHUB_TOKEN/GH_TOKEN set so release requests are authenticated.".into();
    }

    error.to_string()
}

#[tauri::command]
fn load_hub_state() -> Result<HubState, String> {
    read_state()
}

#[tauri::command]
fn read_legal_document(document: String) -> Result<LegalDocument, String> {
    let (title, body) = match document.as_str() {
        "source" => ("Source Code License", include_str!("../../LICENSE")),
        "brand" => (
            "Brand Assets License",
            include_str!("../../LICENSE-BRAND-ASSETS.md"),
        ),
        "notice" => ("Notice", include_str!("../../NOTICE")),
        _ => return Err("Unknown legal document.".into()),
    };

    Ok(LegalDocument {
        title: title.into(),
        body: body.into(),
    })
}

#[tauri::command]
fn detect_system_profile() -> SystemProfile {
    let os = env::consts::OS.to_string();
    let arch = env::consts::ARCH.to_string();
    let godot_platform = match env::consts::OS {
        "linux" => "linux",
        "windows" => "win",
        "macos" => "macos",
        "android" => "android",
        other => other,
    }
    .to_string();

    SystemProfile {
        os,
        arch,
        godot_platform,
    }
}

#[tauri::command]
fn fetch_godot_releases(
    limit: Option<usize>,
    page: Option<usize>,
) -> Result<Vec<GodotRelease>, String> {
    let state = read_state()?;
    let mut repositories = vec![OFFICIAL_RELEASE_REPOSITORY.to_string()];
    repositories.extend(normalize_release_repositories(
        &state.settings.release_repositories,
    )?);
    let max_items = limit.unwrap_or(8).clamp(1, 20);
    let page = page.unwrap_or(1).clamp(1, 50);

    if let Some(releases) = fresh_cached_releases(&repositories, max_items, page) {
        return Ok(releases);
    }

    let client = http_client()?;
    let token = github_token(&state.settings);
    let mut releases = Vec::new();

    for repository in &repositories {
        let url = format!(
            "https://api.github.com/repos/{repository}/releases?per_page={max_items}&page={page}"
        );
        let mut request = client
            .get(url)
            .header("Accept", "application/vnd.github+json");

        if let Some(token) = &token {
            request = request.bearer_auth(token);
        }

        let response = match request.send() {
            Ok(response) => response,
            Err(error) => {
                if let Some(cached_releases) = stale_cached_releases(&repositories, max_items, page)
                {
                    return Ok(cached_releases);
                }

                return Err(github_release_error(error));
            }
        };
        let response = match response.error_for_status() {
            Ok(response) => response,
            Err(error) => {
                if let Some(cached_releases) = stale_cached_releases(&repositories, max_items, page)
                {
                    return Ok(cached_releases);
                }

                return Err(github_release_error(error));
            }
        };

        let mut repository_releases = match response.json::<Vec<GodotRelease>>() {
            Ok(repository_releases) => repository_releases,
            Err(error) => {
                if let Some(cached_releases) = stale_cached_releases(&repositories, max_items, page)
                {
                    return Ok(cached_releases);
                }

                return Err(github_release_error(error));
            }
        };

        for release in &mut repository_releases {
            release.source_repository = repository.clone();
        }

        releases.extend(repository_releases);
    }

    releases.sort_by(|left, right| right.published_at.cmp(&left.published_at));
    let _ = write_release_cache(repositories, max_items, page, releases.clone());
    Ok(releases)
}

#[tauri::command]
fn download_godot_editor(request: DownloadEditorRequest) -> Result<HubState, String> {
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
    Ok(state)
}

#[tauri::command]
fn save_settings(request: UpdateSettingsRequest) -> Result<HubState, String> {
    let mut state = read_state()?;
    let old_install_root = PathBuf::from(&state.settings.default_install_path);
    let old_project_root = PathBuf::from(&state.settings.default_project_path);
    let new_install_root = PathBuf::from(request.default_install_path.trim());
    let new_project_root = PathBuf::from(request.default_project_path.trim());

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
    state.settings.github_token = request.github_token.trim().to_string();
    state.settings.release_repository = None;

    write_state(&state)?;
    Ok(state)
}

#[tauri::command]
fn restore_default_settings() -> Result<HubState, String> {
    let mut state = read_state()?;
    state.settings = default_state().settings;

    write_state(&state)?;
    Ok(state)
}

#[tauri::command]
fn scan_workspace() -> Result<WorkspaceScan, String> {
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
fn register_discovered_editor(
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
    Ok(state)
}

#[tauri::command]
fn register_discovered_project(
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
    Ok(state)
}

#[tauri::command]
fn remove_editor(editor_id: String) -> Result<HubState, String> {
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
    Ok(state)
}

#[tauri::command]
fn set_default_editor(editor_id: String) -> Result<HubState, String> {
    let mut state = read_state()?;

    if !state.editors.iter().any(|editor| editor.id == editor_id) {
        return Err("Editor not found.".into());
    }

    for editor in &mut state.editors {
        editor.is_default = editor.id == editor_id;
    }

    write_state(&state)?;
    Ok(state)
}

#[tauri::command]
fn import_project(request: ImportProjectRequest) -> Result<HubState, String> {
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
    Ok(state)
}

#[tauri::command]
fn create_project(request: CreateProjectRequest) -> Result<HubState, String> {
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
    Ok(state)
}

#[tauri::command]
fn remove_project(project_id: String) -> Result<HubState, String> {
    let mut state = read_state()?;
    state.projects.retain(|project| project.id != project_id);
    write_state(&state)?;
    Ok(state)
}

#[tauri::command]
fn move_project(request: MoveProjectRequest) -> Result<HubState, String> {
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
    Ok(state)
}

#[tauri::command]
fn toggle_project_favorite(project_id: String) -> Result<HubState, String> {
    let mut state = read_state()?;

    if let Some(project) = state
        .projects
        .iter_mut()
        .find(|project| project.id == project_id)
    {
        project.favorite = !project.favorite;
    }

    write_state(&state)?;
    Ok(state)
}

#[tauri::command]
fn launch_project(project_id: String) -> Result<HubState, String> {
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
        project.last_opened = Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|duration| duration.as_secs().to_string())
                .unwrap_or_default(),
        );
    }

    write_state(&state)?;
    Ok(state)
}

#[cfg(target_os = "macos")]
fn app_menu(handle: &tauri::AppHandle) -> tauri::Result<Menu<tauri::Wry>> {
    let about = PredefinedMenuItem::about(
        handle,
        Some("About Godot Forge"),
        Some(AboutMetadata {
            name: Some("Godot Forge".into()),
            version: Some(env!("CARGO_PKG_VERSION").into()),
            short_version: None,
            authors: Some(vec!["ZEPHYRUS PROSPERITY - UNIPESSOAL LDA".into()]),
            comments: Some("Desktop hub for Godot editors, projects, releases, and Git workflows.".into()),
            copyright: Some("Copyright 2026 ZEPHYRUS PROSPERITY - UNIPESSOAL LDA".into()),
            license: Some("Apache-2.0 source code; brand assets all rights reserved.".into()),
            website: None,
            website_label: None,
            credits: Some("Godot Forge is independent and is not affiliated with Godot Engine or the Godot Foundation.".into()),
            icon: tauri::image::Image::from_bytes(include_bytes!("../icons/icon.png")).ok(),
        }),
    )?;
    let app_separator = PredefinedMenuItem::separator(handle)?;
    let quit = PredefinedMenuItem::quit(handle, Some("Quit Godot Forge"))?;
    let app = Submenu::with_items(
        handle,
        "Godot Forge",
        true,
        &[&about, &app_separator, &quit],
    )?;

    let projects = MenuItem::with_id(
        handle,
        "navigate-projects",
        "Projects",
        true,
        Some("CmdOrCtrl+1"),
    )?;
    let editors = MenuItem::with_id(
        handle,
        "navigate-editors",
        "Editors",
        true,
        Some("CmdOrCtrl+2"),
    )?;
    let settings = MenuItem::with_id(
        handle,
        "navigate-settings",
        "Settings",
        true,
        Some("CmdOrCtrl+,"),
    )?;
    let file_separator = PredefinedMenuItem::separator(handle)?;
    let close = PredefinedMenuItem::close_window(handle, Some("Close Window"))?;
    let file = Submenu::with_items(
        handle,
        "File",
        true,
        &[&projects, &editors, &settings, &file_separator, &close],
    )?;

    let undo = PredefinedMenuItem::undo(handle, None)?;
    let redo = PredefinedMenuItem::redo(handle, None)?;
    let edit_separator_one = PredefinedMenuItem::separator(handle)?;
    let cut = PredefinedMenuItem::cut(handle, None)?;
    let copy = PredefinedMenuItem::copy(handle, None)?;
    let paste = PredefinedMenuItem::paste(handle, None)?;
    let edit_separator_two = PredefinedMenuItem::separator(handle)?;
    let select_all = PredefinedMenuItem::select_all(handle, None)?;
    let edit = Submenu::with_items(
        handle,
        "Edit",
        true,
        &[
            &undo,
            &redo,
            &edit_separator_one,
            &cut,
            &copy,
            &paste,
            &edit_separator_two,
            &select_all,
        ],
    )?;

    let minimize = PredefinedMenuItem::minimize(handle, None)?;
    let maximize = PredefinedMenuItem::maximize(handle, None)?;
    let fullscreen = PredefinedMenuItem::fullscreen(handle, None)?;
    let window = Submenu::with_items(handle, "Window", true, &[&minimize, &maximize, &fullscreen])?;

    let security = MenuItem::with_id(
        handle,
        "security-policy",
        "Security Policy",
        true,
        None::<&str>,
    )?;
    let help = Submenu::with_items(handle, "Help", true, &[&security])?;

    Menu::with_items(handle, &[&app, &file, &edit, &window, &help])
}

#[cfg(target_os = "linux")]
fn sanitize_gtk_modules_for_linux() {
    let Ok(modules) = env::var("GTK_MODULES") else {
        return;
    };

    let filtered_modules = modules
        .split(':')
        .filter(|module| !module.trim().is_empty() && *module != "appmenu-gtk-module")
        .collect::<Vec<_>>();

    if filtered_modules.is_empty() {
        env::remove_var("GTK_MODULES");
    } else {
        env::set_var("GTK_MODULES", filtered_modules.join(":"));
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(target_os = "linux")]
    sanitize_gtk_modules_for_linux();

    let builder = tauri::Builder::default();

    #[cfg(target_os = "macos")]
    let builder = builder
        .menu(app_menu)
        .on_menu_event(|app, event| match event.id() {
            id if id == "navigate-projects" => {
                let _ = app.emit("menu-action", "projects");
            }
            id if id == "navigate-editors" => {
                let _ = app.emit("menu-action", "editors");
            }
            id if id == "navigate-settings" => {
                let _ = app.emit("menu-action", "settings");
            }
            id if id == "security-policy" => {
                let _ = app.emit("menu-action", "security-policy");
            }
            _ => {}
        });

    builder
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            if let Some(window) = app.get_webview_window("main") {
                if let Ok(icon) =
                    tauri::image::Image::from_bytes(include_bytes!("../icons/icon.png"))
                {
                    let _ = window.set_icon(icon);
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            load_hub_state,
            detect_system_profile,
            read_legal_document,
            get_project_git_status,
            init_project_git,
            get_project_git_log,
            list_project_git_branches,
            create_project_git_branch,
            checkout_project_git_branch,
            set_project_git_remote,
            push_project_git_branch,
            fetch_godot_releases,
            download_godot_editor,
            save_settings,
            restore_default_settings,
            scan_workspace,
            register_discovered_editor,
            register_discovered_project,
            remove_editor,
            set_default_editor,
            import_project,
            create_project,
            remove_project,
            move_project,
            toggle_project_favorite,
            launch_project
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
