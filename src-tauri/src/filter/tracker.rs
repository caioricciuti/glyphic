use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::filter::estimate_tokens;

fn default_tool_type() -> String {
    "Bash".to_string()
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SavingsRecord {
    /// Unix timestamp in seconds
    pub ts: u64,
    /// Original command string
    pub cmd: String,
    /// Raw output byte count
    pub input_bytes: usize,
    /// Filtered output byte count
    pub output_bytes: usize,
    /// Estimated input tokens
    pub input_tokens: u64,
    /// Estimated output tokens
    pub output_tokens: u64,
    /// Savings percentage (0-100)
    pub savings_pct: f64,
    /// Execution time in milliseconds
    pub time_ms: u64,
    /// Project directory (if determinable from cwd)
    pub project: String,
    /// Tool type: "Bash", "Read", "Grep"
    #[serde(default = "default_tool_type")]
    pub tool_type: String,
}

pub struct SavingsTracker;

impl SavingsTracker {
    /// Directory for all glyphic optimizer data.
    pub fn data_dir() -> PathBuf {
        dirs::home_dir()
            .expect("could not resolve home directory")
            .join(".glyphic")
    }

    /// Path to the savings JSONL log file.
    pub fn log_path() -> PathBuf {
        Self::data_dir().join("savings.jsonl")
    }

    /// Path to user filter overrides directory.
    pub fn filters_dir() -> PathBuf {
        Self::data_dir().join("filters")
    }

    /// Path to the installed sidecar binary.
    pub fn bin_path() -> PathBuf {
        Self::data_dir().join("bin").join("glyphic-filter")
    }

    /// Path to the optimizer behavior settings file.
    pub fn settings_path() -> PathBuf {
        Self::data_dir().join("filter-settings.json")
    }

    /// Whether the hook may auto-approve rewritten Bash commands. Defaults to
    /// true (the historical behavior). When false, the hook still rewrites the
    /// command for output filtering but leaves the permission decision to
    /// Claude Code's normal flow.
    pub fn auto_approve() -> bool {
        std::fs::read_to_string(Self::settings_path())
            .ok()
            .and_then(|c| serde_json::from_str::<serde_json::Value>(&c).ok())
            .and_then(|v| v.get("autoApprove").and_then(|b| b.as_bool()))
            .unwrap_or(true)
    }

    /// Create the data dir and keep it private (0700): savings logs and the
    /// context database can contain command output with secrets in it.
    pub fn ensure_data_dir() -> Result<PathBuf, String> {
        let dir = Self::data_dir();
        fs::create_dir_all(&dir).map_err(|e| format!("failed to create data dir: {e}"))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&dir, fs::Permissions::from_mode(0o700));
        }
        Ok(dir)
    }

    /// Record a savings event with the actual texts, so token estimates are
    /// content-aware instead of bytes/4.
    pub fn record_with_texts(
        cmd: &str,
        input: &str,
        output: &str,
        time_ms: u64,
        project: &str,
        tool_type: &str,
    ) -> Result<(), String> {
        Self::record_inner(
            cmd,
            input.len(),
            output.len(),
            crate::filter::estimate_tokens_text(input),
            crate::filter::estimate_tokens_text(output),
            time_ms,
            project,
            tool_type,
        )
    }

    /// Record a savings event from byte counts alone (texts unavailable).
    pub fn record(
        cmd: &str,
        input_bytes: usize,
        output_bytes: usize,
        time_ms: u64,
        project: &str,
        tool_type: &str,
    ) -> Result<(), String> {
        Self::record_inner(
            cmd,
            input_bytes,
            output_bytes,
            estimate_tokens(input_bytes),
            estimate_tokens(output_bytes),
            time_ms,
            project,
            tool_type,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn record_inner(
        cmd: &str,
        input_bytes: usize,
        output_bytes: usize,
        input_tokens: u64,
        output_tokens: u64,
        time_ms: u64,
        project: &str,
        tool_type: &str,
    ) -> Result<(), String> {
        Self::ensure_data_dir()?;
        // saturating: a filter can legitimately produce output longer than
        // the input (markers, headers), which must not underflow
        let savings_pct = if input_bytes > 0 {
            (input_bytes.saturating_sub(output_bytes)) as f64 / input_bytes as f64 * 100.0
        } else {
            0.0
        };

        let record = SavingsRecord {
            ts: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            cmd: normalize_command(cmd),
            input_bytes,
            output_bytes,
            input_tokens,
            output_tokens,
            savings_pct,
            time_ms,
            project: project.to_string(),
            tool_type: tool_type.to_string(),
        };

        let line =
            serde_json::to_string(&record).map_err(|e| format!("failed to serialize: {e}"))?;

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(Self::log_path())
            .map_err(|e| format!("failed to open savings log: {e}"))?;

        writeln!(file, "{line}").map_err(|e| format!("failed to write savings log: {e}"))?;

        Ok(())
    }
}

/// Normalize command for aggregation: extract the base command (e.g. "git status" from
/// "git status --short -b").
fn normalize_command(cmd: &str) -> String {
    let trimmed = cmd.trim();
    let parts: Vec<&str> = trimmed.split_whitespace().collect();
    match parts.first().copied() {
        Some("git" | "cargo" | "npm" | "bun" | "docker" | "npx" | "kubectl" | "go" | "uv" | "pip" | "pip3") => {
            parts.iter().take(2).copied().collect::<Vec<_>>().join(" ")
        }
        Some(base) => base.to_string(),
        None => trimmed.to_string(),
    }
}
