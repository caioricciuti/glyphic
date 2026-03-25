use crate::commands::settings;
use crate::paths;

/// MCP servers live inside settings.json under the "mcpServers" key.

#[tauri::command]
pub fn list_mcp_servers(
    scope: String,
    project_path: Option<String>,
) -> Result<serde_json::Value, String> {
    let settings = settings::read_settings(scope, project_path)?;
    Ok(settings.get("mcpServers").cloned().unwrap_or(serde_json::json!({})))
}

#[tauri::command]
pub fn upsert_mcp_server(
    scope: String,
    project_path: Option<String>,
    name: String,
    config: serde_json::Value,
) -> Result<(), String> {
    let mut current = settings::read_settings(scope.clone(), project_path.clone())?;
    let obj = current.as_object_mut().ok_or("settings is not an object")?;

    let servers = obj
        .entry("mcpServers")
        .or_insert_with(|| serde_json::json!({}));

    let servers_obj = servers.as_object_mut().ok_or("mcpServers is not an object")?;
    servers_obj.insert(name, config);

    settings::write_settings(scope, project_path, current)
}

#[tauri::command]
pub fn delete_mcp_server(
    scope: String,
    project_path: Option<String>,
    name: String,
) -> Result<(), String> {
    let mut current = settings::read_settings(scope.clone(), project_path.clone())?;
    let obj = current.as_object_mut().ok_or("settings is not an object")?;

    if let Some(servers) = obj.get_mut("mcpServers") {
        if let Some(servers_obj) = servers.as_object_mut() {
            servers_obj.remove(&name);
            if servers_obj.is_empty() {
                obj.remove("mcpServers");
            }
        }
    }

    settings::write_settings(scope, project_path, current)
}

#[tauri::command]
pub fn get_cloud_mcps() -> Result<Vec<String>, String> {
    let path = paths::claude_home().join("mcp-needs-auth-cache.json");
    if !path.exists() {
        return Ok(vec![]);
    }
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("failed to read: {e}"))?;
    let data: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("failed to parse: {e}"))?;

    Ok(data
        .as_object()
        .map(|obj| obj.keys().cloned().collect())
        .unwrap_or_default())
}
