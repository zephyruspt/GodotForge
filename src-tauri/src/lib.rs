use serde::{Deserialize, Serialize};
use std::{
    env, fs,
    hash::{Hash, Hasher},
    io,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct GodotEditor {
    id: String,
    name: String,
    version: String,
    executable_path: String,
    install_path: String,
    architecture: String,
    is_default: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct GodotProject {
    id: String,
    name: String,
    path: String,
    editor_id: Option<String>,
    favorite: bool,
    last_opened: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct HubSettings {
    default_install_path: String,
    default_project_path: String,
    #[serde(default)]
    release_repositories: Vec<String>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    github_token: String,
    #[serde(default, skip_serializing)]
    release_repository: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct HubState {
    editors: Vec<GodotEditor>,
    projects: Vec<GodotProject>,
    settings: HubSettings,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct GodotReleaseAsset {
    id: u64,
    name: String,
    size: u64,
    #[serde(alias = "browser_download_url")]
    browser_download_url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct GodotRelease {
    id: u64,
    name: Option<String>,
    #[serde(alias = "tag_name")]
    tag_name: String,
    prerelease: bool,
    #[serde(alias = "published_at")]
    published_at: Option<String>,
    #[serde(alias = "html_url")]
    html_url: String,
    assets: Vec<GodotReleaseAsset>,
    #[serde(default)]
    source_repository: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReleaseCache {
    repositories: Vec<String>,
    limit: usize,
    #[serde(default = "default_release_page")]
    page: usize,
    fetched_at: u64,
    releases: Vec<GodotRelease>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct SystemProfile {
    os: String,
    arch: String,
    godot_platform: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct GitStatus {
    available: bool,
    is_repo: bool,
    branch: Option<String>,
    remote: Option<String>,
    changed_files: usize,
    untracked_files: usize,
    summary: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct GitLogEntry {
    hash: String,
    author: String,
    relative_date: String,
    subject: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct GitBranch {
    name: String,
    current: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ImportProjectRequest {
    name: Option<String>,
    path: String,
    editor_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateProjectRequest {
    name: String,
    root_path: String,
    editor_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateSettingsRequest {
    default_install_path: String,
    default_project_path: String,
    release_repositories: Vec<String>,
    github_token: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DownloadEditorRequest {
    release_tag: String,
    release_repository: Option<String>,
    asset_name: String,
    asset_url: String,
    install_path: Option<String>,
    make_default: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MoveProjectRequest {
    project_id: String,
    destination_path: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GitBranchRequest {
    project_id: String,
    branch_name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GitRemoteRequest {
    project_id: String,
    remote_url: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct LegalDocument {
    title: String,
    body: String,
}

fn home_dir() -> PathBuf {
    env::var_os("HOME")
        .or_else(|| env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn documents_dir() -> PathBuf {
    #[cfg(target_os = "linux")]
    {
        if let Some(path) = xdg_documents_dir() {
            return path;
        }
    }

    env::var_os("USERPROFILE")
        .map(|path| PathBuf::from(path).join("Documents"))
        .unwrap_or_else(|| home_dir().join("Documents"))
}

#[cfg(target_os = "linux")]
fn xdg_documents_dir() -> Option<PathBuf> {
    let config_home = env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home_dir().join(".config"));
    let config = fs::read_to_string(config_home.join("user-dirs.dirs")).ok()?;

    config.lines().find_map(|line| {
        let value = line.strip_prefix("XDG_DOCUMENTS_DIR=")?;
        let value = value.trim().trim_matches('"');
        let expanded = value
            .strip_prefix("$HOME/")
            .map(|path| home_dir().join(path))
            .unwrap_or_else(|| PathBuf::from(value));

        Some(expanded)
    })
}

fn default_install_path() -> PathBuf {
    #[cfg(target_os = "linux")]
    {
        return home_dir().join(".Godot").join("Editors");
    }

    #[cfg(target_os = "macos")]
    {
        return home_dir()
            .join("Applications")
            .join("GodotForge")
            .join("Editors");
    }

    #[cfg(target_os = "windows")]
    {
        return env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .or_else(|| env::var_os("APPDATA").map(PathBuf::from))
            .unwrap_or_else(|| home_dir().join("AppData").join("Local"))
            .join("GodotForge")
            .join("Editors");
    }

    #[allow(unreachable_code)]
    home_dir().join(".godot-forge").join("editors")
}

fn default_project_path() -> PathBuf {
    documents_dir().join("GodotForge").join("Projects")
}

fn config_path() -> PathBuf {
    let base = env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home_dir().join(".config"));

    base.join("godot-forge").join("hub-state.json")
}

fn release_cache_path(repositories: &[String], limit: usize, page: usize) -> PathBuf {
    let base = env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home_dir().join(".config"));
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    repositories.hash(&mut hasher);

    base.join("godot-forge").join(format!(
        "release-cache-{limit}-{page}-{}.json",
        hasher.finish()
    ))
}

fn now_id(prefix: &str) -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default();

    format!("{prefix}-{millis}")
}

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

fn default_release_page() -> usize {
    1
}

fn current_unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

fn read_state() -> Result<HubState, String> {
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

fn project_name_from_path(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.trim().is_empty())
        .unwrap_or("Godot Project")
        .to_string()
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

fn normalize_release_repositories(values: &[String]) -> Result<Vec<String>, String> {
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

fn safe_folder_name(value: &str) -> String {
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

fn ensure_editor_installed(state: &HubState) -> Result<(), String> {
    if state.editors.is_empty() {
        Err("Install a Godot editor before creating or editing projects.".into())
    } else {
        Ok(())
    }
}

fn version_from_tag(tag: &str) -> String {
    tag.trim_start_matches("godot-")
        .trim_start_matches('v')
        .to_string()
}

fn architecture_from_asset(name: &str) -> String {
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

fn editor_name_from_asset(asset_name: &str) -> String {
    let lower = asset_name.to_lowercase();
    if lower.contains("mono") {
        "Godot .NET".into()
    } else {
        "Godot".into()
    }
}

fn extract_zip(zip_path: &Path, destination: &Path) -> Result<(), String> {
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

fn find_godot_executable(path: &Path) -> Option<PathBuf> {
    let entries = fs::read_dir(path).ok()?;

    for entry in entries.flatten() {
        let entry_path = entry.path();
        if entry_path.is_dir() {
            if let Some(found) = find_godot_executable(&entry_path) {
                return Some(found);
            }
            continue;
        }

        let file_name = entry_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_lowercase();

        let looks_like_godot = file_name.contains("godot")
            && !file_name.ends_with(".zip")
            && !file_name.ends_with(".txt")
            && !file_name.ends_with(".md");

        if looks_like_godot {
            return Some(entry_path);
        }
    }

    None
}

#[cfg(unix)]
fn mark_executable(path: &Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = fs::metadata(path)
        .map_err(|error| error.to_string())?
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).map_err(|error| error.to_string())
}

#[cfg(not(unix))]
fn mark_executable(_path: &Path) -> Result<(), String> {
    Ok(())
}

fn command_text(mut command: Command) -> Result<String, String> {
    let output = command.output().map_err(|error| error.to_string())?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn project_by_id(project_id: &str) -> Result<GodotProject, String> {
    read_state()?
        .projects
        .into_iter()
        .find(|project| project.id == project_id)
        .ok_or_else(|| "Project not found.".to_string())
}

fn git_available() -> bool {
    Command::new("git").arg("--version").output().is_ok()
}

fn copy_dir_recursive(source: &Path, destination: &Path) -> Result<(), String> {
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
fn get_project_git_status(project_id: String) -> Result<GitStatus, String> {
    let project = project_by_id(&project_id)?;

    if !git_available() {
        return Ok(GitStatus {
            available: false,
            is_repo: false,
            branch: None,
            remote: None,
            changed_files: 0,
            untracked_files: 0,
            summary: "Git was not found on this system.".into(),
        });
    }

    let project_path = PathBuf::from(project.path);
    let repo_check = Command::new("git")
        .current_dir(&project_path)
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()
        .map_err(|error| error.to_string())?;

    if !repo_check.status.success() {
        return Ok(GitStatus {
            available: true,
            is_repo: false,
            branch: None,
            remote: None,
            changed_files: 0,
            untracked_files: 0,
            summary: "This project is not a Git repository yet.".into(),
        });
    }

    let branch = command_text({
        let mut command = Command::new("git");
        command
            .current_dir(&project_path)
            .args(["branch", "--show-current"]);
        command
    })
    .ok()
    .filter(|value| !value.is_empty());

    let remote = command_text({
        let mut command = Command::new("git");
        command
            .current_dir(&project_path)
            .args(["remote", "get-url", "origin"]);
        command
    })
    .ok()
    .filter(|value| !value.is_empty());

    let porcelain = command_text({
        let mut command = Command::new("git");
        command
            .current_dir(&project_path)
            .args(["status", "--porcelain"]);
        command
    })?;

    let mut changed_files = 0;
    let mut untracked_files = 0;

    for line in porcelain.lines() {
        if line.starts_with("??") {
            untracked_files += 1;
        } else if !line.trim().is_empty() {
            changed_files += 1;
        }
    }

    let summary = if changed_files == 0 && untracked_files == 0 {
        "Working tree limpo.".into()
    } else {
        format!("{changed_files} changed, {untracked_files} untracked.")
    };

    Ok(GitStatus {
        available: true,
        is_repo: true,
        branch,
        remote,
        changed_files,
        untracked_files,
        summary,
    })
}

#[tauri::command]
fn init_project_git(project_id: String) -> Result<GitStatus, String> {
    let project = project_by_id(&project_id)?;

    if !git_available() {
        return Err("Git was not found on this system.".into());
    }

    let project_path = PathBuf::from(project.path);

    command_text({
        let mut command = Command::new("git");
        command.current_dir(&project_path).arg("init");
        command
    })?;

    let gitignore_path = project_path.join(".gitignore");
    if !gitignore_path.exists() {
        fs::write(
            gitignore_path,
            ".godot/\n.import/\nexport_presets.cfg\n*.translation\n*.tmp\n.mono/\n",
        )
        .map_err(|error| error.to_string())?;
    }

    get_project_git_status(project_id)
}

#[tauri::command]
fn get_project_git_log(project_id: String) -> Result<Vec<GitLogEntry>, String> {
    let project = project_by_id(&project_id)?;

    if !git_available() {
        return Ok(Vec::new());
    }

    let project_path = PathBuf::from(project.path);
    let repo_check = Command::new("git")
        .current_dir(&project_path)
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()
        .map_err(|error| error.to_string())?;

    if !repo_check.status.success() {
        return Ok(Vec::new());
    }

    let output = command_text({
        let mut command = Command::new("git");
        command.current_dir(&project_path).args([
            "log",
            "-12",
            "--pretty=format:%h%x1f%an%x1f%ar%x1f%s",
        ]);
        command
    })
    .unwrap_or_default();

    Ok(output
        .lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split('\u{1f}').collect();
            if parts.len() != 4 {
                return None;
            }

            Some(GitLogEntry {
                hash: parts[0].to_string(),
                author: parts[1].to_string(),
                relative_date: parts[2].to_string(),
                subject: parts[3].to_string(),
            })
        })
        .collect())
}

#[tauri::command]
fn list_project_git_branches(project_id: String) -> Result<Vec<GitBranch>, String> {
    let project = project_by_id(&project_id)?;

    if !git_available() {
        return Err("Git was not found on this system.".into());
    }

    let project_path = PathBuf::from(project.path);
    let output = command_text({
        let mut command = Command::new("git");
        command
            .current_dir(&project_path)
            .args(["branch", "--format=%(HEAD)%09%(refname:short)"]);
        command
    })?;

    Ok(output
        .lines()
        .filter_map(|line| {
            let (head, name) = line.split_once('\t')?;
            let clean_name = name.trim();
            if clean_name.is_empty() {
                return None;
            }

            Some(GitBranch {
                name: clean_name.to_string(),
                current: head.trim() == "*",
            })
        })
        .collect())
}

#[tauri::command]
fn create_project_git_branch(request: GitBranchRequest) -> Result<GitStatus, String> {
    let project = project_by_id(&request.project_id)?;
    let branch_name = request.branch_name.trim();

    if branch_name.is_empty() {
        return Err("Enter a branch name.".into());
    }

    command_text({
        let mut command = Command::new("git");
        command
            .current_dir(project.path)
            .args(["checkout", "-b", branch_name]);
        command
    })?;

    get_project_git_status(request.project_id)
}

#[tauri::command]
fn checkout_project_git_branch(request: GitBranchRequest) -> Result<GitStatus, String> {
    let project = project_by_id(&request.project_id)?;
    let branch_name = request.branch_name.trim();

    if branch_name.is_empty() {
        return Err("Enter a branch name.".into());
    }

    command_text({
        let mut command = Command::new("git");
        command
            .current_dir(project.path)
            .args(["checkout", branch_name]);
        command
    })?;

    get_project_git_status(request.project_id)
}

#[tauri::command]
fn set_project_git_remote(request: GitRemoteRequest) -> Result<GitStatus, String> {
    let project = project_by_id(&request.project_id)?;
    let remote_url = request.remote_url.trim();

    if remote_url.is_empty() {
        return Err("Enter the remote URL.".into());
    }

    let project_path = PathBuf::from(project.path);
    let has_origin = Command::new("git")
        .current_dir(&project_path)
        .args(["remote", "get-url", "origin"])
        .output()
        .map_err(|error| error.to_string())?
        .status
        .success();

    if has_origin {
        command_text({
            let mut command = Command::new("git");
            command
                .current_dir(&project_path)
                .args(["remote", "set-url", "origin", remote_url]);
            command
        })?;
    } else {
        command_text({
            let mut command = Command::new("git");
            command
                .current_dir(&project_path)
                .args(["remote", "add", "origin", remote_url]);
            command
        })?;
    }

    get_project_git_status(request.project_id)
}

#[tauri::command]
fn push_project_git_branch(project_id: String) -> Result<GitStatus, String> {
    let project = project_by_id(&project_id)?;
    let project_path = PathBuf::from(project.path);
    let status = get_project_git_status(project_id.clone())?;
    let branch = status
        .branch
        .clone()
        .filter(|branch| !branch.trim().is_empty())
        .ok_or_else(|| "Could not detect the current branch.".to_string())?;

    if status.remote.is_none() {
        return Err("Configure o remote origin antes de fazer push.".into());
    }

    command_text({
        let mut command = Command::new("git");
        command
            .current_dir(&project_path)
            .args(["push", "-u", "origin", branch.as_str()]);
        command
    })?;

    get_project_git_status(project_id)
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

    state.settings.default_install_path = request.default_install_path;
    state.settings.default_project_path = request.default_project_path;
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
