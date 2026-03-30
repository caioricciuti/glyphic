use crate::filter::tracker::{SavingsRecord, SavingsTracker};
use crate::paths;
use serde::Serialize;
use std::collections::HashMap;
use std::io::BufRead;
use std::path::Path;

// ── Status ──────────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct OptimizerStatus {
    pub enabled: bool,
    #[serde(rename = "sidecarInstalled")]
    pub sidecar_installed: bool,
    #[serde(rename = "sidecarVersion")]
    pub sidecar_version: Option<String>,
    #[serde(rename = "hookInstalled")]
    pub hook_installed: bool,
    #[serde(rename = "savingsLogExists")]
    pub savings_log_exists: bool,
    #[serde(rename = "totalCommandsTracked")]
    pub total_commands_tracked: u64,
}

#[tauri::command]
pub fn get_optimizer_status() -> Result<OptimizerStatus, String> {
    let bin = SavingsTracker::bin_path();
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

    // Auto-upgrade sidecar if version doesn't match the app
    let app_version = format!("glyphic-filter {}", env!("CARGO_PKG_VERSION"));
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
    let log_path = SavingsTracker::log_path();
    let savings_log_exists = log_path.exists();

    let total_commands_tracked = if savings_log_exists {
        count_lines(&log_path).unwrap_or(0)
    } else {
        0
    };

    Ok(OptimizerStatus {
        enabled: sidecar_installed && hook_installed,
        sidecar_installed,
        sidecar_version,
        hook_installed,
        savings_log_exists,
        total_commands_tracked,
    })
}

fn check_hook_installed() -> bool {
    let settings_path = paths::global_settings_path();
    if !settings_path.exists() {
        return false;
    }

    let content = match std::fs::read_to_string(&settings_path) {
        Ok(c) => c,
        Err(_) => return false,
    };

    let settings: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return false,
    };

    settings
        .get("hooks")
        .and_then(|h| h.get("PreToolUse"))
        .and_then(|arr| arr.as_array())
        .map(|configs| {
            configs.iter().any(|config| {
                // Check hooks[].command inside each HookEventConfig
                config
                    .get("hooks")
                    .and_then(|h| h.as_array())
                    .map(|hooks| {
                        hooks.iter().any(|h| {
                            h.get("command")
                                .and_then(|c| c.as_str())
                                .map(|c| c.contains("glyphic-optimizer"))
                                .unwrap_or(false)
                        })
                    })
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false)
}

fn count_lines(path: &Path) -> Result<u64, String> {
    let file =
        std::fs::File::open(path).map_err(|e| format!("failed to open savings log: {e}"))?;
    let reader = std::io::BufReader::new(file);
    Ok(reader.lines().filter_map(|l| l.ok()).count() as u64)
}

// ── Enable / Disable ────────────────────────────────────────────────────────

#[tauri::command]
pub fn enable_optimizer() -> Result<(), String> {
    // 1. Ensure data directories exist
    let data_dir = SavingsTracker::data_dir();
    let bin_dir = data_dir.join("bin");
    std::fs::create_dir_all(&bin_dir)
        .map_err(|e| format!("failed to create ~/.glyphic/bin: {e}"))?;

    // 2. Find and copy the sidecar binary
    let target_bin = SavingsTracker::bin_path();
    let source_bin = find_sidecar_source()?;
    std::fs::copy(&source_bin, &target_bin).map_err(|e| {
        format!(
            "failed to copy sidecar from {} to {}: {e}",
            source_bin.display(),
            target_bin.display()
        )
    })?;

    // Make executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o755);
        std::fs::set_permissions(&target_bin, perms)
            .map_err(|e| format!("failed to set permissions: {e}"))?;
    }

    // 3. Write the hook script
    let hooks_dir = paths::claude_home().join("hooks");
    std::fs::create_dir_all(&hooks_dir)
        .map_err(|e| format!("failed to create hooks dir: {e}"))?;

    let hook_path = hooks_dir.join("glyphic-optimizer.sh");
    let hook_content = format!(
        "#!/bin/bash\n\"{bin}\" hook\n",
        bin = target_bin.to_string_lossy()
    );
    std::fs::write(&hook_path, hook_content)
        .map_err(|e| format!("failed to write hook script: {e}"))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o755);
        std::fs::set_permissions(&hook_path, perms)
            .map_err(|e| format!("failed to set hook permissions: {e}"))?;
    }

    // 4. Update settings.json to register the hook
    let settings_path = paths::global_settings_path();
    let mut settings = if settings_path.exists() {
        let content = std::fs::read_to_string(&settings_path)
            .map_err(|e| format!("failed to read settings: {e}"))?;
        serde_json::from_str::<serde_json::Value>(&content)
            .map_err(|e| format!("failed to parse settings: {e}"))?
    } else {
        serde_json::json!({})
    };

    let hook_entry = serde_json::json!({
        "hooks": [{
            "type": "command",
            "command": format!("bash \"{}\"", hook_path.to_string_lossy())
        }],
        "matcher": "Bash"
    });

    // Ensure hooks.PreToolUse exists and add our hook (avoid duplicates)
    let hooks = settings
        .as_object_mut()
        .ok_or("settings is not an object")?
        .entry("hooks")
        .or_insert_with(|| serde_json::json!({}));

    let pre_tool_use = hooks
        .as_object_mut()
        .ok_or("hooks is not an object")?
        .entry("PreToolUse")
        .or_insert_with(|| serde_json::json!([]));

    let arr = pre_tool_use
        .as_array_mut()
        .ok_or("PreToolUse is not an array")?;

    // Remove any existing glyphic-optimizer entries first
    arr.retain(|entry| {
        !entry
            .get("hooks")
            .and_then(|h| h.as_array())
            .map(|hooks| {
                hooks.iter().any(|h| {
                    h.get("command")
                        .and_then(|c| c.as_str())
                        .map(|c| c.contains("glyphic-optimizer"))
                        .unwrap_or(false)
                })
            })
            .unwrap_or(false)
    });

    arr.push(hook_entry);

    // Write settings back
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

/// Locate the glyphic-filter binary. Checks dev build path, then app bundle.
fn find_sidecar_source() -> Result<std::path::PathBuf, String> {
    // Dev mode: check cargo target directory
    let dev_paths = [
        // Debug build
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.join("glyphic-filter"))),
        // Release build
        std::env::current_exe()
            .ok()
            .and_then(|p| {
                p.parent()
                    .and_then(|d| d.parent())
                    .map(|d| d.join("release").join("glyphic-filter"))
            }),
        // Alongside the main binary
        std::env::current_exe()
            .ok()
            .map(|p| p.with_file_name("glyphic-filter")),
    ];

    for path in dev_paths.iter().flatten() {
        if path.exists() {
            return Ok(path.clone());
        }
    }

    Err(
        "Could not find glyphic-filter binary. Make sure it's built with: cargo build --bin glyphic-filter"
            .to_string(),
    )
}

#[tauri::command]
pub fn disable_optimizer() -> Result<(), String> {
    // Remove hook from settings.json (keep sidecar + data)
    let settings_path = paths::global_settings_path();
    if !settings_path.exists() {
        return Ok(());
    }

    let content = std::fs::read_to_string(&settings_path)
        .map_err(|e| format!("failed to read settings: {e}"))?;
    let mut settings: serde_json::Value =
        serde_json::from_str(&content).map_err(|e| format!("failed to parse settings: {e}"))?;

    if let Some(hooks) = settings.get_mut("hooks") {
        if let Some(pre_tool_use) = hooks.get_mut("PreToolUse") {
            if let Some(arr) = pre_tool_use.as_array_mut() {
                arr.retain(|entry| {
                    !entry
                        .get("hooks")
                        .and_then(|h| h.as_array())
                        .map(|hooks| {
                            hooks.iter().any(|h| {
                                h.get("command")
                                    .and_then(|c| c.as_str())
                                    .map(|c| c.contains("glyphic-optimizer"))
                                    .unwrap_or(false)
                            })
                        })
                        .unwrap_or(false)
                });

                // Clean up empty arrays/objects
                if arr.is_empty() {
                    hooks.as_object_mut().map(|h| h.remove("PreToolUse"));
                }
            }
        }
        if hooks.as_object().map(|h| h.is_empty()).unwrap_or(false) {
            settings.as_object_mut().map(|s| s.remove("hooks"));
        }
    }

    let content = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("failed to serialize settings: {e}"))?;
    std::fs::write(&settings_path, content)
        .map_err(|e| format!("failed to write settings: {e}"))?;

    Ok(())
}

// ── Savings Data ────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct SavingsSummary {
    #[serde(rename = "totalCommands")]
    pub total_commands: u64,
    #[serde(rename = "totalInputTokens")]
    pub total_input_tokens: u64,
    #[serde(rename = "totalOutputTokens")]
    pub total_output_tokens: u64,
    #[serde(rename = "totalSaved")]
    pub total_saved: u64,
    #[serde(rename = "avgSavingsPct")]
    pub avg_savings_pct: f64,
}

#[derive(Serialize)]
pub struct SavingsTimeBucket {
    pub label: String,
    pub commands: u64,
    #[serde(rename = "inputTokens")]
    pub input_tokens: u64,
    #[serde(rename = "outputTokens")]
    pub output_tokens: u64,
    #[serde(rename = "savedTokens")]
    pub saved_tokens: u64,
    #[serde(rename = "savingsPct")]
    pub savings_pct: f64,
}

#[derive(Serialize)]
pub struct CommandSavings {
    pub command: String,
    pub count: u64,
    #[serde(rename = "totalSaved")]
    pub total_saved: u64,
    #[serde(rename = "avgSavingsPct")]
    pub avg_savings_pct: f64,
}

#[derive(Serialize)]
pub struct SavingsData {
    pub summary: SavingsSummary,
    pub daily: Vec<SavingsTimeBucket>,
    #[serde(rename = "topCommands")]
    pub top_commands: Vec<CommandSavings>,
}

#[tauri::command]
pub fn get_savings_data(
    period: String,
    project_path: Option<String>,
) -> Result<SavingsData, String> {
    let log_path = SavingsTracker::log_path();

    if !log_path.exists() {
        return Ok(SavingsData {
            summary: SavingsSummary {
                total_commands: 0,
                total_input_tokens: 0,
                total_output_tokens: 0,
                total_saved: 0,
                avg_savings_pct: 0.0,
            },
            daily: vec![],
            top_commands: vec![],
        });
    }

    let records = read_savings_log(&log_path, project_path.as_deref())?;

    // Aggregate summary
    let total_commands = records.len() as u64;
    let total_input: u64 = records.iter().map(|r| r.input_tokens).sum();
    let total_output: u64 = records.iter().map(|r| r.output_tokens).sum();
    let total_saved = total_input.saturating_sub(total_output);
    let avg_savings_pct = if total_input > 0 {
        (total_saved as f64 / total_input as f64) * 100.0
    } else {
        0.0
    };

    // Aggregate by time period
    let daily = aggregate_by_period(&records, &period);

    // Aggregate by command
    let mut cmd_map: HashMap<String, (u64, u64, f64)> = HashMap::new();
    for r in &records {
        let entry = cmd_map.entry(r.cmd.clone()).or_insert((0, 0, 0.0));
        entry.0 += 1;
        entry.1 += r.input_tokens.saturating_sub(r.output_tokens);
        entry.2 += r.savings_pct;
    }

    let mut top_commands: Vec<CommandSavings> = cmd_map
        .into_iter()
        .map(|(cmd, (count, saved, pct_sum))| CommandSavings {
            command: cmd,
            count,
            total_saved: saved,
            avg_savings_pct: if count > 0 {
                pct_sum / count as f64
            } else {
                0.0
            },
        })
        .collect();
    top_commands.sort_by(|a, b| b.total_saved.cmp(&a.total_saved));
    top_commands.truncate(20);

    Ok(SavingsData {
        summary: SavingsSummary {
            total_commands,
            total_input_tokens: total_input,
            total_output_tokens: total_output,
            total_saved,
            avg_savings_pct,
        },
        daily,
        top_commands,
    })
}

fn read_savings_log(
    path: &Path,
    project_filter: Option<&str>,
) -> Result<Vec<SavingsRecord>, String> {
    let file =
        std::fs::File::open(path).map_err(|e| format!("failed to open savings log: {e}"))?;
    let reader = std::io::BufReader::new(file);

    let mut records = Vec::new();
    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };
        if line.trim().is_empty() {
            continue;
        }
        let record: SavingsRecord = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(_) => continue,
        };
        if let Some(filter) = project_filter {
            if !record.project.starts_with(filter) {
                continue;
            }
        }
        records.push(record);
    }
    Ok(records)
}

fn aggregate_by_period(records: &[SavingsRecord], period: &str) -> Vec<SavingsTimeBucket> {
    let mut buckets: HashMap<String, (u64, u64, u64)> = HashMap::new();

    for r in records {
        let label = match period {
            "weekly" => {
                let date = chrono_lite_date(r.ts as i64);
                // Approximate week: use the Monday of that week
                format!("W{}", &date[..7])
            }
            "monthly" => {
                let date = chrono_lite_date(r.ts as i64);
                date[..7].to_string()
            }
            _ => chrono_lite_date(r.ts as i64), // daily
        };

        let entry = buckets.entry(label).or_insert((0, 0, 0));
        entry.0 += 1;
        entry.1 += r.input_tokens;
        entry.2 += r.output_tokens;
    }

    let mut result: Vec<SavingsTimeBucket> = buckets
        .into_iter()
        .map(|(label, (commands, input, output))| {
            let saved = input.saturating_sub(output);
            let pct = if input > 0 {
                (saved as f64 / input as f64) * 100.0
            } else {
                0.0
            };
            SavingsTimeBucket {
                label,
                commands,
                input_tokens: input,
                output_tokens: output,
                saved_tokens: saved,
                savings_pct: pct,
            }
        })
        .collect();

    result.sort_by(|a, b| a.label.cmp(&b.label));
    result
}

/// Simple date extraction from unix timestamp (reused from stats.rs pattern).
fn chrono_lite_date(secs: i64) -> String {
    let days = secs / 86400;
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02}")
}

// ── Discover ────────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct DiscoverOpportunity {
    pub command: String,
    pub count: u64,
    pub category: String,
    #[serde(rename = "estimatedSavingsTokens")]
    pub estimated_savings_tokens: u64,
    #[serde(rename = "hasFilter")]
    pub has_filter: bool,
}

#[derive(Serialize)]
pub struct DiscoverResult {
    #[serde(rename = "sessionsScanned")]
    pub sessions_scanned: u64,
    #[serde(rename = "totalCommands")]
    pub total_commands: u64,
    pub opportunities: Vec<DiscoverOpportunity>,
    #[serde(rename = "totalPotentialSavings")]
    pub total_potential_savings: u64,
}

#[tauri::command]
pub fn discover_opportunities(project_path: Option<String>) -> Result<DiscoverResult, String> {
    let projects_dir = paths::projects_dir();
    if !projects_dir.exists() {
        return Ok(DiscoverResult {
            sessions_scanned: 0,
            total_commands: 0,
            opportunities: vec![],
            total_potential_savings: 0,
        });
    }

    let mut sessions_scanned: u64 = 0;
    let mut command_counts: HashMap<String, u64> = HashMap::new();
    let mut command_output_sizes: HashMap<String, Vec<u64>> = HashMap::new();

    // Scan all project session files
    let project_dirs: Vec<_> = std::fs::read_dir(&projects_dir)
        .map_err(|e| format!("failed to read projects dir: {e}"))?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .collect();

    for project_dir in &project_dirs {
        // Check project filter
        if let Some(ref filter_path) = project_path {
            let dir_name = project_dir.file_name().to_string_lossy().to_string();
            let resolved = paths::project_hash_to_path(&dir_name);
            if !resolved.starts_with(filter_path) {
                continue;
            }
        }

        // Scan JSONL session files — collect and sort by modified time, cap at newest
        let mut entries: Vec<_> = std::fs::read_dir(project_dir.path())
            .into_iter()
            .flatten()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map(|ext| ext == "jsonl")
                    .unwrap_or(false)
            })
            .collect();

        // Sort by modified time descending (newest first)
        entries.sort_by(|a, b| {
            let ma = a.metadata().and_then(|m| m.modified()).ok();
            let mb = b.metadata().and_then(|m| m.modified()).ok();
            mb.cmp(&ma)
        });

        // Cap to 100 most recent files total
        const MAX_FILES: u64 = 100;
        for entry in entries {
            if sessions_scanned >= MAX_FILES {
                break;
            }
            sessions_scanned += 1;
            scan_session_for_commands(
                &entry.path(),
                &mut command_counts,
                &mut command_output_sizes,
            );
        }
    }

    // Build opportunities from command counts
    let total_commands: u64 = command_counts.values().sum();
    let mut opportunities: Vec<DiscoverOpportunity> = Vec::new();

    for (cmd, count) in &command_counts {
        let has_filter = crate::filter::find_filter(cmd).is_some();
        let category = categorize_command(cmd);

        // Estimate savings based on average output size and typical compression
        let avg_output = command_output_sizes
            .get(cmd)
            .map(|sizes| {
                if sizes.is_empty() {
                    500
                } else {
                    sizes.iter().sum::<u64>() / sizes.len() as u64
                }
            })
            .unwrap_or(500);

        // Assume ~70% savings for filterable commands, ~20% for ANSI-only stripping
        let savings_rate = if has_filter { 0.70 } else { 0.20 };
        let estimated_savings = (avg_output as f64 * savings_rate / 4.0).ceil() as u64 * count;

        opportunities.push(DiscoverOpportunity {
            command: cmd.clone(),
            count: *count,
            category,
            estimated_savings_tokens: estimated_savings,
            has_filter,
        });
    }

    opportunities.sort_by(|a, b| b.estimated_savings_tokens.cmp(&a.estimated_savings_tokens));

    let total_potential_savings: u64 = opportunities.iter().map(|o| o.estimated_savings_tokens).sum();

    Ok(DiscoverResult {
        sessions_scanned,
        total_commands,
        opportunities,
        total_potential_savings,
    })
}

fn scan_session_for_commands(
    path: &Path,
    counts: &mut HashMap<String, u64>,
    output_sizes: &mut HashMap<String, Vec<u64>>,
) {
    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return,
    };
    let reader = std::io::BufReader::new(file);
    let mut lines_read = 0u32;
    const MAX_LINES: u32 = 2000;

    for line in reader.lines() {
        if lines_read >= MAX_LINES {
            break;
        }
        lines_read += 1;

        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };
        if line.trim().is_empty() {
            continue;
        }

        let entry: serde_json::Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let entry_type = entry.get("type").and_then(|t| t.as_str()).unwrap_or("");
        if entry_type != "assistant" {
            continue;
        }

        // Look for tool_use blocks in message content
        let content = match entry.get("message").and_then(|m| m.get("content")) {
            Some(c) => c,
            None => continue,
        };

        let blocks = match content.as_array() {
            Some(arr) => arr,
            None => continue,
        };

        for block in blocks {
            let block_type = block.get("type").and_then(|t| t.as_str()).unwrap_or("");
            if block_type != "tool_use" {
                continue;
            }

            let tool_name = block.get("name").and_then(|n| n.as_str()).unwrap_or("");
            if tool_name != "Bash" {
                continue;
            }

            let command = block
                .get("input")
                .and_then(|i| i.get("command"))
                .and_then(|c| c.as_str())
                .unwrap_or("");

            if command.is_empty() {
                continue;
            }

            let normalized = normalize_for_discover(command);
            *counts.entry(normalized.clone()).or_insert(0) += 1;

            // Estimate output size from the result if available (heuristic)
            let output_len = block
                .get("result")
                .and_then(|r| r.as_str())
                .map(|s| s.len() as u64)
                .unwrap_or(0);

            if output_len > 0 {
                output_sizes
                    .entry(normalized)
                    .or_default()
                    .push(output_len);
            }
        }
    }
}

fn normalize_for_discover(cmd: &str) -> String {
    let trimmed = cmd.trim();

    // Skip commands that look like they contain secrets
    if looks_like_secret(trimmed) {
        // Return just the base command without arguments
        return trimmed
            .split_whitespace()
            .next()
            .unwrap_or(trimmed)
            .to_string();
    }

    let parts: Vec<&str> = trimmed.split_whitespace().collect();
    match parts.first().copied() {
        Some("git" | "cargo" | "npm" | "bun" | "docker" | "npx") => {
            parts.iter().take(2).copied().collect::<Vec<_>>().join(" ")
        }
        Some(base) => base.to_string(),
        None => trimmed.to_string(),
    }
}

/// Check if a command string contains sensitive-looking values.
fn looks_like_secret(cmd: &str) -> bool {
    let upper = cmd.to_uppercase();
    // Environment variable assignments with sensitive names
    if upper.contains("TOKEN=")
        || upper.contains("SECRET=")
        || upper.contains("PASSWORD=")
        || upper.contains("API_KEY=")
        || upper.contains("AUTH=")
        || upper.contains("CREDENTIAL")
    {
        return true;
    }
    // Long base64-looking strings (likely tokens)
    cmd.split_whitespace().any(|part| {
        part.len() > 40 && part.chars().all(|c| c.is_alphanumeric() || c == '.' || c == '-' || c == '_' || c == '=')
    })
}

fn categorize_command(cmd: &str) -> String {
    let first = cmd.split_whitespace().next().unwrap_or("");
    match first {
        "git" => "Version Control".to_string(),
        "cargo" => "Rust".to_string(),
        "npm" | "bun" | "npx" | "vitest" | "jest" => "JavaScript".to_string(),
        "ls" | "tree" | "find" | "cat" | "head" | "tail" => "Files".to_string(),
        "grep" | "rg" | "ripgrep" => "Search".to_string(),
        "docker" | "kubectl" => "Containers".to_string(),
        "curl" | "wget" => "Network".to_string(),
        "python" | "pip" | "pytest" | "ruff" => "Python".to_string(),
        "go" => "Go".to_string(),
        _ => "Other".to_string(),
    }
}

// ── Filters ─────────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct FilterRules {
    pub path: String,
    #[serde(rename = "rawContent")]
    pub raw_content: String,
    #[serde(rename = "filterCount")]
    pub filter_count: u32,
    #[serde(rename = "builtinCount")]
    pub builtin_count: u32,
}

#[tauri::command]
pub fn get_filter_rules() -> Result<FilterRules, String> {
    let filters_dir = SavingsTracker::filters_dir();
    let custom_path = filters_dir.join("custom.toml");

    let raw_content = if custom_path.exists() {
        std::fs::read_to_string(&custom_path)
            .map_err(|e| format!("failed to read filters: {e}"))?
    } else {
        default_custom_filters_template()
    };

    // Count [[filter]] entries in user file
    let filter_count = raw_content.matches("[[filter]]").count() as u32;
    let builtin_count = crate::filter::builtin::FILTERS.len() as u32;

    Ok(FilterRules {
        path: custom_path.to_string_lossy().to_string(),
        raw_content,
        filter_count,
        builtin_count,
    })
}

#[tauri::command]
pub fn save_filter_rules(content: String) -> Result<(), String> {
    // Validate TOML syntax
    content
        .parse::<toml::Table>()
        .map_err(|e| format!("Invalid TOML syntax: {e}"))?;

    let filters_dir = SavingsTracker::filters_dir();
    std::fs::create_dir_all(&filters_dir)
        .map_err(|e| format!("failed to create filters dir: {e}"))?;

    let custom_path = filters_dir.join("custom.toml");
    std::fs::write(&custom_path, content)
        .map_err(|e| format!("failed to write filters: {e}"))?;

    Ok(())
}

fn default_custom_filters_template() -> String {
    r#"# Custom Glyphic Token Optimizer Filters
# Add your own filter rules here. These are applied alongside the built-in filters.
#
# Example:
# [[filter]]
# name = "my_custom_filter"
# match_command = "^my-tool\\s+"
# strip_ansi = true
# head_lines = 20
# on_empty = "(no output)"
"#
    .to_string()
}
