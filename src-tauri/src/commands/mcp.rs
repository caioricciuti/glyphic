use crate::commands::settings;
use crate::paths;

/// MCP servers live inside settings.json under the "mcpServers" key,
/// or in .mcp.json at the project root (local scope, the default for `claude mcp add`).

/// Read .mcp.json from project root. Handles both formats:
/// - Format A (standard): { "server-name": { config } }
/// - Format B (legacy):   { "mcpServers": { "server-name": { config } } }
fn read_mcp_json(project_path: &str) -> Result<serde_json::Value, String> {
    let path = paths::project_mcp_json_path(project_path);
    if !path.exists() {
        return Ok(serde_json::json!({}));
    }
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("failed to read {}: {e}", path.display()))?;
    let data: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("failed to parse {}: {e}", path.display()))?;

    if let Some(mcp_servers) = data.get("mcpServers") {
        Ok(mcp_servers.clone())
    } else {
        Ok(data)
    }
}

/// Write .mcp.json in Format A (direct server map, no wrapper).
fn write_mcp_json(project_path: &str, servers: serde_json::Value) -> Result<(), String> {
    let path = paths::project_mcp_json_path(project_path);
    let content = serde_json::to_string_pretty(&servers)
        .map_err(|e| format!("failed to serialize: {e}"))?;
    std::fs::write(&path, content)
        .map_err(|e| format!("failed to write {}: {e}", path.display()))
}

/// Read Claude Desktop config and extract mcpServers.
fn read_desktop_config() -> Result<(serde_json::Value, serde_json::Value), String> {
    let path = paths::claude_desktop_config_path();
    if !path.exists() {
        return Ok((serde_json::json!({}), serde_json::json!({})));
    }
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("failed to read {}: {e}", path.display()))?;
    let full: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("failed to parse {}: {e}", path.display()))?;
    let servers = full.get("mcpServers").cloned().unwrap_or(serde_json::json!({}));
    Ok((full, servers))
}

/// Write mcpServers back into Claude Desktop config, preserving other keys.
fn write_desktop_config(mut full: serde_json::Value, servers: serde_json::Value) -> Result<(), String> {
    let path = paths::claude_desktop_config_path();
    let obj = full.as_object_mut().ok_or("desktop config is not an object")?;
    if servers.as_object().map_or(true, |s| s.is_empty()) {
        obj.remove("mcpServers");
    } else {
        obj.insert("mcpServers".to_string(), servers);
    }
    let content = serde_json::to_string_pretty(&full)
        .map_err(|e| format!("failed to serialize: {e}"))?;
    std::fs::write(&path, content)
        .map_err(|e| format!("failed to write {}: {e}", path.display()))
}

#[tauri::command]
pub fn list_mcp_servers(
    scope: String,
    project_path: Option<String>,
) -> Result<serde_json::Value, String> {
    if scope == "mcp-local" {
        let pp = project_path.as_deref().ok_or("project_path required for local scope")?;
        return read_mcp_json(pp);
    }
    if scope == "desktop" {
        let (_full, servers) = read_desktop_config()?;
        return Ok(servers);
    }
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
    if scope == "mcp-local" {
        let pp = project_path.as_deref().ok_or("project_path required for local scope")?;
        let mut servers = read_mcp_json(pp)?;
        let obj = servers.as_object_mut().ok_or("mcp.json is not an object")?;
        obj.insert(name, config);
        return write_mcp_json(pp, servers);
    }
    if scope == "desktop" {
        let (full, mut servers) = read_desktop_config()?;
        let obj = servers.as_object_mut().ok_or("desktop mcpServers is not an object")?;
        obj.insert(name, config);
        return write_desktop_config(full, servers);
    }

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
    if scope == "mcp-local" {
        let pp = project_path.as_deref().ok_or("project_path required for local scope")?;
        let mut servers = read_mcp_json(pp)?;
        if let Some(obj) = servers.as_object_mut() {
            obj.remove(&name);
        }
        return write_mcp_json(pp, servers);
    }
    if scope == "desktop" {
        let (full, mut servers) = read_desktop_config()?;
        if let Some(obj) = servers.as_object_mut() {
            obj.remove(&name);
        }
        return write_desktop_config(full, servers);
    }

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
