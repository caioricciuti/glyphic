use crate::paths;
use serde::Serialize;
use std::fs;

#[derive(Serialize)]
pub struct InstructionFile {
    pub path: String,
    pub content: String,
    pub exists: bool,
    pub imports: Vec<String>,
}

fn extract_imports(content: &str) -> Vec<String> {
    let mut imports = Vec::new();
    for line in content.lines() {
        // Match @path patterns (not email addresses)
        for word in line.split_whitespace() {
            if word.starts_with('@') && word.len() > 1 {
                let rest = &word[1..];
                // Skip if it looks like an email (contains @) or is a mention
                if !rest.contains('@') && (rest.contains('/') || rest.contains('.') || rest.ends_with(".md")) {
                    imports.push(rest.to_string());
                }
            }
        }
    }
    imports
}

#[tauri::command]
pub fn read_instructions(scope: String, project_path: Option<String>) -> Result<InstructionFile, String> {
    let path = match scope.as_str() {
        "global" => paths::global_instructions_path(),
        "project" => {
            let pp = project_path.ok_or("project_path required")?;
            std::path::PathBuf::from(&pp).join("CLAUDE.md")
        }
        "project-dot" => {
            let pp = project_path.ok_or("project_path required")?;
            std::path::PathBuf::from(&pp).join(".claude").join("CLAUDE.md")
        }
        "local" => {
            let pp = project_path.ok_or("project_path required")?;
            std::path::PathBuf::from(&pp).join("CLAUDE.local.md")
        }
        _ => return Err(format!("invalid scope: {scope}")),
    };

    if !path.exists() {
        return Ok(InstructionFile {
            path: path.to_string_lossy().to_string(),
            content: String::new(),
            exists: false,
            imports: vec![],
        });
    }

    let content = fs::read_to_string(&path)
        .map_err(|e| format!("failed to read: {e}"))?;

    let imports = extract_imports(&content);

    Ok(InstructionFile {
        path: path.to_string_lossy().to_string(),
        content,
        exists: true,
        imports,
    })
}

#[tauri::command]
pub fn write_instructions(scope: String, project_path: Option<String>, content: String) -> Result<(), String> {
    let path = match scope.as_str() {
        "global" => paths::global_instructions_path(),
        "project" => {
            let pp = project_path.ok_or("project_path required")?;
            std::path::PathBuf::from(&pp).join("CLAUDE.md")
        }
        "project-dot" => {
            let pp = project_path.ok_or("project_path required")?;
            let p = std::path::PathBuf::from(&pp).join(".claude").join("CLAUDE.md");
            if let Some(parent) = p.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("failed to create dir: {e}"))?;
            }
            p
        }
        "local" => {
            let pp = project_path.ok_or("project_path required")?;
            std::path::PathBuf::from(&pp).join("CLAUDE.local.md")
        }
        _ => return Err(format!("invalid scope: {scope}")),
    };

    fs::write(&path, content)
        .map_err(|e| format!("failed to write: {e}"))
}

#[tauri::command]
pub fn read_referenced_file(base_path: String, reference: String) -> Result<String, String> {
    // Resolve reference relative to the base file's directory
    let base = std::path::PathBuf::from(&base_path);
    let base_dir = base.parent().unwrap_or(&base);

    // Handle ~ prefix
    let ref_path = if reference.starts_with('~') {
        let home = dirs::home_dir().ok_or("no home dir")?;
        home.join(&reference[2..]) // skip ~/
    } else {
        base_dir.join(&reference)
    };

    if !ref_path.exists() {
        return Err(format!("file not found: {}", ref_path.display()));
    }

    fs::read_to_string(&ref_path)
        .map_err(|e| format!("failed to read: {e}"))
}
