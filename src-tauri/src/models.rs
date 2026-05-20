use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GodotEditor {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) version: String,
    pub(crate) executable_path: String,
    pub(crate) install_path: String,
    pub(crate) architecture: String,
    pub(crate) is_default: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GodotProject {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) path: String,
    pub(crate) editor_id: Option<String>,
    pub(crate) favorite: bool,
    pub(crate) last_opened: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct HubSettings {
    pub(crate) default_install_path: String,
    pub(crate) default_project_path: String,
    #[serde(default)]
    pub(crate) release_repositories: Vec<String>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub(crate) github_token: String,
    #[serde(default, skip_serializing)]
    pub(crate) release_repository: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct HubState {
    pub(crate) editors: Vec<GodotEditor>,
    pub(crate) projects: Vec<GodotProject>,
    pub(crate) settings: HubSettings,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GodotReleaseAsset {
    pub(crate) id: u64,
    pub(crate) name: String,
    pub(crate) size: u64,
    #[serde(alias = "browser_download_url")]
    pub(crate) browser_download_url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GodotRelease {
    pub(crate) id: u64,
    pub(crate) name: Option<String>,
    #[serde(alias = "tag_name")]
    pub(crate) tag_name: String,
    pub(crate) prerelease: bool,
    #[serde(alias = "published_at")]
    pub(crate) published_at: Option<String>,
    #[serde(alias = "html_url")]
    pub(crate) html_url: String,
    pub(crate) assets: Vec<GodotReleaseAsset>,
    #[serde(default)]
    pub(crate) source_repository: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ReleaseCache {
    pub(crate) repositories: Vec<String>,
    pub(crate) limit: usize,
    #[serde(default = "default_release_page")]
    pub(crate) page: usize,
    pub(crate) fetched_at: u64,
    pub(crate) releases: Vec<GodotRelease>,
}

fn default_release_page() -> usize {
    1
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SystemProfile {
    pub(crate) os: String,
    pub(crate) arch: String,
    pub(crate) godot_platform: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GitStatus {
    pub(crate) available: bool,
    pub(crate) is_repo: bool,
    pub(crate) branch: Option<String>,
    pub(crate) remote: Option<String>,
    pub(crate) changed_files: usize,
    pub(crate) untracked_files: usize,
    pub(crate) summary: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GitLogEntry {
    pub(crate) hash: String,
    pub(crate) author: String,
    pub(crate) relative_date: String,
    pub(crate) subject: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GitBranch {
    pub(crate) name: String,
    pub(crate) current: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ImportProjectRequest {
    pub(crate) name: Option<String>,
    pub(crate) path: String,
    pub(crate) editor_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreateProjectRequest {
    pub(crate) name: String,
    pub(crate) root_path: String,
    pub(crate) editor_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UpdateSettingsRequest {
    pub(crate) default_install_path: String,
    pub(crate) default_project_path: String,
    pub(crate) release_repositories: Vec<String>,
    pub(crate) github_token: String,
    #[serde(default)]
    pub(crate) migrate_existing_paths: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DownloadEditorRequest {
    pub(crate) release_tag: String,
    pub(crate) release_repository: Option<String>,
    pub(crate) asset_name: String,
    pub(crate) asset_url: String,
    pub(crate) install_path: Option<String>,
    pub(crate) make_default: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RegisterDiscoveredEditorRequest {
    pub(crate) executable_path: String,
    pub(crate) install_path: String,
    pub(crate) name: String,
    pub(crate) version: String,
    pub(crate) architecture: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RegisterDiscoveredProjectRequest {
    pub(crate) path: String,
    pub(crate) name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MoveProjectRequest {
    pub(crate) project_id: String,
    pub(crate) destination_path: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GitBranchRequest {
    pub(crate) project_id: String,
    pub(crate) branch_name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GitRemoteRequest {
    pub(crate) project_id: String,
    pub(crate) remote_url: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LegalDocument {
    pub(crate) title: String,
    pub(crate) body: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DiscoveredEditor {
    pub(crate) name: String,
    pub(crate) version: String,
    pub(crate) architecture: String,
    pub(crate) executable_path: String,
    pub(crate) install_path: String,
    pub(crate) registered: bool,
    pub(crate) corrupt: bool,
    pub(crate) reason: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DiscoveredProject {
    pub(crate) name: String,
    pub(crate) path: String,
    pub(crate) registered: bool,
    pub(crate) corrupt: bool,
    pub(crate) reason: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WorkspaceScan {
    pub(crate) editors: Vec<DiscoveredEditor>,
    pub(crate) projects: Vec<DiscoveredProject>,
}
