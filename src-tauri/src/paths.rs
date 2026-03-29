use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// Try to resolve a binary via a shell's login environment.
fn resolve_via_shell(shell: &str, binary: &str) -> Option<String> {
    std::process::Command::new(shell)
        .args(["-lc", &format!("which {binary}")])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| {
            let path = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if !path.is_empty() && Path::new(&path).exists() {
                Some(path)
            } else {
                None
            }
        })
}

/// Resolve the full path to the `claude` CLI binary.
/// macOS GUI apps don't inherit the user's shell PATH, so we try multiple strategies:
/// 1. Ask zsh (macOS default shell, sources .zshrc/.zprofile)
/// 2. Ask sh (POSIX fallback, sources .profile)
/// 3. Check known installation directories directly
/// Cached for the lifetime of the process.
pub fn claude_bin() -> &'static str {
    static BIN: OnceLock<String> = OnceLock::new();
    BIN.get_or_init(|| {
        // Try shells: zsh sources .zshrc (where most users add PATH), sh sources .profile
        for shell in ["zsh", "sh"] {
            if let Some(path) = resolve_via_shell(shell, "claude") {
                return path;
            }
        }
        // Check known install locations explicitly
        if let Some(home) = dirs::home_dir() {
            for subpath in [".local/bin/claude", ".claude/local/bin/claude"] {
                let p = home.join(subpath);
                if p.exists() {
                    return p.to_string_lossy().to_string();
                }
            }
        }
        for p in ["/usr/local/bin/claude", "/opt/homebrew/bin/claude"] {
            if Path::new(p).exists() {
                return p.to_string();
            }
        }
        "claude".to_string()
    })
}

/// Return an enriched PATH that includes common user binary directories.
/// macOS GUI apps inherit a minimal PATH (/usr/bin:/bin:/usr/sbin:/sbin).
/// This adds ~/.local/bin, ~/.bun/bin, /opt/homebrew/bin, etc. so subprocess
/// spawns can find user-installed tools (claude, git, node, bun, etc.).
pub fn enriched_path() -> String {
    let home = dirs::home_dir().unwrap_or_default();
    let extra_dirs = [
        home.join(".local/bin"),
        home.join(".bun/bin"),
        home.join(".cargo/bin"),
        PathBuf::from("/opt/homebrew/bin"),
        PathBuf::from("/usr/local/bin"),
    ];
    let system_path = std::env::var("PATH").unwrap_or_default();
    let mut parts: Vec<String> = extra_dirs.iter()
        .filter(|p| p.is_dir())
        .map(|p| p.to_string_lossy().to_string())
        .collect();
    parts.push(system_path);
    parts.join(":")
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

pub fn project_mcp_json_path(project_path: &str) -> PathBuf {
    PathBuf::from(project_path).join(".mcp.json")
}

pub fn claude_desktop_config_path() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_default()
        .join("Claude")
        .join("claude_desktop_config.json")
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
