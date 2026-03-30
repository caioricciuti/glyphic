//! glyphic-filter — sidecar binary for token-optimized command filtering.
//!
//! Modes:
//!   hook    — Claude Code PreToolUse hook handler (JSON stdin → JSON stdout)
//!   exec    — Execute a command, filter output, log savings, print result
//!   version — Print version

use glyphic_lib::filter;
use std::io::{self, Read, Write};
use std::process::Command;
use std::time::Instant;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    match args.get(1).map(|s| s.as_str()) {
        Some("hook") => handle_hook(),
        Some("exec") => handle_exec(&args[2..]),
        Some("version") => {
            println!("glyphic-filter {}", env!("CARGO_PKG_VERSION"));
        }
        _ => {
            eprintln!("Usage: glyphic-filter <hook|exec|version>");
            std::process::exit(1);
        }
    }
}

// ── Hook mode ───────────────────────────────────────────────────────────────

/// Print a bare "allow" response (no input modification).
fn print_allow() {
    let response = serde_json::json!({
        "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "permissionDecision": "allow"
        }
    });
    println!("{}", serde_json::to_string(&response).unwrap());
}

/// Read Claude Code hook JSON from stdin, rewrite command if a filter matches.
fn handle_hook() {
    let mut input = String::new();
    if io::stdin().read_to_string(&mut input).is_err() {
        // Can't read stdin — allow unchanged
        print_allow();
        return;
    }

    let json: serde_json::Value = match serde_json::from_str(&input) {
        Ok(v) => v,
        Err(_) => {
            print_allow();
            return;
        }
    };

    let tool_name = json
        .get("tool_name")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let command = json
        .get("tool_input")
        .and_then(|ti| ti.get("command"))
        .and_then(|c| c.as_str())
        .unwrap_or("");

    // Only intercept Bash tool with non-empty commands
    if tool_name != "Bash" || command.is_empty() {
        print_allow();
        return;
    }

    // Never rewrite excluded commands
    if should_exclude(command) {
        print_allow();
        return;
    }

    // Check if any filter matches
    if filter::find_filter(command).is_some() {
        let bin = filter::tracker::SavingsTracker::bin_path();
        let bin_str = bin.to_string_lossy();
        // Escape the command for shell embedding
        let escaped = command.replace('\\', r"\\").replace('"', r#"\""#);
        let rewritten = format!(r#""{bin_str}" exec "{escaped}""#);

        let response = serde_json::json!({
            "hookSpecificOutput": {
                "hookEventName": "PreToolUse",
                "permissionDecision": "allow",
                "permissionDecisionReason": "glyphic-filter: wrapping command for token optimization",
                "updatedInput": {
                    "command": rewritten
                }
            }
        });
        println!("{}", serde_json::to_string(&response).unwrap());
    } else {
        // No filter — allow unchanged
        print_allow();
    }
}

/// Commands that should never be rewritten.
fn should_exclude(command: &str) -> bool {
    let trimmed = command.trim();

    // Already using glyphic-filter
    if trimmed.contains("glyphic-filter") {
        return true;
    }

    // Heredocs — complex shell constructs
    if trimmed.contains("<<") {
        return true;
    }

    // Interactive commands
    let interactive = [
        "vim", "nvim", "nano", "vi", "emacs", "less", "more", "ssh", "top", "htop", "man",
    ];
    let first_word = trimmed.split_whitespace().next().unwrap_or("");
    if interactive.contains(&first_word) {
        return true;
    }

    // Chained commands with pipes or semicolons that are complex
    // Allow simple pipes but exclude complex chains with multiple semicolons
    if trimmed.matches(';').count() > 2 {
        return true;
    }

    false
}

// ── Exec mode ───────────────────────────────────────────────────────────────

/// Execute a command, filter its output, log savings, print filtered result.
fn handle_exec(args: &[String]) {
    let command = args.join(" ");
    if command.is_empty() {
        eprintln!("glyphic-filter exec: no command provided");
        std::process::exit(1);
    }

    let start = Instant::now();

    // Execute the command via shell
    let output = Command::new("sh")
        .arg("-c")
        .arg(&command)
        .env(
            "PATH",
            std::env::var("PATH").unwrap_or_default(),
        )
        .output();

    let elapsed_ms = start.elapsed().as_millis() as u64;

    match output {
        Ok(result) => {
            let raw_stdout = String::from_utf8_lossy(&result.stdout).to_string();
            let raw_stderr = String::from_utf8_lossy(&result.stderr).to_string();

            // Combine stdout + stderr for filtering (Claude sees both)
            let combined = if raw_stderr.is_empty() {
                raw_stdout.clone()
            } else {
                format!("{raw_stdout}{raw_stderr}")
            };

            // Apply filter
            let (filtered, original_len, filtered_len) =
                filter::filter_output(&command, &combined);

            // Log savings (best-effort, don't fail on tracking errors)
            let project = std::env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();

            let _ = filter::tracker::SavingsTracker::record(
                &command,
                original_len,
                filtered_len,
                elapsed_ms,
                &project,
            );

            // Output filtered result
            print!("{filtered}");
            io::stdout().flush().ok();

            // Preserve exit code
            std::process::exit(result.status.code().unwrap_or(1));
        }
        Err(e) => {
            eprintln!("glyphic-filter: failed to execute command: {e}");
            std::process::exit(127);
        }
    }
}
