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
}

#[derive(Serialize)]
pub struct SessionEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    pub timestamp: Option<String>,
    pub content: serde_json::Value,
}

#[tauri::command]
pub fn list_sessions() -> Result<Vec<SessionSummary>, String> {
    let projects_dir = paths::projects_dir();
    if !projects_dir.exists() {
        return Ok(vec![]);
    }

    let mut sessions = Vec::new();

    let project_entries = std::fs::read_dir(&projects_dir)
        .map_err(|e| format!("failed to read projects: {e}"))?;

    for project_entry in project_entries {
        let project_entry = project_entry.map_err(|e| format!("{e}"))?;
        if !project_entry.file_type().map_or(false, |ft| ft.is_dir()) {
            continue;
        }
        let project_hash = project_entry.file_name().to_string_lossy().to_string();
        let project_path = paths::project_hash_to_path(&project_hash);
        let project_dir = project_entry.path();

        // Find .jsonl files directly in project dir (not in subdirs like subagents/)
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
            if path.extension().and_then(|e| e.to_str()) != Some("jsonl") {
                continue;
            }

            let file = match std::fs::File::open(&path) {
                Ok(f) => f,
                Err(_) => continue,
            };
            let reader = std::io::BufReader::new(file);

            let mut entry_count = 0u32;
            let mut user_messages = 0u32;
            let mut tool_calls = 0u32;
            let mut first_timestamp: Option<String> = None;
            let mut last_timestamp: Option<String> = None;
            let mut first_message: Option<String> = None;

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

                entry_count += 1;
                let entry_type = parsed.get("type").and_then(|t| t.as_str()).unwrap_or("");

                if let Some(ts) = parsed.get("timestamp").and_then(|t| t.as_str()) {
                    if first_timestamp.is_none() {
                        first_timestamp = Some(ts.to_string());
                    }
                    last_timestamp = Some(ts.to_string());
                }

                if entry_type == "user" {
                    user_messages += 1;
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
                                    tool_calls += 1;
                                }
                            }
                        }
                    }
                }
            }

            if entry_count < 3 {
                continue; // Skip tiny/empty sessions
            }

            let id = path
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();

            sessions.push(SessionSummary {
                id,
                project_hash: project_hash.clone(),
                project_path: project_path.clone(),
                path: path.to_string_lossy().to_string(),
                entry_count,
                user_messages,
                tool_calls,
                first_timestamp,
                last_timestamp,
                first_message,
            });
        }
    }

    // Sort by timestamp descending (newest first)
    sessions.sort_by(|a, b| b.first_timestamp.cmp(&a.first_timestamp));

    Ok(sessions)
}

#[tauri::command]
pub fn load_session(path: String) -> Result<Vec<SessionEvent>, String> {
    let file = std::fs::File::open(&path)
        .map_err(|e| format!("failed to open session: {e}"))?;
    let reader = std::io::BufReader::new(file);

    let mut events = Vec::new();

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

        // Skip non-essential types
        if event_type == "file-history-snapshot" || event_type == "last-prompt" {
            continue;
        }

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

    Ok(events)
}
