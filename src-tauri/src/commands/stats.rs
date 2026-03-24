use crate::paths;

#[tauri::command]
pub fn get_stats() -> Result<serde_json::Value, String> {
    let path = paths::stats_cache_path();

    if !path.exists() {
        return Ok(serde_json::json!({}));
    }

    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("failed to read stats: {e}"))?;

    serde_json::from_str(&content)
        .map_err(|e| format!("failed to parse stats: {e}"))
}
