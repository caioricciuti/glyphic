use crate::paths;
use serde::Serialize;

#[derive(Serialize)]
pub struct ProjectInfo {
    pub hash: String,
    pub path: String,
    pub has_memory: bool,
    pub exists: bool,
}

#[tauri::command]
pub fn list_projects() -> Result<Vec<ProjectInfo>, String> {
    let projects_dir = paths::projects_dir();

    if !projects_dir.exists() {
        return Ok(vec![]);
    }

    let mut projects = Vec::new();

    let entries = std::fs::read_dir(&projects_dir)
        .map_err(|e| format!("failed to read projects dir: {e}"))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("failed to read entry: {e}"))?;
        let file_name = entry.file_name().to_string_lossy().to_string();

        if entry.file_type().map_or(false, |ft| ft.is_dir()) {
            let resolved_path = paths::project_hash_to_path(&file_name);
            let memory_dir = paths::memory_dir(&file_name);
            let exists = std::path::Path::new(&resolved_path).is_dir();

            projects.push(ProjectInfo {
                hash: file_name,
                path: resolved_path,
                has_memory: memory_dir.exists(),
                exists,
            });
        }
    }

    // Sort: existing projects first, then alphabetically
    projects.sort_by(|a, b| {
        b.exists.cmp(&a.exists).then(a.path.cmp(&b.path))
    });

    Ok(projects)
}
