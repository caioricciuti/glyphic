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

/// Read Claude Code hook JSON from stdin, dispatch by tool type.
fn handle_hook() {
    let mut input = String::new();
    if io::stdin().read_to_string(&mut input).is_err() {
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
    let tool_input = json.get("tool_input");

    match tool_name {
        "Bash" => handle_bash_hook(tool_input),
        "Read" => handle_read_hook(tool_input),
        "Grep" => handle_grep_hook(tool_input),
        _ => print_allow(),
    }
}

/// Bash hook: wrap command through glyphic-filter exec for output filtering.
fn handle_bash_hook(tool_input: Option<&serde_json::Value>) {
    let command = tool_input
        .and_then(|ti| ti.get("command"))
        .and_then(|c| c.as_str())
        .unwrap_or("");

    if command.is_empty() || should_exclude(command) {
        print_allow();
        return;
    }

    let bin = filter::tracker::SavingsTracker::bin_path();
    let bin_str = bin.to_string_lossy();
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
}

/// Read hook: set smart `limit` based on file size to prevent oversized reads.
fn handle_read_hook(tool_input: Option<&serde_json::Value>) {
    let ti = match tool_input {
        Some(v) => v,
        None => { print_allow(); return; }
    };

    // If limit or offset already set, Claude is being intentional — don't touch
    if ti.get("limit").and_then(|v| v.as_u64()).is_some()
        || ti.get("offset").and_then(|v| v.as_u64()).is_some()
    {
        print_allow();
        return;
    }

    let file_path = match ti.get("file_path").and_then(|v| v.as_str()) {
        Some(p) => p,
        None => { print_allow(); return; }
    };

    // Skip binary/non-text extensions
    if is_binary_extension(file_path) {
        print_allow();
        return;
    }

    // Get file size via stat (sub-millisecond, no I/O beyond syscall)
    let file_size = match std::fs::metadata(file_path) {
        Ok(m) => m.len(),
        Err(_) => { print_allow(); return; } // file doesn't exist — let Claude's tool handle the error
    };

    // Estimate line count (avg ~50 bytes per line for source code)
    let estimated_lines = file_size / 50;

    // Conservative tiered limits
    let limit: Option<u64> = if estimated_lines < 300 {
        None // small file, full read is fine
    } else if estimated_lines < 1000 {
        Some(500)
    } else if estimated_lines < 3000 {
        Some(300)
    } else {
        Some(200)
    };

    let limit = match limit {
        Some(l) => l,
        None => { print_allow(); return; }
    };

    // Log estimated savings
    let estimated_output_bytes = limit as usize * 50;
    let project = std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    let ext = std::path::Path::new(file_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("unknown");
    let cmd_label = format!("Read .{ext}");

    let _ = filter::tracker::SavingsTracker::record(
        &cmd_label,
        file_size as usize,
        estimated_output_bytes,
        0,
        &project,
        "Read",
    );

    // Return updated input with limit
    let mut updated = ti.clone();
    updated.as_object_mut().map(|o| o.insert("limit".to_string(), serde_json::json!(limit)));

    let response = serde_json::json!({
        "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "permissionDecision": "allow",
            "permissionDecisionReason": format!("glyphic-filter: capping read to {limit} lines (file ~{estimated_lines} lines)"),
            "updatedInput": updated
        }
    });
    println!("{}", serde_json::to_string(&response).unwrap());
}

/// Grep hook: reduce head_limit to prevent oversized search results.
fn handle_grep_hook(tool_input: Option<&serde_json::Value>) {
    let ti = match tool_input {
        Some(v) => v,
        None => { print_allow(); return; }
    };

    let current_limit = ti.get("head_limit").and_then(|v| v.as_u64());

    // If explicitly set below 100, Claude chose a conservative limit — don't touch
    if let Some(limit) = current_limit {
        if limit < 100 {
            print_allow();
            return;
        }
    }

    let output_mode = ti.get("output_mode")
        .and_then(|v| v.as_str())
        .unwrap_or("files_with_matches");

    let new_limit: u64 = match output_mode {
        "content" => 75,
        "count" => { print_allow(); return; } // count output is tiny
        _ => 100, // files_with_matches and others
    };

    // Don't modify if current limit is already at or below our target
    if let Some(current) = current_limit {
        if current <= new_limit {
            print_allow();
            return;
        }
    }

    let original_limit = current_limit.unwrap_or(250); // Claude's default

    // Log estimated savings
    let bytes_per_result: usize = if output_mode == "content" { 120 } else { 80 };
    let input_bytes = original_limit as usize * bytes_per_result;
    let output_bytes = new_limit as usize * bytes_per_result;
    let project = std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    let _ = filter::tracker::SavingsTracker::record(
        &format!("Grep {output_mode}"),
        input_bytes,
        output_bytes,
        0,
        &project,
        "Grep",
    );

    let mut updated = ti.clone();
    updated.as_object_mut().map(|o| o.insert("head_limit".to_string(), serde_json::json!(new_limit)));

    let response = serde_json::json!({
        "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "permissionDecision": "allow",
            "permissionDecisionReason": format!("glyphic-filter: reducing head_limit from {original_limit} to {new_limit}"),
            "updatedInput": updated
        }
    });
    println!("{}", serde_json::to_string(&response).unwrap());
}

/// Check if file extension suggests binary/non-text content.
fn is_binary_extension(path: &str) -> bool {
    let binary_exts = [
        "png", "jpg", "jpeg", "gif", "bmp", "ico", "svg", "webp",
        "pdf", "wasm", "bin", "exe", "dll", "so", "dylib",
        "zip", "tar", "gz", "bz2", "xz", "7z", "rar",
        "mp3", "mp4", "avi", "mov", "mkv", "wav", "flac",
        "ttf", "otf", "woff", "woff2", "eot",
        "lock", "lockb",
        "pyc", "pyo", "class",
        "sqlite", "db",
    ];
    std::path::Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|ext| binary_exts.contains(&ext.to_lowercase().as_str()))
        .unwrap_or(false)
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
                "Bash",
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
