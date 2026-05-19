use serde::{Deserialize, Serialize};
use std::{
    env,
    fs,
    io,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
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
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct HubState {
    editors: Vec<GodotEditor>,
    projects: Vec<GodotProject>,
    settings: HubSettings,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase", deserialize = "snake_case"))]
struct GodotReleaseAsset {
    id: u64,
    name: String,
    size: u64,
    browser_download_url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase", deserialize = "snake_case"))]
struct GodotRelease {
    id: u64,
    name: Option<String>,
    tag_name: String,
    prerelease: bool,
    published_at: Option<String>,
    html_url: String,
    assets: Vec<GodotReleaseAsset>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct SystemProfile {
    os: String,
    arch: String,
    godot_platform: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AddEditorRequest {
    name: String,
    version: String,
    executable_path: String,
    install_path: String,
    architecture: String,
    make_default: bool,
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
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DownloadEditorRequest {
    release_tag: String,
    asset_name: String,
    asset_url: String,
    install_path: Option<String>,
    make_default: bool,
}

fn home_dir() -> PathBuf {
    env::var_os("HOME")
        .or_else(|| env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn config_path() -> PathBuf {
    let base = env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home_dir().join(".config"));

    base.join("godot-forge").join("hub-state.json")
}

fn now_id(prefix: &str) -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default();

    format!("{prefix}-{millis}")
}

fn default_state() -> HubState {
    let home = home_dir();

    HubState {
        editors: Vec::new(),
        projects: Vec::new(),
        settings: HubSettings {
            default_install_path: home
                .join("Applications")
                .join("Godot")
                .to_string_lossy()
                .to_string(),
            default_project_path: home
                .join("Godot")
                .join("Projects")
                .to_string_lossy()
                .to_string(),
        },
    }
}

fn read_state() -> Result<HubState, String> {
    let path = config_path();

    if !path.exists() {
        return Ok(default_state());
    }

    let data = fs::read_to_string(path).map_err(|error| error.to_string())?;
    serde_json::from_str(&data).map_err(|error| error.to_string())
}

fn write_state(state: &HubState) -> Result<(), String> {
    let path = config_path();

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }

    let data = serde_json::to_string_pretty(state).map_err(|error| error.to_string())?;
    fs::write(path, data).map_err(|error| error.to_string())
}

fn project_name_from_path(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.trim().is_empty())
        .unwrap_or("Projeto Godot")
        .to_string()
}

fn http_client() -> Result<reqwest::blocking::Client, String> {
    reqwest::blocking::Client::builder()
        .user_agent("GodotForge/0.1 (+https://github.com/godotengine/godot)")
        .build()
        .map_err(|error| error.to_string())
}

fn version_from_tag(tag: &str) -> String {
    tag.trim_start_matches("godot-").trim_start_matches('v').to_string()
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
        "desconhecida".into()
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
        let output_path = destination.join(entry.mangled_name());

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

#[tauri::command]
fn load_hub_state() -> Result<HubState, String> {
    read_state()
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
fn fetch_godot_releases(limit: Option<usize>) -> Result<Vec<GodotRelease>, String> {
    let max_items = limit.unwrap_or(8).clamp(1, 20);
    let url = format!("https://api.github.com/repos/godotengine/godot/releases?per_page={max_items}");

    http_client()?
        .get(url)
        .header("Accept", "application/vnd.github+json")
        .send()
        .map_err(|error| error.to_string())?
        .error_for_status()
        .map_err(|error| error.to_string())?
        .json::<Vec<GodotRelease>>()
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn download_godot_editor(request: DownloadEditorRequest) -> Result<HubState, String> {
    let mut state = read_state()?;
    let install_root = request
        .install_path
        .filter(|path| !path.trim().is_empty())
        .unwrap_or_else(|| state.settings.default_install_path.clone());
    let install_root = PathBuf::from(install_root);
    let release_folder = install_root.join(request.release_tag.replace('/', "-"));
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
        find_godot_executable(&release_folder)
            .ok_or_else(|| "Download concluido, mas nao encontrei o executavel da Godot.".to_string())?
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

    write_state(&state)?;
    Ok(state)
}

#[tauri::command]
fn add_editor(request: AddEditorRequest) -> Result<HubState, String> {
    let mut state = read_state()?;
    let executable = PathBuf::from(request.executable_path.trim());

    if !executable.exists() {
        return Err("O executavel informado nao existe.".into());
    }

    if request.make_default || state.editors.is_empty() {
        for editor in &mut state.editors {
            editor.is_default = false;
        }
    }

    state.editors.push(GodotEditor {
        id: now_id("editor"),
        name: request.name.trim().to_string(),
        version: request.version.trim().to_string(),
        executable_path: executable.to_string_lossy().to_string(),
        install_path: request.install_path.trim().to_string(),
        architecture: request.architecture.trim().to_string(),
        is_default: request.make_default || state.editors.is_empty(),
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
        return Err("Editor nao encontrado.".into());
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
    let path = PathBuf::from(request.path.trim());

    if !path.join("project.godot").exists() {
        return Err("Nao encontrei um project.godot nesse caminho.".into());
    }

    let clean_path = path.to_string_lossy().to_string();
    if state.projects.iter().any(|project| project.path == clean_path) {
        return Err("Esse projeto ja esta cadastrado.".into());
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
    let project_name = request.name.trim();

    if project_name.is_empty() {
        return Err("Informe um nome para o projeto.".into());
    }

    let root_path = PathBuf::from(request.root_path.trim());
    let project_path = root_path.join(project_name);

    if project_path.exists() {
        return Err("Ja existe uma pasta com esse nome.".into());
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
            "; Engine configuration file.\n; Criado pelo Godot Forge.\n\nconfig_version=5\n\n[application]\n\nconfig/name=\"{}\"\nrun/main_scene=\"res://Main.tscn\"\n",
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
fn toggle_project_favorite(project_id: String) -> Result<HubState, String> {
    let mut state = read_state()?;

    if let Some(project) = state.projects.iter_mut().find(|project| project.id == project_id) {
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
        .ok_or_else(|| "Projeto nao encontrado.".to_string())?;

    let editor = project
        .editor_id
        .as_ref()
        .and_then(|editor_id| state.editors.iter().find(|editor| &editor.id == editor_id))
        .or_else(|| state.editors.iter().find(|editor| editor.is_default))
        .cloned()
        .ok_or_else(|| "Cadastre um editor da Godot antes de abrir projetos.".to_string())?;

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            load_hub_state,
            detect_system_profile,
            fetch_godot_releases,
            download_godot_editor,
            save_settings,
            add_editor,
            remove_editor,
            set_default_editor,
            import_project,
            create_project,
            remove_project,
            toggle_project_favorite,
            launch_project
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
