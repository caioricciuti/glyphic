use crate::paths;
use serde::Serialize;
use std::fs;
use std::path::Path;

#[derive(Serialize)]
pub struct DiskUsageEntry {
    pub name: String,
    pub path: String,
    pub size_bytes: u64,
    pub size_display: String,
    pub description: String,
    pub safe_to_delete: bool,
}

#[derive(Serialize)]
pub struct DiskUsageReport {
    pub total_bytes: u64,
    pub total_display: String,
    pub entries: Vec<DiskUsageEntry>,
}

fn dir_size(path: &Path) -> u64 {
    if !path.exists() {
        return 0;
    }
    let mut size = 0u64;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                size += entry.metadata().map(|m| m.len()).unwrap_or(0);
            } else if path.is_dir() {
                size += dir_size(&path);
            }
        }
    }
    size
}

fn format_size(bytes: u64) -> String {
    if bytes >= 1_073_741_824 {
        format!("{:.1} GB", bytes as f64 / 1_073_741_824.0)
    } else if bytes >= 1_048_576 {
        format!("{:.1} MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{bytes} B")
    }
}

#[tauri::command]
pub fn get_disk_usage() -> Result<DiskUsageReport, String> {
    let home = paths::claude_home();

    let items = vec![
        ("telemetry", "Failed telemetry events", true),
        ("file-history", "File change tracking cache", true),
        ("shell-snapshots", "Shell environment snapshots", true),
        ("todos", "Session todo lists", true),
        ("paste-cache", "Clipboard paste cache", true),
        ("image-cache", "Cached images", true),
        ("plans", "Generated plan files", true),
        ("cache", "General cache", true),
        ("debug", "Debug logs", true),
        ("projects", "Session data & memory", false),
        ("plugins", "Installed plugins", false),
    ];

    let mut entries: Vec<DiskUsageEntry> = Vec::new();
    let mut total = 0u64;

    for (dir_name, description, safe) in items {
        let path = home.join(dir_name);
        let size = dir_size(&path);
        if size > 0 {
            total += size;
            entries.push(DiskUsageEntry {
                name: dir_name.to_string(),
                path: path.to_string_lossy().to_string(),
                size_bytes: size,
                size_display: format_size(size),
                description: description.to_string(),
                safe_to_delete: safe,
            });
        }
    }

    // Add standalone files
    for file in &["history.jsonl", "stats-cache.json"] {
        let path = home.join(file);
        if path.exists() {
            let size = path.metadata().map(|m| m.len()).unwrap_or(0);
            total += size;
            entries.push(DiskUsageEntry {
                name: file.to_string(),
                path: path.to_string_lossy().to_string(),
                size_bytes: size,
                size_display: format_size(size),
                description: if *file == "history.jsonl" { "Session history" } else { "Stats cache" }.to_string(),
                safe_to_delete: false,
            });
        }
    }

    entries.sort_by_key(|e| std::cmp::Reverse(e.size_bytes));

    Ok(DiskUsageReport {
        total_bytes: total,
        total_display: format_size(total),
        entries,
    })
}

#[tauri::command]
pub fn cleanup_directory(name: String) -> Result<u64, String> {
    let home = paths::claude_home();
    let path = home.join(&name);

    // Safety: only allow cleaning known safe directories
    let safe_dirs = [
        "telemetry", "file-history", "shell-snapshots",
        "todos", "paste-cache", "image-cache", "plans", "cache", "debug",
    ];

    if !safe_dirs.contains(&name.as_str()) {
        return Err(format!("Cannot clean '{}' — not a safe target", name));
    }

    if !path.exists() {
        return Ok(0);
    }

    let size = dir_size(&path);

    // Delete contents but keep the directory
    if let Ok(entries) = fs::read_dir(&path) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_file() {
                let _ = fs::remove_file(&p);
            } else if p.is_dir() {
                let _ = fs::remove_dir_all(&p);
            }
        }
    }

    Ok(size)
}
