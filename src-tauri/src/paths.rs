use std::path::PathBuf;

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
    hash.replace('-', "/")
}
