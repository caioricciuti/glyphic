use super::config::{RETRIEVE_CANDIDATES, RETRIEVE_TOP_K};
use super::db::{Db, RetrievedSnippet};

/// Render retrieved snippets as a compact additional-context block. Returned
/// string is empty if nothing relevant was found. `exclude_session` drops
/// rows from the active session so the injected context doesn't echo the
/// current conversation back. When `query_embedding` is provided, BM25
/// candidates are reranked by cosine similarity — "auth bug" surfaces prior
/// results about "login failing". When `None` (model not ready yet), we
/// silently fall back to pure BM25 order.
pub fn build_context_block(
    db: &Db,
    query: &str,
    query_embedding: Option<&[f32]>,
    project: Option<&str>,
    exclude_session: Option<&str>,
) -> String {
    let hits = match db.search_hybrid(
        query,
        query_embedding,
        project,
        exclude_session,
        RETRIEVE_CANDIDATES,
        RETRIEVE_TOP_K,
    ) {
        Ok(h) => h,
        Err(_) => return String::new(),
    };
    if hits.is_empty() {
        return String::new();
    }
    render_block(&hits)
}

fn render_block(hits: &[RetrievedSnippet]) -> String {
    let mut out = String::from(
        "<glyphic-context>\n\
         Relevant prior results. Use `glyphic-ctx expand <id>` via Bash to fetch full content.\n",
    );
    for h in hits {
        let tag = match h.kind.as_str() {
            "tool" => format!("tool={} ref={}", h.tool.clone().unwrap_or_default(), h.id),
            _ => format!("turn ref={}", h.id),
        };
        let preview = truncate(&h.preview, 300).replace('\n', " ");
        out.push_str(&format!("- [{tag}] {preview}\n"));
    }
    out.push_str("</glyphic-context>");
    out
}

fn truncate(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_string()
    } else {
        let head: String = s.chars().take(n).collect();
        format!("{head}…")
    }
}
