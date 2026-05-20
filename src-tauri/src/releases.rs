use std::{env, fs};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use crate::{
    activity::{current_unix_seconds, record_activity},
    godot::{normalize_release_repositories, OFFICIAL_RELEASE_REPOSITORY},
    models::{GodotRelease, HubSettings, ReleaseCache, ReleaseCacheInfo},
    paths::{config_dir, release_cache_path},
    secrets,
    state::read_state,
};

pub(crate) const RELEASE_CACHE_TTL_SECONDS: u64 = 60 * 60 * 6;
pub(crate) const RELEASE_CATALOG_PAGE_SIZE: usize = 100;
const RELEASE_CATALOG_MAX_PAGES: usize = 5;

fn read_release_cache(repositories: &[String], limit: usize, page: usize) -> Option<ReleaseCache> {
    let data = fs::read_to_string(release_cache_path(repositories, limit, page)).ok()?;
    let cache: ReleaseCache = serde_json::from_str(&data).ok()?;

    if cache.repositories == repositories && cache.limit == limit && cache.page == page {
        Some(cache)
    } else {
        None
    }
}

pub(crate) fn current_release_repositories(settings: &HubSettings) -> Result<Vec<String>, String> {
    let mut repositories = vec![OFFICIAL_RELEASE_REPOSITORY.to_string()];
    repositories.extend(normalize_release_repositories(
        &settings.release_repositories,
    )?);
    Ok(repositories)
}

pub(crate) fn release_cache_info_for(
    repositories: &[String],
    limit: usize,
    page: usize,
) -> ReleaseCacheInfo {
    let path = release_cache_path(repositories, limit, page);
    let Some(cache) = read_release_cache(repositories, limit, page) else {
        return ReleaseCacheInfo {
            exists: path.exists(),
            path: path.to_string_lossy().to_string(),
            fetched_at: None,
            age_seconds: None,
            release_count: 0,
        };
    };

    ReleaseCacheInfo {
        exists: true,
        path: path.to_string_lossy().to_string(),
        fetched_at: Some(cache.fetched_at),
        age_seconds: Some(current_unix_seconds().saturating_sub(cache.fetched_at)),
        release_count: cache.releases.len(),
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
    fs::write(&path, data).map_err(|error| error.to_string())?;
    restrict_file_permissions(&path);
    Ok(())
}

#[cfg(unix)]
fn restrict_file_permissions(path: &std::path::Path) {
    let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o600));
}

#[cfg(not(unix))]
fn restrict_file_permissions(_path: &std::path::Path) {}

pub(crate) fn http_client() -> Result<reqwest::blocking::Client, String> {
    reqwest::blocking::Client::builder()
        .user_agent("GodotForge/0.1")
        .build()
        .map_err(|error| error.to_string())
}

pub(crate) fn github_token(settings: &HubSettings) -> Option<String> {
    env::var("GITHUB_TOKEN")
        .or_else(|_| env::var("GH_TOKEN"))
        .ok()
        .or_else(|| secrets::github_token())
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
pub(crate) fn get_release_cache_info() -> Result<ReleaseCacheInfo, String> {
    let state = read_state()?;
    let repositories = current_release_repositories(&state.settings)?;
    Ok(release_cache_info_for(
        &repositories,
        RELEASE_CATALOG_PAGE_SIZE,
        1,
    ))
}

#[tauri::command]
pub(crate) fn clear_release_cache() -> Result<ReleaseCacheInfo, String> {
    let state = read_state()?;
    let repositories = current_release_repositories(&state.settings)?;

    clear_release_cache_files();
    record_activity("info", "Release cache cleared.");
    Ok(release_cache_info_for(
        &repositories,
        RELEASE_CATALOG_PAGE_SIZE,
        1,
    ))
}

pub(crate) fn clear_release_cache_files() {
    if let Ok(entries) = fs::read_dir(config_dir()) {
        for entry in entries.flatten() {
            let path = entry.path();
            let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            if name.starts_with("release-cache-") && name.ends_with(".json") {
                let _ = fs::remove_file(path);
            }
        }
    }
}

#[tauri::command]
pub(crate) fn fetch_godot_releases(
    limit: Option<usize>,
    page: Option<usize>,
) -> Result<Vec<GodotRelease>, String> {
    let state = read_state()?;
    let repositories = current_release_repositories(&state.settings)?;
    let max_items = limit
        .unwrap_or(RELEASE_CATALOG_PAGE_SIZE)
        .clamp(1, RELEASE_CATALOG_PAGE_SIZE);
    let start_page = page.unwrap_or(1).clamp(1, 50);

    if let Some(releases) = fresh_cached_releases(&repositories, max_items, start_page) {
        record_activity(
            "info",
            format!("Loaded {} releases from cache.", releases.len()),
        );
        return Ok(releases);
    }

    let client = http_client()?;
    let token = github_token(&state.settings);
    let mut releases = Vec::new();

    for repository in &repositories {
        for page in start_page..(start_page + RELEASE_CATALOG_MAX_PAGES) {
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
                    if let Some(cached_releases) =
                        stale_cached_releases(&repositories, max_items, start_page)
                    {
                        record_activity(
                            "warning",
                            "GitHub request failed; using stale release cache.",
                        );
                        return Ok(cached_releases);
                    }

                    return Err(github_release_error(error));
                }
            };
            let response = match response.error_for_status() {
                Ok(response) => response,
                Err(error) => {
                    if let Some(cached_releases) =
                        stale_cached_releases(&repositories, max_items, start_page)
                    {
                        record_activity(
                            "warning",
                            "GitHub returned an error; using stale release cache.",
                        );
                        return Ok(cached_releases);
                    }

                    return Err(github_release_error(error));
                }
            };

            let mut repository_releases = match response.json::<Vec<GodotRelease>>() {
                Ok(repository_releases) => repository_releases,
                Err(error) => {
                    if let Some(cached_releases) =
                        stale_cached_releases(&repositories, max_items, start_page)
                    {
                        record_activity(
                            "warning",
                            "GitHub response could not be decoded; using stale release cache.",
                        );
                        return Ok(cached_releases);
                    }

                    return Err(github_release_error(error));
                }
            };

            if repository_releases.is_empty() {
                break;
            }

            for release in &mut repository_releases {
                release.source_repository = repository.clone();
            }

            let fetched_count = repository_releases.len();
            releases.extend(repository_releases);

            if fetched_count < max_items {
                break;
            }
        }
    }

    releases.sort_by(|left, right| right.published_at.cmp(&left.published_at));
    let _ = write_release_cache(repositories, max_items, start_page, releases.clone());
    record_activity(
        "info",
        format!("Fetched {} releases from GitHub.", releases.len()),
    );
    Ok(releases)
}
