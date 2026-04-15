//! Hook dispatcher for glyphic-ctx. Reads a Claude Code hook JSON envelope from
//! stdin and emits a response JSON on stdout.
//!
//! Events handled:
//!   - PreToolUse(Bash)        → short-circuit `glyphic-ctx expand …` calls
//!   - PostToolUse(any)        → log to DB, inject additionalContext with ref
//!   - UserPromptSubmit        → index prompt, inject retrieval block
//!   - SessionStart            → inject last session summary if present
//!
//! All other events fall through to an empty allow/noop response.

use serde_json::{json, Value};

use super::config;
use super::db::{now_ts, Db, Turn};
use super::embed;
use super::retrieve;
use super::virtualize;

pub fn handle(input: &str) -> String {
    if config::is_disabled() {
        return allow_noop();
    }
    let v: Value = match serde_json::from_str(input) {
        Ok(v) => v,
        Err(_) => return allow_noop(),
    };
    let event = v
        .get("hook_event_name")
        .and_then(|x| x.as_str())
        .unwrap_or("");

    // Opening the DB is cheap; a failure disables the engine for this call.
    let db = match Db::open() {
        Ok(d) => d,
        Err(_) => return allow_noop(),
    };

    match event {
        "PreToolUse" => pre_tool_use(&db, &v),
        "PostToolUse" => post_tool_use(&db, &v),
        "UserPromptSubmit" => user_prompt_submit(&db, &v),
        "SessionStart" => session_start(&db, &v),
        _ => allow_noop(),
    }
}

// ── PreToolUse ─────────────────────────────────────────────────────────────

fn pre_tool_use(db: &Db, v: &Value) -> String {
    let tool_name = v.get("tool_name").and_then(|x| x.as_str()).unwrap_or("");
    if tool_name != "Bash" {
        return allow_pre();
    }
    let cmd = v
        .get("tool_input")
        .and_then(|ti| ti.get("command"))
        .and_then(|c| c.as_str())
        .unwrap_or("");

    if let Some(ref_id) = parse_expand_cmd(cmd) {
        if let Ok(Some(tr)) = db.get_tool_result(&ref_id) {
            let range = parse_range_arg(cmd);
            let rendered = virtualize::render_expand(&tr, range);
            let resp = json!({
                "hookSpecificOutput": {
                    "hookEventName": "PreToolUse",
                    "permissionDecision": "deny",
                    "permissionDecisionReason": format!("glyphic-ctx: expanded inline"),
                    "additionalContext": rendered,
                }
            });
            return resp.to_string();
        }
    }
    allow_pre()
}

fn parse_expand_cmd(cmd: &str) -> Option<String> {
    let trimmed = cmd.trim();
    let first_tokens: Vec<&str> = trimmed.split_whitespace().take(3).collect();
    if first_tokens.len() < 3 {
        return None;
    }
    if !first_tokens[0].ends_with("glyphic-ctx") {
        return None;
    }
    if first_tokens[1] != "expand" {
        return None;
    }
    Some(first_tokens[2].to_string())
}

fn parse_range_arg(cmd: &str) -> Option<(usize, usize)> {
    let mut tokens = cmd.split_whitespace();
    while let Some(t) = tokens.next() {
        if t == "--range" {
            if let Some(val) = tokens.next() {
                let mut parts = val.split(':');
                let start = parts.next()?.parse().ok()?;
                let end = parts.next()?.parse().ok()?;
                return Some((start, end));
            }
        }
    }
    None
}

// ── PostToolUse ────────────────────────────────────────────────────────────

fn post_tool_use(db: &Db, v: &Value) -> String {
    let tool = v.get("tool_name").and_then(|x| x.as_str()).unwrap_or("");
    if config::is_skipped_tool(tool) {
        return allow_noop();
    }

    let session = v
        .get("session_id")
        .and_then(|x| x.as_str())
        .unwrap_or("unknown");
    let project = v
        .get("cwd")
        .and_then(|x| x.as_str())
        .map(|s| s.to_string())
        .unwrap_or_default();

    let tool_input = v.get("tool_input");
    let output = extract_output(tool, v.get("tool_response"));
    if output.is_empty() {
        return allow_noop();
    }
    let args_summary = summarize_args(tool_input);
    let dedup_key = compute_dedup_key(tool, tool_input);

    let stored = match virtualize::maybe_virtualize(
        db, session, &project, tool, &args_summary, &output, dedup_key.as_deref(),
    ) {
        Ok(s) => s,
        Err(_) => return allow_noop(),
    };

    // We cannot rewrite the tool_response that Claude already received, but we
    // can inject a ref pointer so subsequent turns reference it. When the
    // output was virtualized, the ref comment also explains the expand flow.
    let note = if stored.virtualized {
        format!(
            "glyphic-ctx: tool output stored as {id} ({bytes} bytes). \
             Use `glyphic-ctx expand {id}` via Bash to re-read; future retrieval \
             will surface this automatically.",
            id = stored.id,
            bytes = stored.original_bytes,
        )
    } else {
        format!("glyphic-ctx: ref {id} ({bytes} bytes)", id = stored.id, bytes = stored.original_bytes)
    };

    json!({
        "hookSpecificOutput": {
            "hookEventName": "PostToolUse",
            "additionalContext": note,
        }
    })
    .to_string()
}

/// Extract human-readable text from a tool_response. Per-tool shape handlers
/// first, generic fallback only over known text-bearing keys. Returns an empty
/// string rather than a JSON dump when the shape is unknown — noise is worse
/// than a missing record.
fn extract_output(tool: &str, resp: Option<&Value>) -> String {
    let resp = match resp {
        Some(r) => r,
        None => return String::new(),
    };
    if let Value::String(s) = resp {
        return s.clone();
    }
    match tool {
        "Bash" => {
            let stdout = resp.get("stdout").and_then(|v| v.as_str()).unwrap_or("");
            let stderr = resp.get("stderr").and_then(|v| v.as_str()).unwrap_or("");
            if stderr.is_empty() { stdout.to_string() } else { format!("{stdout}\n{stderr}") }
        }
        "Read" => resp
            .pointer("/file/content")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_default(),
        "Grep" | "Glob" => resp
            .get("content")
            .or_else(|| resp.get("output"))
            .or_else(|| resp.get("results"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_default(),
        _ => generic_text(resp),
    }
}

/// Generic text extraction for unknown tool shapes. Looks at common text keys
/// and content-block arrays. No JSON dump fallback.
fn generic_text(v: &Value) -> String {
    if let Some(s) = v.as_str() { return s.to_string(); }
    if let Some(s) = v.get("content").and_then(|c| c.as_str()) { return s.to_string(); }
    if let Some(arr) = v.get("content").and_then(|c| c.as_array()) {
        let parts: Vec<&str> = arr
            .iter()
            .filter_map(|el| el.get("text").and_then(|t| t.as_str()))
            .collect();
        if !parts.is_empty() { return parts.join("\n"); }
    }
    if let Some(s) = v.get("text").and_then(|t| t.as_str()) { return s.to_string(); }
    if let Some(s) = v.get("output").and_then(|t| t.as_str()) { return s.to_string(); }
    String::new()
}

/// Derive a stable dedup key for tools where storing multiple copies of
/// "same thing, different timestamp" is pure noise. On insert, existing rows
/// with the same key are deleted first.
fn compute_dedup_key(tool: &str, input: Option<&Value>) -> Option<String> {
    let ti = input?;
    match tool {
        "Read" => ti.get("file_path").and_then(|v| v.as_str()).map(|p| format!("read:{p}")),
        "Glob" => ti.get("pattern").and_then(|v| v.as_str()).map(|p| format!("glob:{p}")),
        _ => None,
    }
}

fn summarize_args(input: Option<&Value>) -> String {
    let ti = match input {
        Some(t) => t,
        None => return String::new(),
    };
    let s = serde_json::to_string(ti).unwrap_or_default();
    if s.len() > 200 {
        s.chars().take(200).collect::<String>() + "…"
    } else {
        s
    }
}

// ── UserPromptSubmit ───────────────────────────────────────────────────────

fn user_prompt_submit(db: &Db, v: &Value) -> String {
    let prompt = v.get("prompt").and_then(|x| x.as_str()).unwrap_or("");
    if prompt.is_empty() {
        return allow_noop();
    }
    let session = v
        .get("session_id")
        .and_then(|x| x.as_str())
        .unwrap_or("unknown");
    let project = v
        .get("cwd")
        .and_then(|x| x.as_str())
        .map(|s| s.to_string())
        .unwrap_or_default();

    // Embed the prompt once — reused for both retrieval rerank and the
    // turn insert. On first run the model download happens here; returns
    // None and we transparently fall back to BM25-only for this turn.
    let q_emb = embed::embed_one(prompt);

    // Retrieve BEFORE indexing, otherwise the prompt matches itself.
    // Also exclude the current session so we don't echo back the active
    // conversation — prior sessions are where the signal lives.
    let project_opt = if project.is_empty() { None } else { Some(project.as_str()) };
    let block = retrieve::build_context_block(
        db,
        prompt,
        q_emb.as_deref(),
        project_opt,
        Some(session),
    );

    // Then index for future turns
    let turn = Turn {
        id: format!("u_{}_{}", session, now_ts()),
        session: session.to_string(),
        ts: now_ts(),
        role: "user".into(),
        content: prompt.to_string(),
        project: project.clone(),
    };
    let _ = db.insert_turn(&turn, q_emb.as_deref());

    if block.is_empty() {
        return allow_noop();
    }

    json!({
        "hookSpecificOutput": {
            "hookEventName": "UserPromptSubmit",
            "additionalContext": block,
        }
    })
    .to_string()
}

// ── SessionStart ───────────────────────────────────────────────────────────

fn session_start(_db: &Db, _v: &Value) -> String {
    // Placeholder — later phase will inject last session's structured summary.
    allow_noop()
}

// ── Helpers ────────────────────────────────────────────────────────────────

fn allow_noop() -> String {
    "{}".to_string()
}

fn allow_pre() -> String {
    json!({
        "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "permissionDecision": "allow"
        }
    })
    .to_string()
}
