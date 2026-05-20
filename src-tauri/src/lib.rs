mod activity;
mod diagnostics;
mod editors;
mod filesystem;
mod git;
mod godot;
mod models;
mod paths;
mod projects;
mod releases;
mod settings;
mod state;
mod system;
mod workspace;

use activity::*;
use diagnostics::*;
use editors::*;
use git::*;
use projects::*;
use releases::*;
use settings::*;
use state::*;
use system::*;
use workspace::*;

#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::env;

use tauri::Manager;

#[cfg(target_os = "macos")]
use tauri::{
    menu::{AboutMetadata, Menu, MenuItem, PredefinedMenuItem, Submenu},
    Emitter,
};

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
            comments: Some(
                "Desktop hub for Godot editors, projects, releases, and Git workflows.".into(),
            ),
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
            get_workspace_diagnostics,
            get_release_cache_info,
            clear_release_cache,
            read_activity_log,
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
