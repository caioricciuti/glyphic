use crate::commands::settings;

/// Hooks live inside settings.json under the "hooks" key.
/// These commands are thin wrappers around settings read/write.

#[tauri::command]
pub fn get_hooks(
    scope: String,
    project_path: Option<String>,
) -> Result<serde_json::Value, String> {
    let settings = settings::read_settings(scope, project_path)?;
    Ok(settings.get("hooks").cloned().unwrap_or(serde_json::json!({})))
}

#[tauri::command]
pub fn set_hooks(
    scope: String,
    project_path: Option<String>,
    hooks: serde_json::Value,
) -> Result<(), String> {
    let mut current = settings::read_settings(scope.clone(), project_path.clone())?;
    let obj = current.as_object_mut().ok_or("settings is not an object")?;

    if hooks.as_object().map_or(false, |h| h.is_empty()) {
        obj.remove("hooks");
    } else {
        obj.insert("hooks".to_string(), hooks);
    }

    settings::write_settings(scope, project_path, current)
}
