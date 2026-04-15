use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use super::config::{HEAD_LINES, TAIL_LINES, VIRTUALIZE_THRESHOLD_BYTES};
use super::db::{now_ts, Db, ToolResult};
use super::embed;

pub struct StoredResult {
    pub id: String,
    pub rendered: String,
    pub virtualized: bool,
    pub original_bytes: usize,
    pub virtualized_bytes: usize,
}

/// Decide whether the tool output should be stored + virtualized. Returns the
/// string Claude Code should see (either unchanged or a summary + expand
/// pointer), plus storage metadata for logging.
pub fn maybe_virtualize(
    db: &Db,
    session: &str,
    project: &str,
    tool: &str,
    args_summary: &str,
    output: &str,
    dedup_key: Option<&str>,
) -> Result<StoredResult, String> {
    let bytes = output.len();
    let line_count = output.lines().count();
    let id = compute_ref_id(session, tool, args_summary, output);

    let virtualized = should_virtualize(bytes);

    let summary = build_summary(output);
    let record = ToolResult {
        id: id.clone(),
        session: session.to_string(),
        ts: now_ts(),
        tool: tool.to_string(),
        args_summary: args_summary.to_string(),
        content: output.to_string(),
        summary: summary.clone(),
        size_bytes: bytes as i64,
        line_count: line_count as i64,
        project: project.to_string(),
        dedup_key: dedup_key.map(|s| s.to_string()),
    };
    // Embed the summary (shorter + distilled) for semantic rerank. If the
    // model isn't ready yet — first-run download in progress, or kill
    // switch — this silently falls through; the row gets NULL embedding
    // and the `reindex` command can backfill it later.
    let emb = embed::embed_one(&summary);
    db.insert_tool_result(&record, emb.as_deref())
        .map_err(|e| format!("insert tool_result: {e}"))?;

    let rendered = if virtualized {
        render_virtualized(&id, tool, bytes, line_count, output)
    } else {
        output.to_string()
    };

    Ok(StoredResult {
        id,
        virtualized_bytes: rendered.len(),
        rendered,
        virtualized,
        original_bytes: bytes,
    })
}

pub fn should_virtualize(bytes: usize) -> bool {
    bytes > VIRTUALIZE_THRESHOLD_BYTES
}

/// Stable, short reference id. Format: `tr_<8 hex chars>`.
fn compute_ref_id(session: &str, tool: &str, args: &str, output: &str) -> String {
    let mut h = DefaultHasher::new();
    session.hash(&mut h);
    tool.hash(&mut h);
    args.hash(&mut h);
    // Sample output to keep hashing cheap for huge payloads
    let sample_end = output.len().min(4096);
    output[..sample_end].hash(&mut h);
    output.len().hash(&mut h);
    now_ts().hash(&mut h);
    format!("tr_{:08x}", (h.finish() as u32))
}

fn build_summary(output: &str) -> String {
    let lines: Vec<&str> = output.lines().collect();
    if lines.len() <= HEAD_LINES + TAIL_LINES + 1 {
        return output.to_string();
    }
    let head: Vec<&str> = lines.iter().take(HEAD_LINES).copied().collect();
    let tail: Vec<&str> = lines
        .iter()
        .skip(lines.len() - TAIL_LINES)
        .copied()
        .collect();
    format!(
        "{head}\n… ({skipped} lines elided) …\n{tail}",
        head = head.join("\n"),
        skipped = lines.len() - HEAD_LINES - TAIL_LINES,
        tail = tail.join("\n"),
    )
}

fn render_virtualized(id: &str, tool: &str, bytes: usize, lines: usize, output: &str) -> String {
    let summary = build_summary(output);
    format!(
        "[glyphic:ref {id}] {tool} output virtualized — {lines} lines / {kb:.1} KB\n\
         To see a specific range, run: glyphic-ctx expand {id} --range START:END\n\
         To see everything, run: glyphic-ctx expand {id}\n\
         --- preview (head {head} / tail {tail}) ---\n{summary}",
        kb = bytes as f64 / 1024.0,
        head = HEAD_LINES,
        tail = TAIL_LINES,
    )
}

/// Render the stored content for the `expand` subcommand.
pub fn render_expand(tr: &ToolResult, range: Option<(usize, usize)>) -> String {
    let lines: Vec<&str> = tr.content.lines().collect();
    let total = lines.len();
    let (start, end) = match range {
        Some((s, e)) => (s.min(total), e.min(total)),
        None => (0, total),
    };
    if start >= end {
        return format!("[glyphic:ref {}] empty range {}:{}", tr.id, start, end);
    }
    let body: String = lines[start..end].join("\n");
    format!(
        "[glyphic:ref {id}] {tool} — lines {start}..{end} of {total}\n{body}",
        id = tr.id,
        tool = tr.tool,
    )
}
