use crate::paths;
use std::path::PathBuf;

fn resolve_settings_path(scope: &str, project_path: Option<&str>) -> Result<PathBuf, String> {
    match scope {
        "global" => Ok(paths::global_settings_path()),
        "project" => {
            let pp = project_path.ok_or("project_path required for project scope")?;
            Ok(paths::project_settings_path(pp))
        }
        "local" => {
            let pp = project_path.ok_or("project_path required for local scope")?;
            Ok(paths::project_local_settings_path(pp))
        }
        _ => Err(format!("invalid scope: {scope}")),
    }
}

#[tauri::command]
pub fn read_settings(
    scope: String,
    project_path: Option<String>,
) -> Result<serde_json::Value, String> {
    let path = resolve_settings_path(&scope, project_path.as_deref())?;

    if !path.exists() {
        return Ok(serde_json::json!({}));
    }

    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("failed to read {}: {e}", path.display()))?;

    serde_json::from_str(&content)
        .map_err(|e| format!("failed to parse {}: {e}", path.display()))
}

#[tauri::command]
pub fn write_settings(
    scope: String,
    project_path: Option<String>,
    settings: serde_json::Value,
) -> Result<(), String> {
    let path = resolve_settings_path(&scope, project_path.as_deref())?;

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("failed to create directory: {e}"))?;
    }

    let content = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("failed to serialize settings: {e}"))?;

    std::fs::write(&path, content)
        .map_err(|e| format!("failed to write {}: {e}", path.display()))
}
