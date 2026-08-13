use crate::paths;
use regex::Regex;
use serde::Serialize;
use std::path::PathBuf;
use std::sync::OnceLock;

fn resolve_settings_path(scope: &str, project_path: Option<&str>) -> Result<PathBuf, String> {
    match scope {
        "global" => Ok(paths::global_settings_path()),
        "project" => {
            let pp = project_path.ok_or("project_path required for project scope")?;
            Ok(paths::project_settings_path(pp))
        }
        "local" => {
            let pp = project_path.ok_or("project_path required for local scope")?;
            Ok(paths::project_local_settings_path(pp))
        }
        _ => Err(format!("invalid scope: {scope}")),
    }
}

#[tauri::command]
pub fn read_settings(
    scope: String,
    project_path: Option<String>,
) -> Result<serde_json::Value, String> {
    let path = resolve_settings_path(&scope, project_path.as_deref())?;

    if !path.exists() {
        return Ok(serde_json::json!({}));
    }

    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("failed to read {}: {e}", path.display()))?;

    serde_json::from_str(&content)
        .map_err(|e| format!("failed to parse {}: {e}", path.display()))
}

#[tauri::command]
pub fn write_settings(
    scope: String,
    project_path: Option<String>,
    settings: serde_json::Value,
) -> Result<(), String> {
    let path = resolve_settings_path(&scope, project_path.as_deref())?;

    let content = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("failed to serialize settings: {e}"))?;

    paths::write_atomic(&path, &content)
}

#[derive(Serialize, Clone)]
pub struct ModelOption {
    pub value: String,
    pub label: String,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ClaudeCapabilities {
    pub version: Option<String>,
    pub effort_levels: Vec<String>,
    pub model_aliases: Vec<ModelOption>,
    pub model_pinned: Vec<ModelOption>,
}

/// Run the installed `claude` CLI and capture stdout (None on failure/empty).
fn run_claude(args: &[&str]) -> Option<String> {
    let out = std::process::Command::new(paths::claude_bin())
        .args(args)
        .env("PATH", paths::enriched_path())
        .output()
        .ok()?;
    let s = String::from_utf8_lossy(&out.stdout).to_string();
    if s.trim().is_empty() {
        None
    } else {
        Some(s)
    }
}

/// "claude-opus-4-8" -> "Claude Opus 4.8 (most capable)"
fn pretty_model_label(id: &str) -> String {
    let parts: Vec<&str> = id.split('-').collect();
    if parts.len() >= 4 && parts[0] == "claude" {
        let mut fam = parts[1].to_string();
        if let Some(first) = fam.get_mut(0..1) {
            first.make_ascii_uppercase();
        }
        let suffix = match parts[1] {
            "opus" => " (most capable)",
            "sonnet" => " (fast + capable)",
            "haiku" => " (fastest)",
            _ => "",
        };
        return format!("Claude {} {}.{}{}", fam, parts[2], parts[3], suffix);
    }
    id.to_string()
}

/// Source model + effort options from the installed Claude CLI so the UI never
/// drifts from what the CLI actually supports. Effort levels and the latest opus
/// pin are parsed from `claude --help`; aliases never go stale.
fn compute_capabilities() -> ClaudeCapabilities {
    let version = run_claude(&["--version"]).map(|s| s.trim().to_string());
    let help = run_claude(&["--help"]).unwrap_or_default();

    // `--effort <level>` documents the levels as "(low, medium, high, xhigh, max)".
    let effort_levels = Regex::new(r"--effort[\s\S]*?\(([^)]+)\)")
        .ok()
        .and_then(|re| re.captures(&help).map(|c| c[1].to_string()))
        .map(|list| {
            list.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty() && s.chars().all(|c| c.is_ascii_lowercase()))
                .collect::<Vec<_>>()
        })
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| {
            ["low", "medium", "high", "xhigh", "max"]
                .iter()
                .map(|s| s.to_string())
                .collect()
        });

    // Latest opus pin from the `--model` example, e.g. "claude-opus-4-8".
    let opus_pin = Regex::new(r"claude-opus-\d+-\d+")
        .ok()
        .and_then(|re| re.find(&help).map(|m| m.as_str().to_string()))
        .unwrap_or_else(|| "claude-opus-4-8".to_string());

    let model_aliases = vec![
        ModelOption { value: "opus".into(), label: "Opus (latest)".into() },
        ModelOption { value: "sonnet".into(), label: "Sonnet (latest)".into() },
        ModelOption { value: "haiku".into(), label: "Haiku (latest)".into() },
    ];

    // Pinned versions. Opus is parsed live from the CLI; sonnet/haiku are the
    // current pins — bump them here when Anthropic ships new ones (the aliases
    // above stay current automatically regardless).
    let model_pinned = [opus_pin.as_str(), "claude-sonnet-4-6", "claude-haiku-4-5"]
        .iter()
        .map(|id| ModelOption { value: id.to_string(), label: pretty_model_label(id) })
        .collect();

    ClaudeCapabilities { version, effort_levels, model_aliases, model_pinned }
}

#[tauri::command(async)]
pub fn get_claude_capabilities() -> ClaudeCapabilities {
    static CACHE: OnceLock<ClaudeCapabilities> = OnceLock::new();
    CACHE.get_or_init(compute_capabilities).clone()
}
