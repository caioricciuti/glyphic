use crate::paths;
use std::fs;
use std::process::Command;

#[tauri::command]
pub fn get_installed_plugins() -> Result<serde_json::Value, String> {
    let path = paths::claude_home().join("plugins").join("installed_plugins.json");

    if !path.exists() {
        return Ok(serde_json::json!({ "version": 2, "plugins": [] }));
    }

    let content = fs::read_to_string(&path)
        .map_err(|e| format!("failed to read installed plugins: {e}"))?;

    serde_json::from_str(&content)
        .map_err(|e| format!("failed to parse: {e}"))
}

#[tauri::command]
pub fn get_blocked_plugins() -> Result<serde_json::Value, String> {
    let path = paths::claude_home().join("plugins").join("blocked_plugins.json");

    if !path.exists() {
        return Ok(serde_json::json!([]));
    }

    let content = fs::read_to_string(&path)
        .map_err(|e| format!("failed to read blocked plugins: {e}"))?;

    serde_json::from_str(&content)
        .map_err(|e| format!("failed to parse: {e}"))
}

#[tauri::command]
pub fn get_marketplace_plugins() -> Result<serde_json::Value, String> {
    let marketplaces_dir = paths::claude_home().join("plugins").join("marketplaces");

    if !marketplaces_dir.exists() {
        return Ok(serde_json::json!([]));
    }

    let mut all_plugins = Vec::new();

    let entries = fs::read_dir(&marketplaces_dir)
        .map_err(|e| format!("failed to read marketplaces dir: {e}"))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("failed to read entry: {e}"))?;
        let marketplace_path = entry.path().join(".claude-plugin").join("marketplace.json");

        if marketplace_path.exists() {
            let content = fs::read_to_string(&marketplace_path)
                .map_err(|e| format!("failed to read marketplace: {e}"))?;

            if let Ok(data) = serde_json::from_str::<serde_json::Value>(&content) {
                all_plugins.push(data);
            }
        }
    }

    Ok(serde_json::json!(all_plugins))
}

#[tauri::command]
pub fn get_install_counts() -> Result<serde_json::Value, String> {
    let path = paths::claude_home()
        .join("plugins")
        .join("install-counts-cache.json");

    if !path.exists() {
        return Ok(serde_json::json!([]));
    }

    let content = fs::read_to_string(&path)
        .map_err(|e| format!("failed to read install counts: {e}"))?;

    serde_json::from_str(&content)
        .map_err(|e| format!("failed to parse: {e}"))
}

#[tauri::command]
pub fn install_plugin(name: String) -> Result<String, String> {
    let output = Command::new("claude")
        .args(["plugin", "install", &name])
        .output()
        .map_err(|e| format!("failed to run claude: {e}"))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(format!("install failed: {stderr}"))
    }
}
