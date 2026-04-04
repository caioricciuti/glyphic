use crate::paths;
use serde::Serialize;
use std::fs;
use std::path::PathBuf;

#[derive(Serialize)]
pub struct SkillInfo {
    pub name: String,
    pub path: String,
    pub scope: String,
    pub content: String,
}

fn skills_dir_for_scope(scope: &str, project_path: Option<&str>) -> Result<PathBuf, String> {
    match scope {
        "global" => Ok(paths::global_skills_dir()),
        "project" => {
            let pp = project_path.ok_or("project_path required")?;
            Ok(PathBuf::from(pp).join(".claude").join("skills"))
        }
        _ => Err(format!("invalid scope: {scope}")),
    }
}

fn agents_dir_for_scope(scope: &str, project_path: Option<&str>) -> Result<PathBuf, String> {
    match scope {
        "global" => Ok(paths::global_agents_dir()),
        "project" => {
            let pp = project_path.ok_or("project_path required")?;
            Ok(PathBuf::from(pp).join(".claude").join("agents"))
        }
        _ => Err(format!("invalid scope: {scope}")),
    }
}

fn list_definitions(dir: &PathBuf, scope: &str, marker: &str) -> Result<Vec<SkillInfo>, String> {
    if !dir.exists() {
        return Ok(vec![]);
    }

    let mut items = Vec::new();

    let entries = fs::read_dir(dir)
        .map_err(|e| format!("failed to read dir: {e}"))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("failed to read entry: {e}"))?;
        if entry.file_type().is_ok_and(|ft| ft.is_dir()) {
            let name = entry.file_name().to_string_lossy().to_string();
            let marker_path = entry.path().join(marker);

            let content = if marker_path.exists() {
                fs::read_to_string(&marker_path).unwrap_or_default()
            } else {
                String::new()
            };

            items.push(SkillInfo {
                name,
                path: entry.path().to_string_lossy().to_string(),
                scope: scope.to_string(),
                content,
            });
        }
    }

    items.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(items)
}

#[tauri::command]
pub fn list_skills(scope: String, project_path: Option<String>) -> Result<Vec<SkillInfo>, String> {
    let dir = skills_dir_for_scope(&scope, project_path.as_deref())?;
    list_definitions(&dir, &scope, "SKILL.md")
}

#[tauri::command]
pub fn list_agents(scope: String, project_path: Option<String>) -> Result<Vec<SkillInfo>, String> {
    let dir = agents_dir_for_scope(&scope, project_path.as_deref())?;
    list_definitions(&dir, &scope, "AGENT.md")
}

#[tauri::command]
pub fn write_skill(
    scope: String,
    project_path: Option<String>,
    name: String,
    content: String,
) -> Result<(), String> {
    let dir = skills_dir_for_scope(&scope, project_path.as_deref())?;
    let skill_dir = dir.join(&name);
    fs::create_dir_all(&skill_dir)
        .map_err(|e| format!("failed to create skill dir: {e}"))?;

    fs::write(skill_dir.join("SKILL.md"), content)
        .map_err(|e| format!("failed to write skill: {e}"))
}

#[tauri::command]
pub fn write_agent(
    scope: String,
    project_path: Option<String>,
    name: String,
    content: String,
) -> Result<(), String> {
    let dir = agents_dir_for_scope(&scope, project_path.as_deref())?;
    let agent_dir = dir.join(&name);
    fs::create_dir_all(&agent_dir)
        .map_err(|e| format!("failed to create agent dir: {e}"))?;

    fs::write(agent_dir.join("AGENT.md"), content)
        .map_err(|e| format!("failed to write agent: {e}"))
}

#[tauri::command]
pub fn delete_skill(
    scope: String,
    project_path: Option<String>,
    name: String,
) -> Result<(), String> {
    let dir = skills_dir_for_scope(&scope, project_path.as_deref())?;
    let skill_dir = dir.join(&name);
    if skill_dir.exists() {
        fs::remove_dir_all(&skill_dir)
            .map_err(|e| format!("failed to delete skill: {e}"))?;
    }
    Ok(())
}

#[tauri::command]
pub fn delete_agent(
    scope: String,
    project_path: Option<String>,
    name: String,
) -> Result<(), String> {
    let dir = agents_dir_for_scope(&scope, project_path.as_deref())?;
    let agent_dir = dir.join(&name);
    if agent_dir.exists() {
        fs::remove_dir_all(&agent_dir)
            .map_err(|e| format!("failed to delete agent: {e}"))?;
    }
    Ok(())
}
