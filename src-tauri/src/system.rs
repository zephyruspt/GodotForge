use std::env;

use crate::models::{LegalDocument, SystemProfile};

#[tauri::command]
pub(crate) fn read_legal_document(document: String) -> Result<LegalDocument, String> {
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
pub(crate) fn detect_system_profile() -> SystemProfile {
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
