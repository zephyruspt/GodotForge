use std::{fs, path::PathBuf, process::Command};

use crate::{models::*, state::read_state};

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

#[tauri::command]
pub(crate) fn get_project_git_status(project_id: String) -> Result<GitStatus, String> {
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
pub(crate) fn init_project_git(project_id: String) -> Result<GitStatus, String> {
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
pub(crate) fn get_project_git_log(project_id: String) -> Result<Vec<GitLogEntry>, String> {
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
pub(crate) fn list_project_git_branches(project_id: String) -> Result<Vec<GitBranch>, String> {
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
pub(crate) fn create_project_git_branch(request: GitBranchRequest) -> Result<GitStatus, String> {
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
pub(crate) fn checkout_project_git_branch(request: GitBranchRequest) -> Result<GitStatus, String> {
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
pub(crate) fn set_project_git_remote(request: GitRemoteRequest) -> Result<GitStatus, String> {
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
pub(crate) fn push_project_git_branch(project_id: String) -> Result<GitStatus, String> {
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
