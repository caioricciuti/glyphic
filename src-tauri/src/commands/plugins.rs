use crate::paths;
use std::fs;
use std::process::Command;

/// Run `claude plugin ...` / `claude plugin marketplace ...` and return stdout.
/// Args are passed as separate argv entries (no shell), so the only injection
/// vector is a value being parsed as a flag; callers validate names/scopes.
fn run_plugin_cli(args: &[&str], project_path: Option<&str>) -> Result<String, String> {
    let mut cmd = Command::new(paths::claude_bin());
    cmd.args(args).env("PATH", paths::enriched_path());
    if let Some(pp) = project_path {
        cmd.current_dir(pp);
    }
    let output = cmd
        .output()
        .map_err(|e| format!("failed to run claude: {e}"))?;
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if output.status.success() {
        Ok(stdout)
    } else {
        Err(if stderr.is_empty() { stdout } else { stderr })
    }
}

fn validate_name(name: &str) -> Result<(), String> {
    if name.is_empty() || name.starts_with('-') || name.contains(char::is_whitespace) {
        return Err(format!("invalid plugin name: {name:?}"));
    }
    Ok(())
}

fn validate_scope(scope: &Option<String>) -> Result<(), String> {
    if let Some(s) = scope {
        if !matches!(s.as_str(), "user" | "project" | "local") {
            return Err(format!("invalid scope: {s:?}"));
        }
    }
    Ok(())
}

/// Build `[verb, name, (-s scope)]` after validating both.
fn scoped_args<'a>(
    verb: &'a str,
    name: &'a str,
    scope: &'a Option<String>,
) -> Result<Vec<&'a str>, String> {
    validate_name(name)?;
    validate_scope(scope)?;
    let mut args = vec!["plugin", verb, name];
    if let Some(s) = scope {
        args.push("-s");
        args.push(s.as_str());
    }
    Ok(args)
}

#[tauri::command]
pub fn get_installed_plugins() -> Result<serde_json::Value, String> {
    let path = paths::claude_home().join("plugins").join("installed_plugins.json");

    if !path.exists() {
        return Ok(serde_json::json!({ "version": 2, "plugins": [] }));
    }

    let content = fs::read_to_string(&path)
        .map_err(|e| format!("failed to read installed plugins: {e}"))?;

    serde_json::from_str(&content)
        .map_err(|e| format!("failed to parse: {e}"))
}

/// Authoritative list from the CLI: id, version, scope, enabled, installPath.
#[tauri::command(async)]
pub fn list_plugins() -> Result<serde_json::Value, String> {
    let out = run_plugin_cli(&["plugin", "list", "--json"], None)?;
    serde_json::from_str(&out).map_err(|e| format!("failed to parse plugin list: {e}"))
}

#[tauri::command]
pub fn get_blocked_plugins() -> Result<serde_json::Value, String> {
    let path = paths::claude_home().join("plugins").join("blocked_plugins.json");

    if !path.exists() {
        return Ok(serde_json::json!([]));
    }

    let content = fs::read_to_string(&path)
        .map_err(|e| format!("failed to read blocked plugins: {e}"))?;

    serde_json::from_str(&content)
        .map_err(|e| format!("failed to parse: {e}"))
}

#[tauri::command]
pub fn get_marketplace_plugins() -> Result<serde_json::Value, String> {
    let marketplaces_dir = paths::claude_home().join("plugins").join("marketplaces");

    if !marketplaces_dir.exists() {
        return Ok(serde_json::json!([]));
    }

    let mut all_plugins = Vec::new();

    let entries = fs::read_dir(&marketplaces_dir)
        .map_err(|e| format!("failed to read marketplaces dir: {e}"))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("failed to read entry: {e}"))?;
        let marketplace_path = entry.path().join(".claude-plugin").join("marketplace.json");

        if marketplace_path.exists() {
            let content = fs::read_to_string(&marketplace_path)
                .map_err(|e| format!("failed to read marketplace: {e}"))?;

            if let Ok(data) = serde_json::from_str::<serde_json::Value>(&content) {
                all_plugins.push(data);
            }
        }
    }

    Ok(serde_json::json!(all_plugins))
}

#[tauri::command]
pub fn get_install_counts() -> Result<serde_json::Value, String> {
    let path = paths::claude_home()
        .join("plugins")
        .join("install-counts-cache.json");

    if !path.exists() {
        return Ok(serde_json::json!([]));
    }

    let content = fs::read_to_string(&path)
        .map_err(|e| format!("failed to read install counts: {e}"))?;

    serde_json::from_str(&content)
        .map_err(|e| format!("failed to parse: {e}"))
}

#[tauri::command(async)]
pub fn install_plugin(
    name: String,
    scope: Option<String>,
    project_path: Option<String>,
) -> Result<String, String> {
    let args = scoped_args("install", &name, &scope)?;
    run_plugin_cli(&args, project_path.as_deref()).map_err(|e| format!("install failed: {e}"))
}

#[tauri::command(async)]
pub fn uninstall_plugin(
    name: String,
    scope: Option<String>,
    project_path: Option<String>,
) -> Result<String, String> {
    let args = scoped_args("uninstall", &name, &scope)?;
    run_plugin_cli(&args, project_path.as_deref()).map_err(|e| format!("uninstall failed: {e}"))
}

#[tauri::command(async)]
pub fn enable_plugin(
    name: String,
    scope: Option<String>,
    project_path: Option<String>,
) -> Result<String, String> {
    let args = scoped_args("enable", &name, &scope)?;
    run_plugin_cli(&args, project_path.as_deref()).map_err(|e| format!("enable failed: {e}"))
}

#[tauri::command(async)]
pub fn disable_plugin(
    name: String,
    scope: Option<String>,
    project_path: Option<String>,
) -> Result<String, String> {
    let args = scoped_args("disable", &name, &scope)?;
    run_plugin_cli(&args, project_path.as_deref()).map_err(|e| format!("disable failed: {e}"))
}

#[tauri::command(async)]
pub fn update_plugin(name: String) -> Result<String, String> {
    validate_name(&name)?;
    run_plugin_cli(&["plugin", "update", &name], None).map_err(|e| format!("update failed: {e}"))
}

/// Remove auto-installed dependencies no longer needed.
#[tauri::command(async)]
pub fn prune_plugins() -> Result<String, String> {
    run_plugin_cli(&["plugin", "prune", "-y"], None).map_err(|e| format!("prune failed: {e}"))
}

/// Component inventory + projected token cost (text output; the CLI has no
/// --json for this subcommand). Resolves enabled plugins by bare name.
#[tauri::command(async)]
pub fn plugin_details(name: String) -> Result<String, String> {
    validate_name(&name)?;
    run_plugin_cli(&["plugin", "details", &name], None)
}

#[tauri::command(async)]
pub fn marketplace_list() -> Result<serde_json::Value, String> {
    let out = run_plugin_cli(&["plugin", "marketplace", "list", "--json"], None)?;
    serde_json::from_str(&out).map_err(|e| format!("failed to parse marketplace list: {e}"))
}

#[tauri::command(async)]
pub fn marketplace_add(source: String) -> Result<String, String> {
    let source = source.trim();
    if source.is_empty() || source.starts_with('-') {
        return Err(format!("invalid marketplace source: {source:?}"));
    }
    run_plugin_cli(&["plugin", "marketplace", "add", source], None)
        .map_err(|e| format!("marketplace add failed: {e}"))
}

#[tauri::command(async)]
pub fn marketplace_remove(name: String) -> Result<String, String> {
    validate_name(&name)?;
    run_plugin_cli(&["plugin", "marketplace", "remove", &name], None)
        .map_err(|e| format!("marketplace remove failed: {e}"))
}

#[tauri::command(async)]
pub fn marketplace_update(name: Option<String>) -> Result<String, String> {
    let mut args = vec!["plugin", "marketplace", "update"];
    if let Some(ref n) = name {
        validate_name(n)?;
        args.push(n.as_str());
    }
    run_plugin_cli(&args, None).map_err(|e| format!("marketplace update failed: {e}"))
}
