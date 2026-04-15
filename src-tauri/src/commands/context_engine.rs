//! Tauri commands for the Context Engine (glyphic-ctx sidecar).
//!
//! Mirrors the shape of `token_savings.rs`: install the binary into
//! `~/.glyphic/bin`, write a wrapper script into `~/.claude/hooks/`, register
//! PostToolUse / UserPromptSubmit / PreToolUse entries in settings.json.

use serde::Serialize;
use std::path::{Path, PathBuf};

use crate::ctx::{config, db::{Db, EmbedKind}, embed};
use crate::paths;

const HOOK_SCRIPT_NAME: &str = "glyphic-ctx.sh";
const HOOK_EVENTS: &[&str] = &["PreToolUse", "PostToolUse", "UserPromptSubmit"];

#[derive(Serialize)]
pub struct ContextEngineStatus {
    pub enabled: bool,
    #[serde(rename = "sidecarInstalled")]
    pub sidecar_installed: bool,
    #[serde(rename = "sidecarVersion")]
    pub sidecar_version: Option<String>,
    #[serde(rename = "hookInstalled")]
    pub hook_installed: bool,
    #[serde(rename = "dbPath")]
    pub db_path: String,
    #[serde(rename = "toolResults")]
    pub tool_results: i64,
    pub turns: i64,
    #[serde(rename = "bytesStored")]
    pub bytes_stored: i64,
    #[serde(rename = "embeddedToolResults")]
    pub embedded_tool_results: i64,
    #[serde(rename = "embeddedTurns")]
    pub embedded_turns: i64,
    #[serde(rename = "embeddingReady")]
    pub embedding_ready: bool,
}

#[tauri::command]
pub fn ctx_get_status() -> Result<ContextEngineStatus, String> {
    let bin = config::bin_path();
    let sidecar_installed = bin.exists();

    let mut sidecar_version = if sidecar_installed {
        std::process::Command::new(&bin)
            .arg("version")
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
    } else {
        None
    };

    // Auto-upgrade if version mismatch
    let app_version = format!("glyphic-ctx {}", env!("CARGO_PKG_VERSION"));
    if sidecar_installed && sidecar_version.as_deref() != Some(&app_version) {
        if let Ok(source) = find_sidecar_source() {
            if std::fs::copy(&source, &bin).is_ok() {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let _ = std::fs::set_permissions(
                        &bin,
                        std::fs::Permissions::from_mode(0o755),
                    );
                }
                sidecar_version = Some(app_version);
            }
        }
    }

    let hook_installed = check_hook_installed();

    let (tool_results, turns, bytes_stored, embedded_tool_results, embedded_turns) =
        match Db::open() {
            Ok(db) => {
                let stats = db.stats().ok();
                let emb = db.embedded_counts().ok();
                (
                    stats.as_ref().map(|s| s.tool_results).unwrap_or(0),
                    stats.as_ref().map(|s| s.turns).unwrap_or(0),
                    stats.as_ref().map(|s| s.bytes_stored).unwrap_or(0),
                    emb.as_ref().map(|e| e.tool_results_embedded).unwrap_or(0),
                    emb.as_ref().map(|e| e.turns_embedded).unwrap_or(0),
                )
            }
            Err(_) => (0, 0, 0, 0, 0),
        };

    Ok(ContextEngineStatus {
        enabled: sidecar_installed && hook_installed,
        sidecar_installed,
        sidecar_version,
        hook_installed,
        db_path: config::db_path().to_string_lossy().to_string(),
        tool_results,
        turns,
        bytes_stored,
        embedded_tool_results,
        embedded_turns,
        embedding_ready: embed::is_ready(),
    })
}

#[derive(Serialize)]
pub struct ReindexReport {
    pub processed: u64,
    pub remaining: i64,
}

/// Embed rows currently missing an embedding. Caller loops until
/// `remaining == 0`; batching this way keeps individual invocations short
/// so the UI can show progress and the user can cancel at any time.
#[tauri::command]
pub fn ctx_reindex_embeddings(batch: Option<usize>) -> Result<ReindexReport, String> {
    let batch = batch.unwrap_or(64).clamp(1, 512);
    let db = Db::open().map_err(|e| format!("db open: {e}"))?;
    let rows = db
        .rows_needing_embedding(batch)
        .map_err(|e| format!("db scan: {e}"))?;

    let mut processed: u64 = 0;
    if !rows.is_empty() {
        let texts: Vec<&str> = rows.iter().map(|r| r.text.as_str()).collect();
        let embs = embed::embed_batch(&texts).ok_or_else(|| {
            "embedding model not available — download may still be in progress".to_string()
        })?;
        for (row, vec) in rows.iter().zip(embs.iter()) {
            let res = match row.kind {
                EmbedKind::ToolResult => db.update_tool_result_embedding(&row.id, vec),
                EmbedKind::Turn => db.update_turn_embedding(&row.id, vec),
            };
            if res.is_ok() {
                processed += 1;
            }
        }
    }

    // Fresh counts so the UI can drive its loop without double-calling.
    let remaining = match db.embedded_counts() {
        Ok(c) => (c.tool_results_total - c.tool_results_embedded)
            + (c.turns_total - c.turns_embedded),
        Err(_) => 0,
    };
    Ok(ReindexReport {
        processed,
        remaining,
    })
}

#[derive(Serialize)]
pub struct PurgeReport {
    pub deleted: i64,
}

/// One-shot cleanup of tool-result rows that the current hook would not have
/// stored: entries for tools now on `SKIP_TOOLS`, and pre-extractor rows
/// whose `content` is still the raw `{"tool_name": …, "tool_response": …}`
/// JSON envelope. Idempotent; running twice just returns 0 the second time.
#[tauri::command]
pub fn ctx_purge_legacy() -> Result<PurgeReport, String> {
    let db = Db::open().map_err(|e| format!("db open: {e}"))?;
    let deleted = db
        .purge_legacy_tool_results(config::SKIP_TOOLS)
        .map_err(|e| format!("db purge: {e}"))?;
    Ok(PurgeReport { deleted })
}

#[tauri::command]
pub fn ctx_enable() -> Result<(), String> {
    // 1. Install binary
    let data_dir = config::data_dir();
    let bin_dir = data_dir.join("bin");
    std::fs::create_dir_all(&bin_dir)
        .map_err(|e| format!("failed to create ~/.glyphic/bin: {e}"))?;

    let target_bin = config::bin_path();
    let source_bin = find_sidecar_source()?;
    std::fs::copy(&source_bin, &target_bin).map_err(|e| {
        format!(
            "failed to copy sidecar from {} to {}: {e}",
            source_bin.display(),
            target_bin.display()
        )
    })?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&target_bin, std::fs::Permissions::from_mode(0o755))
            .map_err(|e| format!("failed to set permissions: {e}"))?;
    }

    // 2. Write hook wrapper script
    let hooks_dir = paths::claude_home().join("hooks");
    std::fs::create_dir_all(&hooks_dir)
        .map_err(|e| format!("failed to create hooks dir: {e}"))?;
    let hook_path = hooks_dir.join(HOOK_SCRIPT_NAME);
    let hook_content = format!(
        "#!/bin/bash\n\"{bin}\" hook\n",
        bin = target_bin.to_string_lossy()
    );
    std::fs::write(&hook_path, hook_content)
        .map_err(|e| format!("failed to write hook script: {e}"))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&hook_path, std::fs::Permissions::from_mode(0o755))
            .map_err(|e| format!("failed to set hook permissions: {e}"))?;
    }

    // 3. Register in settings.json
    update_settings(&hook_path, true)?;
    Ok(())
}

#[tauri::command]
pub fn ctx_disable() -> Result<(), String> {
    let hooks_dir = paths::claude_home().join("hooks");
    let hook_path = hooks_dir.join(HOOK_SCRIPT_NAME);
    update_settings(&hook_path, false)?;
    Ok(())
}

#[derive(Serialize)]
pub struct RecentToolResult {
    pub id: String,
    pub tool: String,
    pub ts: u64,
    #[serde(rename = "sizeBytes")]
    pub size_bytes: i64,
    #[serde(rename = "lineCount")]
    pub line_count: i64,
    pub project: String,
    pub summary: String,
}

#[tauri::command]
pub fn ctx_recent_tool_results(project: Option<String>, limit: Option<usize>) -> Result<Vec<RecentToolResult>, String> {
    let db = Db::open().map_err(|e| format!("db open: {e}"))?;
    let proj = project.as_deref();
    let rows = db
        .recent_tool_results(proj, limit.unwrap_or(50))
        .map_err(|e| format!("db read: {e}"))?;
    Ok(rows
        .into_iter()
        .map(|r| RecentToolResult {
            id: r.id,
            tool: r.tool,
            ts: r.ts,
            size_bytes: r.size_bytes,
            line_count: r.line_count,
            project: r.project,
            summary: truncate_chars(&r.summary, 400),
        })
        .collect())
}

fn truncate_chars(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_string()
    } else {
        s.chars().take(n).collect::<String>() + "…"
    }
}

// ── Internals ──────────────────────────────────────────────────────────────

fn check_hook_installed() -> bool {
    let settings_path = paths::global_settings_path();
    let content = match std::fs::read_to_string(&settings_path) {
        Ok(c) => c,
        Err(_) => return false,
    };
    let settings: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return false,
    };
    let hooks = match settings.get("hooks") {
        Some(h) => h,
        None => return false,
    };
    HOOK_EVENTS.iter().all(|ev| event_has_glyphic_ctx(hooks, ev))
}

fn event_has_glyphic_ctx(hooks: &serde_json::Value, event: &str) -> bool {
    hooks
        .get(event)
        .and_then(|arr| arr.as_array())
        .map(|configs| {
            configs.iter().any(|config| {
                config
                    .get("hooks")
                    .and_then(|h| h.as_array())
                    .map(|list| {
                        list.iter().any(|h| {
                            h.get("command")
                                .and_then(|c| c.as_str())
                                .map(|c| c.contains("glyphic-ctx"))
                                .unwrap_or(false)
                        })
                    })
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false)
}

/// Add or remove our hook entry across all relevant events.
fn update_settings(hook_path: &Path, enable: bool) -> Result<(), String> {
    let settings_path = paths::global_settings_path();
    let mut settings: serde_json::Value = if settings_path.exists() {
        let content = std::fs::read_to_string(&settings_path)
            .map_err(|e| format!("failed to read settings: {e}"))?;
        serde_json::from_str(&content).map_err(|e| format!("failed to parse settings: {e}"))?
    } else {
        serde_json::json!({})
    };

    let hooks = settings
        .as_object_mut()
        .ok_or("settings is not an object")?
        .entry("hooks")
        .or_insert_with(|| serde_json::json!({}));

    for event in HOOK_EVENTS {
        let arr = hooks
            .as_object_mut()
            .ok_or("hooks is not an object")?
            .entry(event.to_string())
            .or_insert_with(|| serde_json::json!([]))
            .as_array_mut()
            .ok_or(format!("{event} is not an array"))?;

        // Always strip any existing glyphic-ctx entries first (idempotent)
        arr.retain(|entry| {
            !entry
                .get("hooks")
                .and_then(|h| h.as_array())
                .map(|list| {
                    list.iter().any(|h| {
                        h.get("command")
                            .and_then(|c| c.as_str())
                            .map(|c| c.contains("glyphic-ctx"))
                            .unwrap_or(false)
                    })
                })
                .unwrap_or(false)
        });

        if enable {
            arr.push(serde_json::json!({
                "hooks": [{
                    "type": "command",
                    "command": format!("bash \"{}\"", hook_path.to_string_lossy())
                }]
            }));
        }
    }

    if let Some(parent) = settings_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("failed to create settings dir: {e}"))?;
    }
    let content = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("failed to serialize settings: {e}"))?;
    std::fs::write(&settings_path, content)
        .map_err(|e| format!("failed to write settings: {e}"))?;
    Ok(())
}

fn find_sidecar_source() -> Result<PathBuf, String> {
    let candidates: Vec<Option<PathBuf>> = vec![
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.join("glyphic-ctx"))),
        std::env::current_exe().ok().and_then(|p| {
            p.parent()
                .and_then(|d| d.parent())
                .map(|d| d.join("release").join("glyphic-ctx"))
        }),
        std::env::current_exe().ok().and_then(|p| {
            p.parent()
                .and_then(|d| d.parent())
                .map(|d| d.join("debug").join("glyphic-ctx"))
        }),
    ];
    for c in candidates.into_iter().flatten() {
        if c.exists() {
            return Ok(c);
        }
    }
    Err("glyphic-ctx binary not found; build the project first".into())
}
