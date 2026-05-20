use std::{fs, path::PathBuf, process::Command};

use crate::{
    models::{DiagnosticCheck, WorkspaceDiagnostics},
    releases::{
        current_release_repositories, github_token, release_cache_info_for,
        RELEASE_CATALOG_PAGE_SIZE,
    },
    state::read_state,
};

fn directory_check(key: &str, label: &str, path: &str) -> DiagnosticCheck {
    let path = PathBuf::from(path);
    let ok = path.exists()
        && path.is_dir()
        && fs::metadata(&path)
            .map(|metadata| !metadata.permissions().readonly())
            .unwrap_or(false);
    let detail = if ok {
        format!("{} exists and is writable.", path.to_string_lossy())
    } else if path.exists() {
        format!(
            "{} exists but is not writable or is not a folder.",
            path.to_string_lossy()
        )
    } else {
        format!("{} is missing or is not a folder.", path.to_string_lossy())
    };

    DiagnosticCheck {
        key: key.into(),
        label: label.into(),
        ok,
        detail,
    }
}

fn git_diagnostic_check() -> DiagnosticCheck {
    match Command::new("git").arg("--version").output() {
        Ok(output) if output.status.success() => DiagnosticCheck {
            key: "git".into(),
            label: "Git".into(),
            ok: true,
            detail: String::from_utf8_lossy(&output.stdout).trim().to_string(),
        },
        _ => DiagnosticCheck {
            key: "git".into(),
            label: "Git".into(),
            ok: false,
            detail: "Git was not found on this system.".into(),
        },
    }
}

#[tauri::command]
pub(crate) fn get_workspace_diagnostics() -> Result<WorkspaceDiagnostics, String> {
    let state = read_state()?;
    let repositories = current_release_repositories(&state.settings)?;
    let token_configured = github_token(&state.settings).is_some();
    let token_detail = if token_configured {
        "A GitHub token is configured for release requests."
    } else {
        "No GitHub token is configured; public API rate limits may apply."
    };
    let cache = release_cache_info_for(&repositories, RELEASE_CATALOG_PAGE_SIZE, 1);

    Ok(WorkspaceDiagnostics {
        checks: vec![
            git_diagnostic_check(),
            directory_check(
                "install-path",
                "Editor install path",
                &state.settings.default_install_path,
            ),
            directory_check(
                "project-path",
                "Project workspace path",
                &state.settings.default_project_path,
            ),
            DiagnosticCheck {
                key: "github-token".into(),
                label: "GitHub token".into(),
                ok: token_configured,
                detail: token_detail.into(),
            },
        ],
        cache,
    })
}
