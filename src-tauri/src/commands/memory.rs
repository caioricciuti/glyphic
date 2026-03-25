use crate::paths;
use serde::Serialize;
use std::fs;

#[derive(Serialize)]
pub struct MemoryFile {
    pub filename: String,
    pub path: String,
    pub content: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub memory_type: Option<String>,
}

fn parse_frontmatter(content: &str) -> (Option<String>, Option<String>, Option<String>, String) {
    if !content.starts_with("---") {
        return (None, None, None, content.to_string());
    }

    if let Some(end) = content[3..].find("---") {
        let front = &content[3..3 + end];
        let body = content[3 + end + 3..].trim_start().to_string();

        let mut name = None;
        let mut description = None;
        let mut memory_type = None;

        for line in front.lines() {
            let line = line.trim();
            if let Some(val) = line.strip_prefix("name:") {
                name = Some(val.trim().to_string());
            } else if let Some(val) = line.strip_prefix("description:") {
                description = Some(val.trim().to_string());
            } else if let Some(val) = line.strip_prefix("type:") {
                memory_type = Some(val.trim().to_string());
            }
        }

        (name, description, memory_type, body)
    } else {
        (None, None, None, content.to_string())
    }
}

#[tauri::command]
pub fn list_memory_files(project_hash: String) -> Result<Vec<MemoryFile>, String> {
    let dir = paths::memory_dir(&project_hash);

    if !dir.exists() {
        return Ok(vec![]);
    }

    let mut files = Vec::new();

    let entries = fs::read_dir(&dir)
        .map_err(|e| format!("failed to read memory dir: {e}"))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("failed to read entry: {e}"))?;
        let path = entry.path();

        if path.extension().and_then(|e| e.to_str()) == Some("md") {
            let filename = entry.file_name().to_string_lossy().to_string();
            let content = fs::read_to_string(&path)
                .map_err(|e| format!("failed to read {}: {e}", path.display()))?;

            let (name, description, memory_type, body) = parse_frontmatter(&content);

            files.push(MemoryFile {
                filename,
                path: path.to_string_lossy().to_string(),
                content: body,
                name,
                description,
                memory_type,
            });
        }
    }

    files.sort_by(|a, b| a.filename.cmp(&b.filename));
    Ok(files)
}

#[tauri::command]
pub fn read_memory_file(project_hash: String, filename: String) -> Result<MemoryFile, String> {
    let path = paths::memory_dir(&project_hash).join(&filename);

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()));
    }

    let content = fs::read_to_string(&path)
        .map_err(|e| format!("failed to read: {e}"))?;

    let (name, description, memory_type, body) = parse_frontmatter(&content);

    Ok(MemoryFile {
        filename,
        path: path.to_string_lossy().to_string(),
        content: body,
        name,
        description,
        memory_type,
    })
}

#[tauri::command]
pub fn write_memory_file(
    project_hash: String,
    filename: String,
    name: Option<String>,
    description: Option<String>,
    memory_type: Option<String>,
    content: String,
) -> Result<(), String> {
    let dir = paths::memory_dir(&project_hash);
    fs::create_dir_all(&dir)
        .map_err(|e| format!("failed to create memory dir: {e}"))?;

    let path = dir.join(&filename);

    let mut output = String::new();

    if name.is_some() || description.is_some() || memory_type.is_some() {
        output.push_str("---\n");
        if let Some(ref n) = name {
            output.push_str(&format!("name: {n}\n"));
        }
        if let Some(ref d) = description {
            output.push_str(&format!("description: {d}\n"));
        }
        if let Some(ref t) = memory_type {
            output.push_str(&format!("type: {t}\n"));
        }
        output.push_str("---\n\n");
    }

    output.push_str(&content);

    fs::write(&path, output)
        .map_err(|e| format!("failed to write: {e}"))
}

#[tauri::command]
pub fn delete_memory_file(project_hash: String, filename: String) -> Result<(), String> {
    let path = paths::memory_dir(&project_hash).join(&filename);

    if path.exists() {
        fs::remove_file(&path)
            .map_err(|e| format!("failed to delete: {e}"))?;
    }

    Ok(())
}
