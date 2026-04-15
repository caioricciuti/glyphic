//! glyphic-ctx — sidecar for the Context Engine.
//!
//! Subcommands:
//!   hook               Read a Claude Code hook JSON envelope from stdin, emit response.
//!   expand <ref>       Print stored tool-result content. Optional `--range A:B`.
//!   query <text>       Hybrid retrieval preview (BM25 + embedding rerank).
//!   reindex            Backfill embeddings for rows missing them.
//!   stats              Print DB stats (counts, bytes, embedding coverage).
//!   version            Print version.

use std::io::{self, Read};

use glyphic_lib::ctx::db::{Db, EmbedKind};
use glyphic_lib::ctx::{embed, hook, virtualize};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(|s| s.as_str()) {
        Some("hook") => run_hook(),
        Some("expand") => run_expand(&args[2..]),
        Some("query") => run_query(&args[2..]),
        Some("reindex") => run_reindex(),
        Some("stats") => run_stats(),
        Some("version") => println!("glyphic-ctx {}", env!("CARGO_PKG_VERSION")),
        _ => {
            eprintln!("Usage: glyphic-ctx <hook|expand|query|reindex|stats|version>");
            std::process::exit(1);
        }
    }
}

fn run_hook() {
    let mut input = String::new();
    if io::stdin().read_to_string(&mut input).is_err() {
        println!("{{}}");
        return;
    }
    println!("{}", hook::handle(&input));
}

fn run_expand(args: &[String]) {
    let id = match args.first() {
        Some(id) => id.clone(),
        None => {
            eprintln!("Usage: glyphic-ctx expand <ref_id> [--range START:END]");
            std::process::exit(1);
        }
    };

    let mut range: Option<(usize, usize)> = None;
    let mut it = args.iter().skip(1);
    while let Some(a) = it.next() {
        if a == "--range" {
            if let Some(val) = it.next() {
                let parts: Vec<&str> = val.split(':').collect();
                if parts.len() == 2 {
                    if let (Ok(s), Ok(e)) = (parts[0].parse::<usize>(), parts[1].parse::<usize>()) {
                        range = Some((s, e));
                    }
                }
            }
        }
    }

    let db = match Db::open() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("glyphic-ctx: db open failed: {e}");
            std::process::exit(1);
        }
    };
    match db.get_tool_result(&id) {
        Ok(Some(tr)) => println!("{}", virtualize::render_expand(&tr, range)),
        Ok(None) => {
            eprintln!("glyphic-ctx: ref {id} not found");
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("glyphic-ctx: db error: {e}");
            std::process::exit(1);
        }
    }
}

fn run_query(args: &[String]) {
    let q = args.join(" ");
    if q.is_empty() {
        eprintln!("Usage: glyphic-ctx query <text>");
        std::process::exit(1);
    }
    let db = match Db::open() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("glyphic-ctx: db open failed: {e}");
            std::process::exit(1);
        }
    };
    let project = std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();
    let project_opt = if project.is_empty() { None } else { Some(project.as_str()) };
    // Mirror hook behaviour: embed query, hybrid-search. If the model isn't
    // ready we transparently get BM25 order — same as a fresh install.
    let q_emb = embed::embed_one(&q);
    let hits = db
        .search_hybrid(&q, q_emb.as_deref(), project_opt, None, 30, 10)
        .unwrap_or_default();
    if hits.is_empty() {
        println!("(no results)");
        return;
    }
    for h in hits {
        println!(
            "[{kind} {id}] {tool} — {preview}",
            kind = h.kind,
            id = h.id,
            tool = h.tool.unwrap_or_default(),
            preview = h.preview.replace('\n', " "),
        );
    }
}

/// Backfill embeddings for any row with a NULL embedding. Safe to run
/// repeatedly; stops when the NULL pool is empty.
fn run_reindex() {
    const BATCH: usize = 32;
    let db = match Db::open() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("glyphic-ctx: db open failed: {e}");
            std::process::exit(1);
        }
    };
    let mut total: u64 = 0;
    loop {
        let rows = match db.rows_needing_embedding(BATCH) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("glyphic-ctx: rows_needing_embedding: {e}");
                std::process::exit(1);
            }
        };
        if rows.is_empty() {
            break;
        }
        let texts: Vec<&str> = rows.iter().map(|r| r.text.as_str()).collect();
        let embs = match embed::embed_batch(&texts) {
            Some(e) => e,
            None => {
                eprintln!(
                    "glyphic-ctx: embedding model not available — model download \
                     may still be in progress or disabled. Try again shortly."
                );
                std::process::exit(1);
            }
        };
        for (row, vec) in rows.iter().zip(embs.iter()) {
            let res = match row.kind {
                EmbedKind::ToolResult => db.update_tool_result_embedding(&row.id, vec),
                EmbedKind::Turn => db.update_turn_embedding(&row.id, vec),
            };
            if let Err(e) = res {
                eprintln!("glyphic-ctx: update embedding ({}): {e}", row.id);
            } else {
                total += 1;
            }
        }
        println!("embedded {total} rows…");
    }
    println!("done: {total} rows backfilled");
}

fn run_stats() {
    let db = match Db::open() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("glyphic-ctx: db open failed: {e}");
            std::process::exit(1);
        }
    };
    match db.stats() {
        Ok(s) => {
            println!("tool_results: {}", s.tool_results);
            println!("turns:        {}", s.turns);
            println!("bytes_stored: {}", s.bytes_stored);
        }
        Err(e) => {
            eprintln!("glyphic-ctx: stats error: {e}");
            std::process::exit(1);
        }
    }
    if let Ok(c) = db.embedded_counts() {
        println!(
            "embedded:     tool_results {}/{}, turns {}/{}",
            c.tool_results_embedded, c.tool_results_total, c.turns_embedded, c.turns_total,
        );
    }
}
