use crate::paths;
use serde::Serialize;
use std::fs;
use std::path::PathBuf;

#[derive(Serialize)]
pub struct RuleFile {
    pub name: String,
    pub path: String,
    pub scope: String,
    pub content: String,
    pub paths_filter: Vec<String>,
}

fn rules_dir_for_scope(scope: &str, project_path: Option<&str>) -> Result<PathBuf, String> {
    match scope {
        "global" => Ok(paths::global_rules_dir()),
        "project" => {
            let pp = project_path.ok_or("project_path required")?;
            Ok(PathBuf::from(pp).join(".claude").join("rules"))
        }
        _ => Err(format!("invalid scope: {scope}")),
    }
}

fn parse_rule_frontmatter(content: &str) -> (Vec<String>, String) {
    if !content.starts_with("---") {
        return (vec![], content.to_string());
    }

    if let Some(end) = content[3..].find("---") {
        let front = &content[3..3 + end];
        let body = content[3 + end + 3..].trim_start().to_string();

        let mut paths = Vec::new();
        let mut in_paths = false;

        for line in front.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("paths:") {
                in_paths = true;
                continue;
            }
            if in_paths {
                if trimmed.starts_with("- ") {
                    let val = trimmed[2..].trim().trim_matches('"').trim_matches('\'');
                    paths.push(val.to_string());
                } else if !trimmed.is_empty() {
                    in_paths = false;
                }
            }
        }

        (paths, body)
    } else {
        (vec![], content.to_string())
    }
}

#[tauri::command]
pub fn list_rules(scope: String, project_path: Option<String>) -> Result<Vec<RuleFile>, String> {
    let dir = rules_dir_for_scope(&scope, project_path.as_deref())?;

    if !dir.exists() {
        return Ok(vec![]);
    }

    let mut rules = Vec::new();

    fn walk_dir(dir: &PathBuf, scope: &str, rules: &mut Vec<RuleFile>) -> Result<(), String> {
        let entries = fs::read_dir(dir)
            .map_err(|e| format!("failed to read dir: {e}"))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("failed to read entry: {e}"))?;
            let path = entry.path();

            if path.is_dir() {
                walk_dir(&path, scope, rules)?;
            } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
                let content = fs::read_to_string(&path)
                    .map_err(|e| format!("failed to read {}: {e}", path.display()))?;

                let (paths_filter, body) = parse_rule_frontmatter(&content);

                let name = path
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default();

                rules.push(RuleFile {
                    name,
                    path: path.to_string_lossy().to_string(),
                    scope: scope.to_string(),
                    content: body,
                    paths_filter,
                });
            }
        }
        Ok(())
    }

    walk_dir(&dir, &scope, &mut rules)?;
    rules.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(rules)
}

#[tauri::command]
pub fn write_rule(
    scope: String,
    project_path: Option<String>,
    filename: String,
    paths_filter: Vec<String>,
    content: String,
) -> Result<(), String> {
    let dir = rules_dir_for_scope(&scope, project_path.as_deref())?;
    fs::create_dir_all(&dir)
        .map_err(|e| format!("failed to create rules dir: {e}"))?;

    let path = dir.join(&filename);

    let mut output = String::new();

    if !paths_filter.is_empty() {
        output.push_str("---\npaths:\n");
        for p in &paths_filter {
            output.push_str(&format!("  - \"{p}\"\n"));
        }
        output.push_str("---\n\n");
    }

    output.push_str(&content);

    fs::write(&path, output)
        .map_err(|e| format!("failed to write rule: {e}"))
}

#[tauri::command]
pub fn delete_rule(
    scope: String,
    project_path: Option<String>,
    filename: String,
) -> Result<(), String> {
    let dir = rules_dir_for_scope(&scope, project_path.as_deref())?;
    let path = dir.join(&filename);

    if path.exists() {
        fs::remove_file(&path)
            .map_err(|e| format!("failed to delete rule: {e}"))?;
    }

    Ok(())
}
