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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ctx::db::{Db, ToolResult, Turn};

    fn mem_db() -> Db {
        Db::open_in_memory().expect("open in-memory db")
    }

    fn seed_tool(db: &Db, id: &str, session: &str, content: &str, embedding: Option<&[f32]>) {
        let r = ToolResult {
            id: id.to_string(),
            session: session.to_string(),
            ts: 1_000,
            tool: "Bash".to_string(),
            args_summary: format!("args-{id}"),
            content: content.to_string(),
            summary: content.to_string(),
            size_bytes: content.len() as i64,
            line_count: content.lines().count() as i64,
            project: "proj".to_string(),
            dedup_key: None,
        };
        db.insert_tool_result(&r, embedding).unwrap();
    }

    fn seed_turn(db: &Db, id: &str, session: &str, content: &str) {
        let t = Turn {
            id: id.to_string(),
            session: session.to_string(),
            ts: 1_000,
            role: "user".to_string(),
            content: content.to_string(),
            project: "proj".to_string(),
        };
        db.insert_turn(&t, None).unwrap();
    }

    /// First `ref=<id>` appearing in a rendered block.
    fn first_ref(block: &str) -> &str {
        let start = block.find("ref=").expect("block has a ref") + 4;
        let rest = &block[start..];
        let end = rest.find(']').expect("ref is bracketed");
        &rest[..end]
    }

    // ── truncate ───────────────────────────────────────────────────────────

    #[test]
    fn truncate_passes_short_strings_and_appends_ellipsis() {
        assert_eq!(truncate("short", 10), "short");
        assert_eq!(truncate("exactly", 7), "exactly");
        assert_eq!(truncate("abcdefgh", 3), "abc…");
        assert_eq!(truncate("", 5), "");
        // Char-based, so multi-byte input cannot split a code point.
        assert_eq!(truncate("ééééé", 3), "ééé…");
    }

    // ── render_block ───────────────────────────────────────────────────────

    #[test]
    fn render_block_formats_tool_and_turn_hits() {
        let hits = vec![
            RetrievedSnippet {
                kind: "tool".to_string(),
                id: "tr_aaaa0001".to_string(),
                ts: 1,
                tool: Some("Grep".to_string()),
                preview: "match one\nmatch two".to_string(),
            },
            RetrievedSnippet {
                kind: "turn".to_string(),
                id: "tn_bbbb0002".to_string(),
                ts: 2,
                tool: None,
                preview: "earlier question".to_string(),
            },
        ];
        let block = render_block(&hits);
        assert!(block.starts_with("<glyphic-context>\n"));
        assert!(block.ends_with("</glyphic-context>"));
        assert!(block.contains("glyphic-ctx expand <id>"));
        // Newlines inside previews are flattened so the block stays line-oriented.
        assert!(block.contains("- [tool=Grep ref=tr_aaaa0001] match one match two\n"));
        assert!(block.contains("- [turn ref=tn_bbbb0002] earlier question\n"));
    }

    #[test]
    fn render_block_truncates_long_previews_at_300_chars() {
        let hits = vec![RetrievedSnippet {
            kind: "turn".to_string(),
            id: "tn_long0001".to_string(),
            ts: 1,
            tool: None,
            preview: "x".repeat(400),
        }];
        let block = render_block(&hits);
        let line = block.lines().find(|l| l.starts_with("- [")).unwrap();
        assert!(line.ends_with('…'));
        assert!(line.contains(&"x".repeat(300)));
        assert!(!line.contains(&"x".repeat(301)));
    }

    // ── build_context_block ────────────────────────────────────────────────

    #[test]
    fn context_block_is_empty_when_nothing_matches() {
        let db = mem_db();
        assert_eq!(build_context_block(&db, "anything relevant", None, None, None), "");
        seed_tool(&db, "tr_1", "s1", "cargo build finished", None);
        // Query sanitizes to nothing → no search → empty block.
        assert_eq!(build_context_block(&db, "the a an", None, None, None), "");
        // Query matches nothing stored.
        assert_eq!(build_context_block(&db, "zebra xylophone", None, None, None), "");
    }

    #[test]
    fn context_block_renders_matching_hits() {
        let db = mem_db();
        seed_tool(&db, "tr_build", "s1", "cargo build failed with linker error", None);
        seed_turn(&db, "tn_build", "s2", "why did the cargo build fail");

        let block = build_context_block(&db, "cargo build failure", None, None, None);
        assert!(block.starts_with("<glyphic-context>"));
        assert!(block.ends_with("</glyphic-context>"));
        assert!(block.contains("ref=tr_build"));
        assert!(block.contains("ref=tn_build"));
        assert!(block.contains("tool=Bash"));
    }

    #[test]
    fn context_block_excludes_active_session() {
        let db = mem_db();
        seed_tool(&db, "tr_cur", "sess-cur", "flaky test timeout in ci", None);
        seed_tool(&db, "tr_old", "sess-old", "flaky test timeout in ci", None);

        let block = build_context_block(&db, "flaky test timeout", None, None, Some("sess-cur"));
        assert!(block.contains("ref=tr_old"));
        assert!(!block.contains("ref=tr_cur"));
    }

    // ── retrieval quality eval (BM25 + synthetic vectors, no model) ────────

    /// (id, body, 10-dim one-hot index)
    const DOCS: &[(&str, &str)] = &[
        ("d_pg", "postgres index bloat: ran vacuum analyze, query planner now uses the btree index, slow query fixed by reindex and tuning work_mem"),
        ("d_docker", "docker network inspect shows the bridge network, containers attached to docker bridge with port mapping and overlay networking"),
        ("d_tls", "tls handshake failed: certificate expired yesterday, renewed the letsencrypt certificate chain with openssl and reloaded nginx"),
        ("d_git", "git rebase interactive hit a conflict, resolved the conflict markers, continued the rebase and squashed the fixup commits"),
        ("d_sqlite", "sqlite fts5 virtual table created with porter tokenizer, full text search over documents ranked by bm25, fts index rebuilt"),
        ("d_react", "react component rerenders too often, memoized props with usememo and moved state down, useeffect dependency array fixed"),
        ("d_k8s", "kubernetes pod stuck pending, scheduler could not place replica on node, kubelet taint removed and deployment rolled out"),
        ("d_py", "python virtualenv recreated with uv, pip dependency resolution conflict fixed by pinning packages in requirements"),
        ("d_dns", "dns lookup failure: nameserver in resolv conf unreachable, added fallback resolver, domain record propagated"),
        ("d_rust", "rust borrow checker error: lifetime does not live long enough, fixed by cloning at the boundary, cargo build green"),
    ];

    fn one_hot(i: usize) -> Vec<f32> {
        let mut v = vec![0.0f32; 10];
        v[i] = 1.0;
        v
    }

    fn seed_eval_corpus(db: &Db) {
        for (i, (id, body)) in DOCS.iter().enumerate() {
            seed_tool(db, id, "sess-eval", body, Some(&one_hot(i)));
        }
    }

    #[test]
    fn eval_bm25_hit_at_1_across_topics() {
        let db = mem_db();
        seed_eval_corpus(&db);

        let cases = [
            ("postgres slow query index tuning", "d_pg"),
            ("docker bridge networking containers", "d_docker"),
            ("expired tls certificate renewal", "d_tls"),
            ("resolve git rebase conflict", "d_git"),
            ("kubernetes pod pending scheduler", "d_k8s"),
        ];
        for (query, expected) in cases {
            let hits = db.search(query, None, None, 5).unwrap();
            assert!(!hits.is_empty(), "no hits for {query:?}");
            assert_eq!(hits[0].id, expected, "hit@1 failed for {query:?}");
        }
    }

    #[test]
    fn eval_hybrid_hit_at_1_with_synthetic_vectors() {
        let db = mem_db();
        seed_eval_corpus(&db);

        let cases = [
            ("sqlite full text search fts5 bm25", 4, "d_sqlite"),
            ("react useeffect rerender state", 5, "d_react"),
            ("python uv pip dependency conflict", 7, "d_py"),
            ("dns nameserver lookup failure", 8, "d_dns"),
            ("rust borrow checker lifetime error", 9, "d_rust"),
        ];
        for (query, hot, expected) in cases {
            let q = one_hot(hot);
            let hits = db.search_hybrid(query, Some(&q), None, None, 30, 5).unwrap();
            assert!(!hits.is_empty(), "no hits for {query:?}");
            assert_eq!(hits[0].id, expected, "hit@1 failed for {query:?}");
        }
    }

    #[test]
    fn eval_embedding_rerank_disambiguates_lexically_close_docs() {
        let db = mem_db();
        seed_eval_corpus(&db);
        // "index" and "query/search" overlap between the postgres and sqlite
        // docs; the query vector decides which one wins.
        let query = "index search query performance";
        let toward_sqlite = db.search_hybrid(query, Some(&one_hot(4)), None, None, 30, 5).unwrap();
        assert_eq!(toward_sqlite[0].id, "d_sqlite");
        let toward_pg = db.search_hybrid(query, Some(&one_hot(0)), None, None, 30, 5).unwrap();
        assert_eq!(toward_pg[0].id, "d_pg");
    }

    #[test]
    fn eval_context_block_surfaces_best_doc_first() {
        let db = mem_db();
        seed_eval_corpus(&db);
        let block = build_context_block(
            &db,
            "expired tls certificate renewal",
            Some(&one_hot(2)),
            None,
            None,
        );
        assert!(!block.is_empty());
        assert_eq!(first_ref(&block), "d_tls");
    }
}
