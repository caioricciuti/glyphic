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
    // Sample output to keep hashing cheap for huge payloads. Walk back to a
    // char boundary so multi-byte content can't panic the slice.
    let mut sample_end = output.len().min(4096);
    while !output.is_char_boundary(sample_end) {
        sample_end -= 1;
    }
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

/// Render a stored turn (prompt or assistant message) for the `expand` flow.
pub fn render_turn_expand(t: &super::db::Turn, range: Option<(usize, usize)>) -> String {
    let lines: Vec<&str> = t.content.lines().collect();
    let total = lines.len();
    let (start, end) = match range {
        Some((s, e)) => (s.min(total), e.min(total)),
        None => (0, total),
    };
    if start >= end {
        return format!("[glyphic:ref {}] empty range {}:{}", t.id, start, end);
    }
    let body: String = lines[start..end].join("\n");
    format!(
        "[glyphic:ref {id}] turn:{role} — lines {start}..{end} of {total}\n{body}",
        id = t.id,
        role = t.role,
    )
}

#[cfg(test)]
mod tests {
    //! NOTE: tests that go through `maybe_virtualize` use whitespace-only
    //! output on purpose. `embed::embed_one` short-circuits to `None` on
    //! whitespace before touching the model, so nothing here can trigger the
    //! fastembed download. Everything else avoids the embed path entirely.
    use super::*;
    use crate::ctx::config::{HEAD_LINES, TAIL_LINES, VIRTUALIZE_THRESHOLD_BYTES};

    fn mem_db() -> Db {
        Db::open_in_memory().expect("open in-memory db")
    }

    // ── should_virtualize / thresholds ─────────────────────────────────────

    #[test]
    fn virtualize_threshold_is_strictly_greater_than() {
        assert!(!should_virtualize(0));
        assert!(!should_virtualize(VIRTUALIZE_THRESHOLD_BYTES));
        assert!(should_virtualize(VIRTUALIZE_THRESHOLD_BYTES + 1));
    }

    // ── ref id generation ──────────────────────────────────────────────────

    #[test]
    fn ref_id_has_stable_format() {
        let id = compute_ref_id("sess", "Bash", "ls -la", "some output");
        assert!(id.starts_with("tr_"), "got {id}");
        assert_eq!(id.len(), 11);
        assert!(id[3..].chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn ref_id_differs_for_different_inputs() {
        let a = compute_ref_id("sess", "Bash", "ls", "output-a");
        let b = compute_ref_id("sess", "Bash", "ls", "output-b");
        let c = compute_ref_id("sess", "Grep", "ls", "output-a");
        assert_ne!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn ref_id_does_not_panic_on_multibyte_output_past_sample_boundary() {
        // 3-byte chars: byte 4096 is not a char boundary (4096 % 3 != 0).
        // Regression: the sample slice used to panic here.
        let output = "€".repeat(2000); // 6000 bytes
        let id = compute_ref_id("sess", "Bash", "cat euros", &output);
        assert!(id.starts_with("tr_"));
    }

    // ── build_summary ──────────────────────────────────────────────────────

    #[test]
    fn short_output_passes_through_summary_unchanged() {
        let lines = HEAD_LINES + TAIL_LINES + 1;
        let output = (0..lines).map(|i| format!("line {i}")).collect::<Vec<_>>().join("\n");
        assert_eq!(build_summary(&output), output);
        assert_eq!(build_summary("single line"), "single line");
        assert_eq!(build_summary(""), "");
    }

    #[test]
    fn long_output_summary_keeps_head_and_tail_and_counts_elided() {
        let total = HEAD_LINES + TAIL_LINES + 7;
        let output = (0..total).map(|i| format!("line {i}")).collect::<Vec<_>>().join("\n");
        let summary = build_summary(&output);

        assert!(summary.contains("… (7 lines elided) …"));
        assert!(summary.starts_with("line 0\n"));
        assert!(summary.ends_with(&format!("line {}", total - 1)));
        // Last head line and first tail line survive; first elided line is gone.
        assert!(summary.contains(&format!("line {}\n", HEAD_LINES - 1)));
        assert!(summary.contains(&format!("line {}", total - TAIL_LINES)));
        assert!(!summary.contains(&format!("line {}\n", HEAD_LINES)));
    }

    // ── maybe_virtualize (whitespace-only, see module note) ────────────────

    #[test]
    fn small_output_is_stored_but_rendered_verbatim() {
        let db = mem_db();
        let output = "  \n   ";
        let res = maybe_virtualize(&db, "sess-1", "proj-1", "Bash", "echo", output, None).unwrap();

        assert!(!res.virtualized);
        assert_eq!(res.rendered, output);
        assert_eq!(res.original_bytes, output.len());
        assert_eq!(res.virtualized_bytes, output.len());

        let row = db.get_tool_result(&res.id).unwrap().expect("stored");
        assert_eq!(row.content, output);
        assert_eq!(row.session, "sess-1");
        assert_eq!(row.project, "proj-1");
        assert_eq!(row.tool, "Bash");
        assert_eq!(row.args_summary, "echo");
        assert_eq!(row.size_bytes, output.len() as i64);
        assert_eq!(row.line_count, 2);
        assert!(row.dedup_key.is_none());
    }

    #[test]
    fn oversized_output_is_virtualized_with_expand_pointer() {
        let db = mem_db();
        // 3 lines x 1000 spaces = 3002 bytes: over the byte threshold, but few
        // enough lines that the summary stays whitespace (keeps embed inert).
        let output = vec![" ".repeat(1000); 3].join("\n");
        let res = maybe_virtualize(&db, "sess-1", "proj-1", "Bash", "big", &output, None).unwrap();

        assert!(res.virtualized);
        assert_eq!(res.original_bytes, output.len());
        assert_eq!(res.virtualized_bytes, res.rendered.len());
        assert!(res.rendered.starts_with(&format!("[glyphic:ref {}] Bash output virtualized", res.id)));
        assert!(res.rendered.contains("3 lines"));
        assert!(res.rendered.contains(&format!("glyphic-ctx expand {} --range START:END", res.id)));
        assert!(res.rendered.contains(&format!("glyphic-ctx expand {}\n", res.id)));

        // Full content is still retrievable from the store.
        let row = db.get_tool_result(&res.id).unwrap().expect("stored");
        assert_eq!(row.content, output);
        assert_eq!(row.line_count, 3);
    }

    #[test]
    fn render_virtualized_includes_header_pointer_and_elided_preview() {
        let total = HEAD_LINES + TAIL_LINES + 7;
        let output = (0..total).map(|i| format!("line {i}")).collect::<Vec<_>>().join("\n");
        let rendered = render_virtualized("tr_abc12345", "Bash", 4096, total, &output);

        assert!(rendered.starts_with(&format!(
            "[glyphic:ref tr_abc12345] Bash output virtualized — {total} lines / 4.0 KB"
        )));
        assert!(rendered.contains("glyphic-ctx expand tr_abc12345 --range START:END"));
        assert!(rendered.contains(&format!("preview (head {HEAD_LINES} / tail {TAIL_LINES})")));
        assert!(rendered.contains("… (7 lines elided) …"));
        assert!(rendered.contains("line 0"));
        assert!(rendered.ends_with(&format!("line {}", total - 1)));
    }

    #[test]
    fn dedup_key_replaces_prior_stored_result() {
        let db = mem_db();
        let first = maybe_virtualize(&db, "s", "p", "Bash", "args-one", " \n ", Some("dk")).unwrap();
        let second = maybe_virtualize(&db, "s", "p", "Bash", "args-two", " \n \n ", Some("dk")).unwrap();
        assert_ne!(first.id, second.id);
        assert!(db.get_tool_result(&first.id).unwrap().is_none(), "deduped row must be gone");
        let row = db.get_tool_result(&second.id).unwrap().expect("latest kept");
        assert_eq!(row.dedup_key.as_deref(), Some("dk"));
        assert_eq!(row.args_summary, "args-two");
    }

    // ── render_expand / render_turn_expand ─────────────────────────────────

    fn sample_tr(content: &str) -> ToolResult {
        ToolResult {
            id: "tr_test01".to_string(),
            session: "s".to_string(),
            ts: 1,
            tool: "Grep".to_string(),
            args_summary: "pattern".to_string(),
            content: content.to_string(),
            summary: content.to_string(),
            size_bytes: content.len() as i64,
            line_count: content.lines().count() as i64,
            project: "p".to_string(),
            dedup_key: None,
        }
    }

    #[test]
    fn render_expand_full_and_ranged() {
        let tr = sample_tr("alpha\nbeta\ngamma\ndelta");

        let full = render_expand(&tr, None);
        assert_eq!(full, "[glyphic:ref tr_test01] Grep — lines 0..4 of 4\nalpha\nbeta\ngamma\ndelta");

        let ranged = render_expand(&tr, Some((1, 3)));
        assert_eq!(ranged, "[glyphic:ref tr_test01] Grep — lines 1..3 of 4\nbeta\ngamma");
    }

    #[test]
    fn render_expand_clamps_out_of_bounds_ranges() {
        let tr = sample_tr("alpha\nbeta");
        // End past EOF clamps to total.
        let clamped = render_expand(&tr, Some((1, 99)));
        assert_eq!(clamped, "[glyphic:ref tr_test01] Grep — lines 1..2 of 2\nbeta");
        // Fully past EOF collapses to empty.
        assert_eq!(render_expand(&tr, Some((5, 9))), "[glyphic:ref tr_test01] empty range 2:2");
        // Inverted range reports as empty.
        assert_eq!(render_expand(&tr, Some((2, 1))), "[glyphic:ref tr_test01] empty range 2:1");
    }

    #[test]
    fn render_expand_handles_multibyte_lines() {
        let tr = sample_tr("café résumé\nnaïve €100");
        let out = render_expand(&tr, Some((1, 2)));
        assert!(out.ends_with("naïve €100"));
    }

    #[test]
    fn render_turn_expand_full_ranged_and_empty() {
        let t = super::super::db::Turn {
            id: "tn_test01".to_string(),
            session: "s".to_string(),
            ts: 1,
            role: "assistant".to_string(),
            content: "one\ntwo\nthree".to_string(),
            project: "p".to_string(),
        };
        assert_eq!(
            render_turn_expand(&t, None),
            "[glyphic:ref tn_test01] turn:assistant — lines 0..3 of 3\none\ntwo\nthree"
        );
        assert_eq!(
            render_turn_expand(&t, Some((0, 1))),
            "[glyphic:ref tn_test01] turn:assistant — lines 0..1 of 3\none"
        );
        assert_eq!(render_turn_expand(&t, Some((3, 3))), "[glyphic:ref tn_test01] empty range 3:3");
    }
}
