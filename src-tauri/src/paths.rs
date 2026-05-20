use std::{
    env, fs,
    hash::{Hash, Hasher},
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

pub(crate) fn home_dir() -> PathBuf {
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

pub(crate) fn default_install_path() -> PathBuf {
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

pub(crate) fn default_project_path() -> PathBuf {
    documents_dir().join("GodotForge").join("Projects")
}

pub(crate) fn config_path() -> PathBuf {
    let base = env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home_dir().join(".config"));

    base.join("godot-forge").join("hub-state.json")
}

pub(crate) fn release_cache_path(repositories: &[String], limit: usize, page: usize) -> PathBuf {
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

pub(crate) fn now_id(prefix: &str) -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default();

    format!("{prefix}-{millis}")
}
