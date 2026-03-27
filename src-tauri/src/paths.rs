use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// Resolve the full path to the `claude` CLI binary.
/// macOS GUI apps don't inherit the user's shell PATH, so we ask a login shell to find it.
/// Cached for the lifetime of the process.
pub fn claude_bin() -> &'static str {
    static BIN: OnceLock<String> = OnceLock::new();
    BIN.get_or_init(|| {
        std::process::Command::new("sh")
            .args(["-lc", "which claude"])
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "claude".to_string())
    })
}

pub fn claude_home() -> PathBuf {
    dirs::home_dir()
        .expect("could not resolve home directory")
        .join(".claude")
}

pub fn global_settings_path() -> PathBuf {
    claude_home().join("settings.json")
}

pub fn project_settings_path(project_path: &str) -> PathBuf {
    PathBuf::from(project_path)
        .join(".claude")
        .join("settings.json")
}

pub fn project_local_settings_path(project_path: &str) -> PathBuf {
    PathBuf::from(project_path)
        .join(".claude")
        .join("settings.local.json")
}

pub fn stats_cache_path() -> PathBuf {
    claude_home().join("stats-cache.json")
}

pub fn projects_dir() -> PathBuf {
    claude_home().join("projects")
}

pub fn memory_dir(project_hash: &str) -> PathBuf {
    projects_dir().join(project_hash).join("memory")
}

pub fn global_instructions_path() -> PathBuf {
    claude_home().join("CLAUDE.md")
}

pub fn global_skills_dir() -> PathBuf {
    claude_home().join("skills")
}

pub fn global_agents_dir() -> PathBuf {
    claude_home().join("agents")
}

pub fn global_rules_dir() -> PathBuf {
    claude_home().join("rules")
}

pub fn project_hash_to_path(hash: &str) -> String {
    // Naive: replace all `-` with `/`
    let naive = hash.replace('-', "/");
    if Path::new(&naive).is_dir() {
        return naive;
    }

    // Smart: try to find a real path by grouping segments
    let segments: Vec<&str> = hash.split('-').filter(|s| !s.is_empty()).collect();
    if let Some(path) = resolve_segments(&segments, 0, "/") {
        return path;
    }

    naive
}

fn resolve_segments(segments: &[&str], idx: usize, current: &str) -> Option<String> {
    if idx >= segments.len() {
        return if Path::new(current).is_dir() {
            Some(current.to_string())
        } else {
            None
        };
    }

    // Try joining segments with `-` (longer matches first to prefer real dir names)
    for end in (idx + 1..=segments.len()).rev() {
        let joined = segments[idx..end].join("-");
        let candidate = if current == "/" {
            format!("/{joined}")
        } else {
            format!("{current}/{joined}")
        };

        if Path::new(&candidate).is_dir() {
            if let Some(result) = resolve_segments(segments, end, &candidate) {
                return Some(result);
            }
        }
    }

    None
}
