use crate::paths;
use serde::Serialize;
use std::io::BufRead;

#[derive(Serialize)]
pub struct SessionSummary {
    pub id: String,
    pub project_hash: String,
    pub project_path: String,
    pub path: String,
    pub entry_count: u32,
    pub user_messages: u32,
    pub tool_calls: u32,
    pub first_timestamp: Option<String>,
    pub last_timestamp: Option<String>,
    pub first_message: Option<String>,
    pub modified_at: f64,
}

#[derive(Serialize)]
pub struct SessionListResult {
    pub sessions: Vec<SessionSummary>,
    pub total: usize,
    pub has_more: bool,
}

#[derive(Serialize)]
pub struct SessionEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    pub timestamp: Option<String>,
    pub content: serde_json::Value,
}

/// Quickly scan a JSONL file — read only first 20 lines for metadata, use file size for entry estimate
fn quick_scan(path: &std::path::Path) -> Option<SessionSummary> {
    let metadata = std::fs::metadata(path).ok()?;
    let file_size = metadata.len();

    // Skip tiny files
    if file_size < 200 {
        return None;
    }

    let modified = metadata
        .modified()
        .ok()?
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_secs_f64();

    let file = std::fs::File::open(path).ok()?;
    let reader = std::io::BufReader::new(file);

    let mut first_timestamp: Option<String> = None;
    let mut first_message: Option<String> = None;
    let mut user_count = 0u32;
    let mut tool_count = 0u32;
    let mut line_count = 0u32;
    let mut last_timestamp: Option<String> = None;

    // Read only first 30 lines for quick metadata
    for (i, line) in reader.lines().enumerate() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };
        if line.trim().is_empty() {
            continue;
        }
        line_count += 1;

        let parsed: serde_json::Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let entry_type = parsed.get("type").and_then(|t| t.as_str()).unwrap_or("");

        if let Some(ts) = parsed.get("timestamp").and_then(|t| t.as_str()) {
            if first_timestamp.is_none() {
                first_timestamp = Some(ts.to_string());
            }
            last_timestamp = Some(ts.to_string());
        }

        if entry_type == "user" {
            user_count += 1;
            if first_message.is_none() {
                if let Some(msg) = parsed.get("message") {
                    if let Some(content) = msg.get("content") {
                        let text = if let Some(s) = content.as_str() {
                            s.to_string()
                        } else if let Some(arr) = content.as_array() {
                            arr.iter()
                                .filter_map(|item| {
                                    if item.get("type")?.as_str()? == "text" {
                                        item.get("text")?.as_str().map(|s| s.to_string())
                                    } else {
                                        None
                                    }
                                })
                                .next()
                                .unwrap_or_default()
                        } else {
                            String::new()
                        };
                        if !text.is_empty() {
                            first_message = Some(text.chars().take(100).collect());
                        }
                    }
                }
            }
        }

        if entry_type == "assistant" {
            if let Some(msg) = parsed.get("message") {
                if let Some(content) = msg.get("content").and_then(|c| c.as_array()) {
                    for item in content {
                        if item.get("type").and_then(|t| t.as_str()) == Some("tool_use") {
                            tool_count += 1;
                        }
                    }
                }
            }
        }

        // After 30 lines, estimate the rest from file size
        if i >= 29 {
            break;
        }
    }

    // Estimate total entries from file size and avg line size
    let avg_line_bytes = if line_count > 0 {
        // Rough estimate: read 30 lines, extrapolate
        file_size / line_count.max(1) as u64
    } else {
        500 // default estimate
    };
    let estimated_entries = (file_size / avg_line_bytes.max(1)) as u32;

    // Scale user_messages and tool_calls by the ratio
    let scale = if line_count > 0 {
        estimated_entries as f32 / line_count as f32
    } else {
        1.0
    };

    if line_count < 3 {
        return None;
    }

    let id = path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();

    let project_dir = path.parent()?;
    let project_hash = project_dir.file_name()?.to_string_lossy().to_string();
    let project_path = paths::project_hash_to_path(&project_hash);

    Some(SessionSummary {
        id,
        project_hash,
        project_path,
        path: path.to_string_lossy().to_string(),
        entry_count: estimated_entries,
        user_messages: (user_count as f32 * scale) as u32,
        tool_calls: (tool_count as f32 * scale) as u32,
        first_timestamp,
        last_timestamp,
        first_message,
        modified_at: modified,
    })
}

#[tauri::command]
pub fn list_sessions(limit: Option<usize>, offset: Option<usize>) -> Result<SessionListResult, String> {
    let projects_dir = paths::projects_dir();
    if !projects_dir.exists() {
        return Ok(SessionListResult { sessions: vec![], total: 0, has_more: false });
    }

    // Collect all JSONL file paths first (fast — just readdir)
    let mut jsonl_paths: Vec<std::path::PathBuf> = Vec::new();

    let project_entries = std::fs::read_dir(&projects_dir)
        .map_err(|e| format!("failed to read projects: {e}"))?;

    for project_entry in project_entries {
        let project_entry = match project_entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        if !project_entry.file_type().map_or(false, |ft| ft.is_dir()) {
            continue;
        }
        let project_dir = project_entry.path();

        let entries = match std::fs::read_dir(&project_dir) {
            Ok(e) => e,
            Err(_) => continue,
        };

        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("jsonl") {
                jsonl_paths.push(path);
            }
        }
    }

    // Sort by modified time (newest first) — using file metadata, no parsing needed
    jsonl_paths.sort_by(|a, b| {
        let ma = std::fs::metadata(a).and_then(|m| m.modified()).unwrap_or(std::time::SystemTime::UNIX_EPOCH);
        let mb = std::fs::metadata(b).and_then(|m| m.modified()).unwrap_or(std::time::SystemTime::UNIX_EPOCH);
        mb.cmp(&ma)
    });

    let total = jsonl_paths.len();
    let off = offset.unwrap_or(0);
    let lim = limit.unwrap_or(10);

    // Only scan the files we need (paginated)
    let sessions: Vec<SessionSummary> = jsonl_paths
        .iter()
        .skip(off)
        .take(lim)
        .filter_map(|path| quick_scan(path))
        .collect();

    Ok(SessionListResult {
        sessions,
        total,
        has_more: off + lim < total,
    })
}

#[derive(Serialize)]
pub struct SessionLoadResult {
    pub events: Vec<SessionEvent>,
    pub total: usize,
    pub has_more: bool,
}

#[tauri::command]
pub fn load_session(path: String, limit: Option<usize>, offset: Option<usize>) -> Result<SessionLoadResult, String> {
    let file = std::fs::File::open(&path)
        .map_err(|e| format!("failed to open session: {e}"))?;
    let reader = std::io::BufReader::new(file);

    let off = offset.unwrap_or(0);
    let lim = limit.unwrap_or(50);
    let mut events = Vec::new();
    let mut total = 0usize;

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };
        if line.trim().is_empty() {
            continue;
        }

        let parsed: serde_json::Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let event_type = parsed
            .get("type")
            .and_then(|t| t.as_str())
            .unwrap_or("unknown")
            .to_string();

        if event_type == "file-history-snapshot" || event_type == "last-prompt" || event_type == "progress" {
            continue;
        }

        if total >= off && events.len() < lim {
            let timestamp = parsed
                .get("timestamp")
                .and_then(|t| t.as_str())
                .map(|s| s.to_string());

            events.push(SessionEvent {
                event_type,
                timestamp,
                content: parsed,
            });
        }

        total += 1;
    }

    Ok(SessionLoadResult {
        events,
        total,
        has_more: off + lim < total,
    })
}
