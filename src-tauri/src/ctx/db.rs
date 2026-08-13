use rusqlite::{params, Connection};
use std::time::{SystemTime, UNIX_EPOCH};

use super::config;
use super::embed;

pub struct Db {
    conn: Connection,
}

pub struct ToolResult {
    pub id: String,
    pub session: String,
    pub ts: u64,
    pub tool: String,
    pub args_summary: String,
    pub content: String,
    pub summary: String,
    pub size_bytes: i64,
    pub line_count: i64,
    pub project: String,
    pub dedup_key: Option<String>,
}

pub struct Turn {
    pub id: String,
    pub session: String,
    pub ts: u64,
    pub role: String,
    pub content: String,
    pub project: String,
}

pub struct RetrievedSnippet {
    pub kind: String,
    pub id: String,
    pub ts: u64,
    pub tool: Option<String>,
    pub preview: String,
}

/// Row that still needs an embedding. Drives the `reindex` backfill loop.
pub struct RowToEmbed {
    pub kind: EmbedKind,
    pub id: String,
    pub text: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EmbedKind {
    ToolResult,
    Turn,
}

pub struct EmbeddedCounts {
    pub tool_results_total: i64,
    pub tool_results_embedded: i64,
    pub turns_total: i64,
    pub turns_embedded: i64,
}

impl Db {
    pub fn open() -> rusqlite::Result<Self> {
        let path = config::db_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let conn = Connection::open(path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        let db = Self { conn };
        db.migrate()?;
        Ok(db)
    }

    /// Test-only: fully migrated in-memory database. Keeps unit tests off the
    /// real `~/.glyphic/ctx.db` and away from the filesystem entirely.
    #[cfg(test)]
    pub(crate) fn open_in_memory() -> rusqlite::Result<Self> {
        let conn = Connection::open_in_memory()?;
        let db = Self { conn };
        db.migrate()?;
        Ok(db)
    }

    fn migrate(&self) -> rusqlite::Result<()> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS tool_results (
                id TEXT PRIMARY KEY,
                session TEXT NOT NULL,
                ts INTEGER NOT NULL,
                tool TEXT NOT NULL,
                args_summary TEXT NOT NULL,
                content TEXT NOT NULL,
                summary TEXT NOT NULL,
                size_bytes INTEGER NOT NULL,
                line_count INTEGER NOT NULL,
                project TEXT NOT NULL DEFAULT ''
            );
            CREATE INDEX IF NOT EXISTS idx_tr_session_ts ON tool_results(session, ts);
            CREATE INDEX IF NOT EXISTS idx_tr_project_ts ON tool_results(project, ts);

            CREATE TABLE IF NOT EXISTS turns (
                id TEXT PRIMARY KEY,
                session TEXT NOT NULL,
                ts INTEGER NOT NULL,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                project TEXT NOT NULL DEFAULT ''
            );
            CREATE INDEX IF NOT EXISTS idx_turns_session_ts ON turns(session, ts);
            CREATE INDEX IF NOT EXISTS idx_turns_project_ts ON turns(project, ts);

            CREATE VIRTUAL TABLE IF NOT EXISTS tool_results_fts
                USING fts5(id UNINDEXED, content, summary, tokenize='porter unicode61');
            CREATE VIRTUAL TABLE IF NOT EXISTS turns_fts
                USING fts5(id UNINDEXED, content, tokenize='porter unicode61');

            CREATE TABLE IF NOT EXISTS sessions_summary (
                session TEXT PRIMARY KEY,
                project TEXT NOT NULL DEFAULT '',
                updated_at INTEGER NOT NULL,
                payload TEXT NOT NULL
            );
            "#,
        )?;
        // Idempotent ADD COLUMN — ignore error if already present.
        let _ = self.conn.execute("ALTER TABLE tool_results ADD COLUMN dedup_key TEXT", []);
        self.conn.execute_batch(
            "CREATE INDEX IF NOT EXISTS idx_tr_dedup ON tool_results(dedup_key);",
        )?;
        // Embedding BLOBs — BGE-Small-EN-v1.5 outputs 384 × f32 = 1536 bytes.
        // Stored as little-endian raw bytes; NULL means "not embedded yet"
        // (new install, failed embed, pre-embedding row).
        let _ = self.conn.execute("ALTER TABLE tool_results ADD COLUMN embedding BLOB", []);
        let _ = self.conn.execute("ALTER TABLE turns ADD COLUMN embedding BLOB", []);
        Ok(())
    }

    // ── Inserts ────────────────────────────────────────────────────────────

    pub fn insert_tool_result(&self, r: &ToolResult, embedding: Option<&[f32]>) -> rusqlite::Result<()> {
        // Same-key rows are noise (e.g. Read of the same file N times).
        // Drop their FTS rows first so the virtual table doesn't dangle.
        if let Some(key) = r.dedup_key.as_deref() {
            self.conn.execute(
                "DELETE FROM tool_results_fts
                 WHERE id IN (SELECT id FROM tool_results WHERE dedup_key = ?1)",
                params![key],
            )?;
            self.conn.execute(
                "DELETE FROM tool_results WHERE dedup_key = ?1",
                params![key],
            )?;
        }
        let emb_blob: Option<Vec<u8>> = embedding.map(embed::encode);
        self.conn.execute(
            "INSERT OR REPLACE INTO tool_results
             (id, session, ts, tool, args_summary, content, summary, size_bytes, line_count, project, dedup_key, embedding)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                r.id, r.session, r.ts as i64, r.tool, r.args_summary,
                r.content, r.summary, r.size_bytes, r.line_count, r.project,
                r.dedup_key, emb_blob,
            ],
        )?;
        // FTS5 has no PK on the UNINDEXED id column, so INSERT OR REPLACE
        // would stack duplicate rows for the same id. Scrub first.
        self.conn.execute(
            "DELETE FROM tool_results_fts WHERE id = ?1",
            params![r.id],
        )?;
        self.conn.execute(
            "INSERT INTO tool_results_fts (id, content, summary) VALUES (?1, ?2, ?3)",
            params![r.id, r.content, r.summary],
        )?;
        Ok(())
    }

    pub fn insert_turn(&self, t: &Turn, embedding: Option<&[f32]>) -> rusqlite::Result<()> {
        let emb_blob: Option<Vec<u8>> = embedding.map(embed::encode);
        self.conn.execute(
            "INSERT OR REPLACE INTO turns (id, session, ts, role, content, project, embedding)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![t.id, t.session, t.ts as i64, t.role, t.content, t.project, emb_blob],
        )?;
        // Same duplicate-row hazard as tool_results_fts: scrub by id first.
        self.conn.execute("DELETE FROM turns_fts WHERE id = ?1", params![t.id])?;
        self.conn.execute(
            "INSERT INTO turns_fts (id, content) VALUES (?1, ?2)",
            params![t.id, t.content],
        )?;
        Ok(())
    }

    /// Update the embedding for an existing tool_result. Used by the
    /// `reindex` backfill path so we don't have to re-derive summaries etc.
    pub fn update_tool_result_embedding(&self, id: &str, embedding: &[f32]) -> rusqlite::Result<()> {
        self.conn.execute(
            "UPDATE tool_results SET embedding = ?1 WHERE id = ?2",
            params![embed::encode(embedding), id],
        )?;
        Ok(())
    }

    pub fn update_turn_embedding(&self, id: &str, embedding: &[f32]) -> rusqlite::Result<()> {
        self.conn.execute(
            "UPDATE turns SET embedding = ?1 WHERE id = ?2",
            params![embed::encode(embedding), id],
        )?;
        Ok(())
    }

    /// Rows still missing an embedding. Small shape so we can batch-embed
    /// without pulling whole bodies more than once.
    pub fn rows_needing_embedding(&self, batch: usize) -> rusqlite::Result<Vec<RowToEmbed>> {
        let mut out: Vec<RowToEmbed> = Vec::new();
        {
            let mut stmt = self.conn.prepare(
                "SELECT id, summary FROM tool_results WHERE embedding IS NULL LIMIT ?1",
            )?;
            let rows = stmt.query_map(params![batch as i64], |row| {
                Ok(RowToEmbed {
                    kind: EmbedKind::ToolResult,
                    id: row.get(0)?,
                    text: row.get(1)?,
                })
            })?;
            for r in rows.flatten() {
                out.push(r);
            }
        }
        let remaining = batch.saturating_sub(out.len());
        if remaining > 0 {
            let mut stmt = self.conn.prepare(
                "SELECT id, content FROM turns WHERE embedding IS NULL LIMIT ?1",
            )?;
            let rows = stmt.query_map(params![remaining as i64], |row| {
                Ok(RowToEmbed {
                    kind: EmbedKind::Turn,
                    id: row.get(0)?,
                    text: row.get(1)?,
                })
            })?;
            for r in rows.flatten() {
                out.push(r);
            }
        }
        Ok(out)
    }

    /// Delete rows that shouldn't be in the store anymore:
    ///   * tool is in the caller-supplied skip list (new `SKIP_TOOLS`
    ///     entries didn't exist when those rows were written)
    ///   * content looks like a raw JSON envelope (`{"…`) — these come from
    ///     pre-per-tool-`extract_output` runs that dumped the whole
    ///     `tool_response` instead of the human-readable payload.
    ///
    /// FTS5 virtual rows are scrubbed first so the index never dangles.
    /// Returns the count of `tool_results` rows removed.
    pub fn purge_legacy_tool_results(&self, skip_tools: &[&str]) -> rusqlite::Result<i64> {
        let mut deleted: i64 = 0;

        // Skip-listed tools first.
        if !skip_tools.is_empty() {
            let placeholders = skip_tools
                .iter()
                .enumerate()
                .map(|(i, _)| format!("?{}", i + 1))
                .collect::<Vec<_>>()
                .join(",");
            let fts_sql = format!(
                "DELETE FROM tool_results_fts
                 WHERE id IN (SELECT id FROM tool_results WHERE tool IN ({placeholders}))"
            );
            let del_sql = format!("DELETE FROM tool_results WHERE tool IN ({placeholders})");
            let params_box: Vec<Box<dyn rusqlite::ToSql>> = skip_tools
                .iter()
                .map(|t| Box::new(t.to_string()) as Box<dyn rusqlite::ToSql>)
                .collect();
            let params_ref: Vec<&dyn rusqlite::ToSql> =
                params_box.iter().map(|b| b.as_ref()).collect();
            self.conn.execute(&fts_sql, params_ref.as_slice())?;
            deleted += self.conn.execute(&del_sql, params_ref.as_slice())? as i64;
        }

        // Raw-JSON-envelope rows (pre-per-tool extractor).
        self.conn.execute(
            "DELETE FROM tool_results_fts
             WHERE id IN (SELECT id FROM tool_results WHERE substr(content, 1, 2) = '{\"')",
            [],
        )?;
        deleted += self.conn.execute(
            "DELETE FROM tool_results WHERE substr(content, 1, 2) = '{\"'",
            [],
        )? as i64;

        Ok(deleted)
    }

    pub fn embedded_counts(&self) -> rusqlite::Result<EmbeddedCounts> {
        let tr_total: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM tool_results", [], |r| r.get(0))?;
        let tr_embedded: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM tool_results WHERE embedding IS NOT NULL",
            [],
            |r| r.get(0),
        )?;
        let tu_total: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM turns", [], |r| r.get(0))?;
        let tu_embedded: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM turns WHERE embedding IS NOT NULL",
            [],
            |r| r.get(0),
        )?;
        Ok(EmbeddedCounts {
            tool_results_total: tr_total,
            tool_results_embedded: tr_embedded,
            turns_total: tu_total,
            turns_embedded: tu_embedded,
        })
    }

    // ── Reads ──────────────────────────────────────────────────────────────

    pub fn get_tool_result(&self, id: &str) -> rusqlite::Result<Option<ToolResult>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, session, ts, tool, args_summary, content, summary, size_bytes, line_count, project, dedup_key
             FROM tool_results WHERE id = ?1",
        )?;
        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(map_tool_result(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn get_turn(&self, id: &str) -> rusqlite::Result<Option<Turn>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, session, ts, role, content, project FROM turns WHERE id = ?1",
        )?;
        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Turn {
                id: row.get(0)?,
                session: row.get(1)?,
                ts: row.get::<_, i64>(2)? as u64,
                role: row.get(3)?,
                content: row.get(4)?,
                project: row.get(5)?,
            }))
        } else {
            Ok(None)
        }
    }

    /// FTS5 search across tool_results + turns, scoped to a project when
    /// provided. When `exclude_session` is set, rows from that session are
    /// filtered out — used so UserPromptSubmit doesn't echo the current
    /// conversation back at itself. Returns up to `limit` items ranked by
    /// bm25 score.
    pub fn search(
        &self,
        query: &str,
        project: Option<&str>,
        exclude_session: Option<&str>,
        limit: usize,
    ) -> rusqlite::Result<Vec<RetrievedSnippet>> {
        let fts_query = sanitize_fts_query(query);
        if fts_query.is_empty() {
            return Ok(Vec::new());
        }

        let mut results: Vec<RetrievedSnippet> = Vec::new();

        // ── tool_results side ──
        // Build SQL + bind list dynamically so we don't explode into four
        // hardcoded variants.
        let mut sql_tr = String::from(
            "SELECT tr.id, tr.ts, tr.tool, tr.summary
             FROM tool_results_fts f
             JOIN tool_results tr ON tr.id = f.id
             WHERE tool_results_fts MATCH ?",
        );
        let mut binds: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(fts_query.clone())];
        if let Some(proj) = project {
            sql_tr.push_str(" AND tr.project = ?");
            binds.push(Box::new(proj.to_string()));
        }
        if let Some(sess) = exclude_session {
            sql_tr.push_str(" AND tr.session <> ?");
            binds.push(Box::new(sess.to_string()));
        }
        sql_tr.push_str(" ORDER BY bm25(tool_results_fts) LIMIT ?");
        binds.push(Box::new(limit as i64));

        let bind_refs: Vec<&dyn rusqlite::ToSql> = binds.iter().map(|b| b.as_ref()).collect();
        let mut stmt = self.conn.prepare(&sql_tr)?;
        let rows: Vec<RetrievedSnippet> = stmt
            .query_map(bind_refs.as_slice(), |row| {
                Ok(RetrievedSnippet {
                    kind: "tool".into(),
                    id: row.get(0)?,
                    ts: row.get::<_, i64>(1)? as u64,
                    tool: Some(row.get(2)?),
                    preview: row.get(3)?,
                })
            })?
            .filter_map(Result::ok)
            .collect();
        results.extend(rows);

        // ── turns side ──
        let mut sql_turns = String::from(
            "SELECT t.id, t.ts, t.content
             FROM turns_fts f
             JOIN turns t ON t.id = f.id
             WHERE turns_fts MATCH ?",
        );
        let mut binds2: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(fts_query)];
        if let Some(proj) = project {
            sql_turns.push_str(" AND t.project = ?");
            binds2.push(Box::new(proj.to_string()));
        }
        if let Some(sess) = exclude_session {
            sql_turns.push_str(" AND t.session <> ?");
            binds2.push(Box::new(sess.to_string()));
        }
        sql_turns.push_str(" ORDER BY bm25(turns_fts) LIMIT ?");
        binds2.push(Box::new(limit as i64));

        let bind_refs2: Vec<&dyn rusqlite::ToSql> = binds2.iter().map(|b| b.as_ref()).collect();
        let mut stmt2 = self.conn.prepare(&sql_turns)?;
        let rows2: Vec<RetrievedSnippet> = stmt2
            .query_map(bind_refs2.as_slice(), |row| {
                Ok(RetrievedSnippet {
                    kind: "turn".into(),
                    id: row.get(0)?,
                    ts: row.get::<_, i64>(1)? as u64,
                    tool: None,
                    preview: first_n_chars(&row.get::<_, String>(2)?, 240),
                })
            })?
            .filter_map(Result::ok)
            .collect();
        results.extend(rows2);

        results.truncate(limit);
        Ok(results)
    }

    /// Hybrid retrieval: BM25 recall → embedding rerank.
    ///
    /// Pulls up to `candidates` top-bm25 rows from both `tool_results` and
    /// `turns`, decodes each row's embedding, scores cosine similarity
    /// against `query_embedding`, sorts descending, returns top `limit`.
    ///
    /// Rows without an embedding fall back to a score of 0 — they still
    /// appear if the candidate pool is short, but get outranked by any row
    /// with even weak semantic overlap. When `query_embedding` is `None` this
    /// collapses to the same ordering as `search()`.
    pub fn search_hybrid(
        &self,
        query: &str,
        query_embedding: Option<&[f32]>,
        project: Option<&str>,
        exclude_session: Option<&str>,
        candidates: usize,
        limit: usize,
    ) -> rusqlite::Result<Vec<RetrievedSnippet>> {
        let fts_query = sanitize_fts_query(query);
        if fts_query.is_empty() {
            return Ok(Vec::new());
        }

        // Candidate carries enough to score + return, plus an optional
        // embedding BLOB for reranking.
        struct Cand {
            snip: RetrievedSnippet,
            embedding: Option<Vec<u8>>,
        }
        let mut cands: Vec<Cand> = Vec::new();

        // ── tool_results side ──
        let mut sql_tr = String::from(
            "SELECT tr.id, tr.ts, tr.tool, tr.summary, tr.embedding
             FROM tool_results_fts f
             JOIN tool_results tr ON tr.id = f.id
             WHERE tool_results_fts MATCH ?",
        );
        let mut binds: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(fts_query.clone())];
        if let Some(proj) = project {
            sql_tr.push_str(" AND tr.project = ?");
            binds.push(Box::new(proj.to_string()));
        }
        if let Some(sess) = exclude_session {
            sql_tr.push_str(" AND tr.session <> ?");
            binds.push(Box::new(sess.to_string()));
        }
        sql_tr.push_str(" ORDER BY bm25(tool_results_fts) LIMIT ?");
        binds.push(Box::new(candidates as i64));
        let bind_refs: Vec<&dyn rusqlite::ToSql> = binds.iter().map(|b| b.as_ref()).collect();
        let mut stmt = self.conn.prepare(&sql_tr)?;
        let iter = stmt.query_map(bind_refs.as_slice(), |row| {
            Ok(Cand {
                snip: RetrievedSnippet {
                    kind: "tool".into(),
                    id: row.get(0)?,
                    ts: row.get::<_, i64>(1)? as u64,
                    tool: Some(row.get(2)?),
                    preview: row.get(3)?,
                },
                embedding: row.get(4)?,
            })
        })?;
        for c in iter.flatten() {
            cands.push(c);
        }

        // ── turns side ──
        let mut sql_turns = String::from(
            "SELECT t.id, t.ts, t.content, t.embedding
             FROM turns_fts f
             JOIN turns t ON t.id = f.id
             WHERE turns_fts MATCH ?",
        );
        let mut binds2: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(fts_query)];
        if let Some(proj) = project {
            sql_turns.push_str(" AND t.project = ?");
            binds2.push(Box::new(proj.to_string()));
        }
        if let Some(sess) = exclude_session {
            sql_turns.push_str(" AND t.session <> ?");
            binds2.push(Box::new(sess.to_string()));
        }
        sql_turns.push_str(" ORDER BY bm25(turns_fts) LIMIT ?");
        binds2.push(Box::new(candidates as i64));
        let bind_refs2: Vec<&dyn rusqlite::ToSql> = binds2.iter().map(|b| b.as_ref()).collect();
        let mut stmt2 = self.conn.prepare(&sql_turns)?;
        let iter2 = stmt2.query_map(bind_refs2.as_slice(), |row| {
            Ok(Cand {
                snip: RetrievedSnippet {
                    kind: "turn".into(),
                    id: row.get(0)?,
                    ts: row.get::<_, i64>(1)? as u64,
                    tool: None,
                    preview: first_n_chars(&row.get::<_, String>(2)?, 240),
                },
                embedding: row.get(3)?,
            })
        })?;
        for c in iter2.flatten() {
            cands.push(c);
        }

        // ── rerank ──
        // No query embedding → preserve BM25 order. With a query embedding,
        // score each candidate by cosine; missing embeddings score 0.
        if let Some(q) = query_embedding {
            let mut scored: Vec<(f32, RetrievedSnippet)> = cands
                .into_iter()
                .map(|c| {
                    let score = c
                        .embedding
                        .as_ref()
                        .and_then(|b| embed::decode(b))
                        .map(|v| embed::cosine(q, &v))
                        .unwrap_or(0.0);
                    (score, c.snip)
                })
                .collect();
            // Descending by cosine; NaN sorts low via total_cmp.
            scored.sort_by(|a, b| b.0.total_cmp(&a.0));
            let mut out: Vec<RetrievedSnippet> =
                scored.into_iter().map(|(_, s)| s).collect();
            out.truncate(limit);
            Ok(out)
        } else {
            let mut out: Vec<RetrievedSnippet> =
                cands.into_iter().map(|c| c.snip).collect();
            out.truncate(limit);
            Ok(out)
        }
    }

    pub fn stats(&self) -> rusqlite::Result<DbStats> {
        let tool_count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM tool_results", [], |r| r.get(0))?;
        let turn_count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM turns", [], |r| r.get(0))?;
        let bytes_stored: i64 = self
            .conn
            .query_row(
                "SELECT COALESCE(SUM(size_bytes), 0) FROM tool_results",
                [],
                |r| r.get(0),
            )?;
        Ok(DbStats {
            tool_results: tool_count,
            turns: turn_count,
            bytes_stored,
        })
    }

    pub fn recent_tool_results(&self, project: Option<&str>, limit: usize) -> rusqlite::Result<Vec<ToolResult>> {
        let sql = match project {
            Some(_) => "SELECT id, session, ts, tool, args_summary, content, summary, size_bytes, line_count, project, dedup_key
                        FROM tool_results WHERE project = ?1 ORDER BY ts DESC LIMIT ?2",
            None => "SELECT id, session, ts, tool, args_summary, content, summary, size_bytes, line_count, project, dedup_key
                     FROM tool_results ORDER BY ts DESC LIMIT ?1",
        };
        let mut stmt = self.conn.prepare(sql)?;
        let rows: Vec<ToolResult> = if let Some(proj) = project {
            stmt.query_map(params![proj, limit as i64], map_tool_result)?
                .filter_map(Result::ok)
                .collect()
        } else {
            stmt.query_map(params![limit as i64], map_tool_result)?
                .filter_map(Result::ok)
                .collect()
        };
        Ok(rows)
    }
}

pub struct DbStats {
    pub tool_results: i64,
    pub turns: i64,
    pub bytes_stored: i64,
}

fn map_tool_result(row: &rusqlite::Row) -> rusqlite::Result<ToolResult> {
    Ok(ToolResult {
        id: row.get(0)?,
        session: row.get(1)?,
        ts: row.get::<_, i64>(2)? as u64,
        tool: row.get(3)?,
        args_summary: row.get(4)?,
        content: row.get(5)?,
        summary: row.get(6)?,
        size_bytes: row.get(7)?,
        line_count: row.get(8)?,
        project: row.get(9)?,
        dedup_key: row.get(10)?,
    })
}

/// Unix seconds.
pub fn now_ts() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn first_n_chars(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}

/// Build an FTS5 MATCH query from free text. Strategy:
///     - strip non-alphanumeric to spaces
///     - drop tokens < 3 chars and very common stopwords
///     - cap at 12 tokens
///     - join with OR so any token match surfaces the row; bm25 ranks on overlap
///
/// Returns an empty string when nothing useful remains.
fn sanitize_fts_query(raw: &str) -> String {
    const STOP: &[&str] = &[
        "the", "and", "for", "with", "that", "this", "from", "into", "you",
        "are", "was", "were", "have", "has", "but", "not", "all", "any",
        "can", "will", "would", "should", "could", "does", "did", "been",
        "how", "why", "what", "when", "where", "who", "show", "get", "let",
    ];
    let cleaned: String = raw
        .chars()
        .map(|c| if c.is_alphanumeric() || c.is_whitespace() { c } else { ' ' })
        .collect();
    let tokens: Vec<String> = cleaned
        .split_whitespace()
        .map(|t| t.to_lowercase())
        .filter(|t| t.len() >= 3 && !STOP.contains(&t.as_str()))
        .take(12)
        .collect();
    if tokens.is_empty() {
        return String::new();
    }
    tokens
        .iter()
        .map(|t| format!("\"{t}\""))
        .collect::<Vec<_>>()
        .join(" OR ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mem_db() -> Db {
        Db::open_in_memory().expect("open in-memory db")
    }

    fn make_tr(
        id: &str,
        session: &str,
        project: &str,
        ts: u64,
        content: &str,
        dedup: Option<&str>,
    ) -> ToolResult {
        ToolResult {
            id: id.to_string(),
            session: session.to_string(),
            ts,
            tool: "Bash".to_string(),
            args_summary: format!("args-{id}"),
            content: content.to_string(),
            summary: content.to_string(),
            size_bytes: content.len() as i64,
            line_count: content.lines().count() as i64,
            project: project.to_string(),
            dedup_key: dedup.map(str::to_string),
        }
    }

    fn make_turn(id: &str, session: &str, project: &str, ts: u64, content: &str) -> Turn {
        Turn {
            id: id.to_string(),
            session: session.to_string(),
            ts,
            role: "user".to_string(),
            content: content.to_string(),
            project: project.to_string(),
        }
    }

    fn table_count(db: &Db, table: &str) -> i64 {
        db.conn
            .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0))
            .unwrap()
    }

    // ── schema ─────────────────────────────────────────────────────────────

    #[test]
    fn migrate_creates_all_tables_and_is_idempotent() {
        let db = mem_db();
        // Running the migration again must not error (ALTER TABLE guards).
        db.migrate().expect("second migrate");

        let mut stmt = db
            .conn
            .prepare("SELECT name FROM sqlite_master WHERE type IN ('table','index') ORDER BY name")
            .unwrap();
        let names: Vec<String> = stmt
            .query_map([], |r| r.get::<_, String>(0))
            .unwrap()
            .filter_map(Result::ok)
            .collect();
        for expected in [
            "tool_results",
            "turns",
            "tool_results_fts",
            "turns_fts",
            "sessions_summary",
            "idx_tr_session_ts",
            "idx_tr_project_ts",
            "idx_tr_dedup",
            "idx_turns_session_ts",
            "idx_turns_project_ts",
        ] {
            assert!(names.iter().any(|n| n == expected), "missing {expected}");
        }
        // Migration must have added the late columns.
        let cols: i64 = db
            .conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('tool_results') WHERE name IN ('dedup_key','embedding')",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(cols, 2);
    }

    // ── insert / read roundtrip ────────────────────────────────────────────

    #[test]
    fn tool_result_roundtrip_preserves_all_fields() {
        let db = mem_db();
        let r = make_tr("tr_1", "sess-a", "proj-x", 1234, "hello roundtrip world", Some("key-1"));
        let emb = [0.25f32, -1.5, 3.0];
        db.insert_tool_result(&r, Some(&emb)).unwrap();

        let got = db.get_tool_result("tr_1").unwrap().expect("row present");
        assert_eq!(got.id, "tr_1");
        assert_eq!(got.session, "sess-a");
        assert_eq!(got.ts, 1234);
        assert_eq!(got.tool, "Bash");
        assert_eq!(got.args_summary, "args-tr_1");
        assert_eq!(got.content, "hello roundtrip world");
        assert_eq!(got.summary, "hello roundtrip world");
        assert_eq!(got.size_bytes, r.content.len() as i64);
        assert_eq!(got.line_count, 1);
        assert_eq!(got.project, "proj-x");
        assert_eq!(got.dedup_key.as_deref(), Some("key-1"));

        // Embedding blob survives encode → store → decode.
        let blob: Vec<u8> = db
            .conn
            .query_row("SELECT embedding FROM tool_results WHERE id = 'tr_1'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(embed::decode(&blob).unwrap(), emb.to_vec());
    }

    #[test]
    fn turn_roundtrip_preserves_all_fields() {
        let db = mem_db();
        let t = make_turn("tn_1", "sess-a", "proj-x", 99, "what is the plan");
        db.insert_turn(&t, None).unwrap();
        let got = db.get_turn("tn_1").unwrap().expect("row present");
        assert_eq!(got.id, "tn_1");
        assert_eq!(got.session, "sess-a");
        assert_eq!(got.ts, 99);
        assert_eq!(got.role, "user");
        assert_eq!(got.content, "what is the plan");
        assert_eq!(got.project, "proj-x");
    }

    #[test]
    fn get_missing_rows_return_none() {
        let db = mem_db();
        assert!(db.get_tool_result("nope").unwrap().is_none());
        assert!(db.get_turn("nope").unwrap().is_none());
    }

    // ── dedup_key ──────────────────────────────────────────────────────────

    #[test]
    fn dedup_key_deletes_previous_rows_before_insert() {
        let db = mem_db();
        db.insert_tool_result(
            &make_tr("tr_old", "s1", "p", 1, "olduniquetoken content", Some("read:/tmp/f")),
            None,
        )
        .unwrap();
        db.insert_tool_result(
            &make_tr("tr_new", "s1", "p", 2, "newuniquetoken content", Some("read:/tmp/f")),
            None,
        )
        .unwrap();

        assert!(db.get_tool_result("tr_old").unwrap().is_none(), "old row must be gone");
        assert!(db.get_tool_result("tr_new").unwrap().is_some());
        assert_eq!(table_count(&db, "tool_results"), 1);
        // FTS side scrubbed too: the old token no longer matches anything.
        assert!(db.search("olduniquetoken", None, None, 10).unwrap().is_empty());
        let hits = db.search("newuniquetoken", None, None, 10).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].id, "tr_new");
    }

    #[test]
    fn rows_without_dedup_key_accumulate() {
        let db = mem_db();
        db.insert_tool_result(&make_tr("tr_a", "s1", "p", 1, "alpha", None), None).unwrap();
        db.insert_tool_result(&make_tr("tr_b", "s1", "p", 2, "alpha", None), None).unwrap();
        assert_eq!(table_count(&db, "tool_results"), 2);
    }

    #[test]
    fn reinserting_same_id_leaves_single_fts_row() {
        let db = mem_db();
        let r = make_tr("tr_same", "s1", "p", 1, "repeatable token", None);
        db.insert_tool_result(&r, None).unwrap();
        db.insert_tool_result(&r, None).unwrap();
        assert_eq!(table_count(&db, "tool_results"), 1);
        assert_eq!(table_count(&db, "tool_results_fts"), 1);
        let hits = db.search("repeatable", None, None, 10).unwrap();
        assert_eq!(hits.len(), 1, "duplicate FTS rows would duplicate hits");

        let t = make_turn("tn_same", "s1", "p", 1, "turnrepeat token");
        db.insert_turn(&t, None).unwrap();
        db.insert_turn(&t, None).unwrap();
        assert_eq!(table_count(&db, "turns_fts"), 1);
    }

    // ── FTS search ─────────────────────────────────────────────────────────

    #[test]
    fn search_returns_matching_tool_and_turn_rows() {
        let db = mem_db();
        db.insert_tool_result(
            &make_tr("tr_pg", "s1", "p", 1, "postgres vacuum finished successfully", None),
            None,
        )
        .unwrap();
        db.insert_tool_result(&make_tr("tr_js", "s1", "p", 2, "eslint passed cleanly", None), None)
            .unwrap();
        db.insert_turn(&make_turn("tn_pg", "s2", "p", 3, "please vacuum the postgres table"), None)
            .unwrap();

        let hits = db.search("postgres vacuum", None, None, 10).unwrap();
        let ids: Vec<&str> = hits.iter().map(|h| h.id.as_str()).collect();
        assert!(ids.contains(&"tr_pg"));
        assert!(ids.contains(&"tn_pg"));
        assert!(!ids.contains(&"tr_js"));

        let tool_hit = hits.iter().find(|h| h.id == "tr_pg").unwrap();
        assert_eq!(tool_hit.kind, "tool");
        assert_eq!(tool_hit.tool.as_deref(), Some("Bash"));
        assert_eq!(tool_hit.preview, "postgres vacuum finished successfully");
        let turn_hit = hits.iter().find(|h| h.id == "tn_pg").unwrap();
        assert_eq!(turn_hit.kind, "turn");
        assert!(turn_hit.tool.is_none());
    }

    #[test]
    fn search_scopes_by_project() {
        let db = mem_db();
        db.insert_tool_result(&make_tr("tr_p1", "s1", "proj-one", 1, "shared needle", None), None)
            .unwrap();
        db.insert_tool_result(&make_tr("tr_p2", "s1", "proj-two", 2, "shared needle", None), None)
            .unwrap();
        let hits = db.search("needle shared", Some("proj-one"), None, 10).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].id, "tr_p1");
    }

    #[test]
    fn search_excludes_active_session_rows() {
        let db = mem_db();
        db.insert_tool_result(&make_tr("tr_cur", "sess-cur", "p", 1, "sessionscoped needle", None), None)
            .unwrap();
        db.insert_tool_result(&make_tr("tr_other", "sess-old", "p", 2, "sessionscoped needle", None), None)
            .unwrap();
        db.insert_turn(&make_turn("tn_cur", "sess-cur", "p", 3, "sessionscoped needle turn"), None)
            .unwrap();
        db.insert_turn(&make_turn("tn_other", "sess-old", "p", 4, "sessionscoped needle turn"), None)
            .unwrap();

        let hits = db.search("sessionscoped needle", None, Some("sess-cur"), 10).unwrap();
        let ids: Vec<&str> = hits.iter().map(|h| h.id.as_str()).collect();
        assert_eq!(ids, vec!["tr_other", "tn_other"]);
    }

    #[test]
    fn search_with_useless_query_returns_empty() {
        let db = mem_db();
        db.insert_tool_result(&make_tr("tr_x", "s", "p", 1, "content here", None), None).unwrap();
        // Stopwords and short tokens only → sanitized to empty → no results.
        assert!(db.search("the and for it a", None, None, 10).unwrap().is_empty());
        assert!(db.search("!!! ???", None, None, 10).unwrap().is_empty());
    }

    #[test]
    fn search_truncates_to_limit() {
        let db = mem_db();
        for i in 0..6 {
            db.insert_tool_result(
                &make_tr(&format!("tr_{i}"), "s", "p", i as u64, "limited needle row", None),
                None,
            )
            .unwrap();
        }
        assert_eq!(db.search("limited needle", None, None, 3).unwrap().len(), 3);
    }

    #[test]
    fn turn_preview_is_capped_at_240_chars_and_multibyte_safe() {
        let db = mem_db();
        let long = format!("previewtoken {}", "é".repeat(300));
        db.insert_turn(&make_turn("tn_long", "s", "p", 1, &long), None).unwrap();
        let hits = db.search("previewtoken", None, None, 10).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].preview.chars().count(), 240);
        assert_eq!(hits[0].preview, first_n_chars(&long, 240));
    }

    // ── sanitize_fts_query ─────────────────────────────────────────────────

    #[test]
    fn sanitize_drops_stopwords_short_tokens_and_punctuation() {
        assert_eq!(
            sanitize_fts_query("Fix the DB-connection bug now!"),
            "\"fix\" OR \"connection\" OR \"bug\" OR \"now\""
        );
        assert_eq!(sanitize_fts_query("the and for with"), "");
        assert_eq!(sanitize_fts_query("a b c"), "");
        assert_eq!(sanitize_fts_query(""), "");
    }

    #[test]
    fn sanitize_caps_token_count_at_twelve() {
        let raw = (0..20).map(|i| format!("token{i:02}")).collect::<Vec<_>>().join(" ");
        let q = sanitize_fts_query(&raw);
        assert_eq!(q.split(" OR ").count(), 12);
        assert!(q.starts_with("\"token00\""));
    }

    // ── hybrid search / rerank ─────────────────────────────────────────────

    #[test]
    fn hybrid_without_query_embedding_matches_bm25_search() {
        let db = mem_db();
        db.insert_tool_result(&make_tr("tr_1", "s", "p", 1, "hybrid needle one", None), None).unwrap();
        db.insert_tool_result(&make_tr("tr_2", "s", "p", 2, "hybrid needle two", None), None).unwrap();
        let plain: Vec<String> = db
            .search("hybrid needle", None, None, 10)
            .unwrap()
            .into_iter()
            .map(|h| h.id)
            .collect();
        let hybrid: Vec<String> = db
            .search_hybrid("hybrid needle", None, None, None, 30, 10)
            .unwrap()
            .into_iter()
            .map(|h| h.id)
            .collect();
        assert_eq!(plain, hybrid);
    }

    #[test]
    fn hybrid_reranks_by_cosine_and_scores_missing_embeddings_zero() {
        let db = mem_db();
        let ea = [0.9f32, 0.1, 0.0, 0.0];
        let eb = [0.0f32, 1.0, 0.0, 0.0];
        db.insert_tool_result(&make_tr("tr_a", "s", "p", 1, "rerank shared token", None), Some(&ea))
            .unwrap();
        db.insert_tool_result(&make_tr("tr_b", "s", "p", 2, "rerank shared token", None), Some(&eb))
            .unwrap();
        db.insert_tool_result(&make_tr("tr_none", "s", "p", 3, "rerank shared token", None), None)
            .unwrap();

        let q = [0.0f32, 1.0, 0.0, 0.0];
        let ids: Vec<String> = db
            .search_hybrid("rerank shared", Some(&q), None, None, 30, 10)
            .unwrap()
            .into_iter()
            .map(|h| h.id)
            .collect();
        // tr_b: cos = 1.0; tr_a: cos ≈ 0.11; tr_none: no embedding → 0.0
        assert_eq!(ids, vec!["tr_b", "tr_a", "tr_none"]);
    }

    #[test]
    fn hybrid_respects_limit_and_session_exclusion() {
        let db = mem_db();
        let e = [1.0f32, 0.0];
        for i in 0..5 {
            db.insert_tool_result(
                &make_tr(&format!("tr_{i}"), "sess-cur", "p", i as u64, "hybscope needle", None),
                Some(&e),
            )
            .unwrap();
        }
        db.insert_tool_result(&make_tr("tr_keep", "sess-old", "p", 9, "hybscope needle", None), Some(&e))
            .unwrap();
        let hits = db
            .search_hybrid("hybscope needle", Some(&e), None, Some("sess-cur"), 30, 3)
            .unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].id, "tr_keep");
    }

    // ── embedding backfill plumbing ────────────────────────────────────────

    #[test]
    fn rows_needing_embedding_and_updates_drive_counts() {
        let db = mem_db();
        let e = [0.5f32, 0.5];
        db.insert_tool_result(&make_tr("tr_has", "s", "p", 1, "embedded row", None), Some(&e)).unwrap();
        db.insert_tool_result(&make_tr("tr_miss", "s", "p", 2, "bare row", None), None).unwrap();
        db.insert_turn(&make_turn("tn_has", "s", "p", 3, "embedded turn"), Some(&e)).unwrap();
        db.insert_turn(&make_turn("tn_miss", "s", "p", 4, "bare turn"), None).unwrap();

        let c = db.embedded_counts().unwrap();
        assert_eq!((c.tool_results_total, c.tool_results_embedded), (2, 1));
        assert_eq!((c.turns_total, c.turns_embedded), (2, 1));

        let todo = db.rows_needing_embedding(10).unwrap();
        assert_eq!(todo.len(), 2);
        assert!(todo.iter().any(|r| r.kind == EmbedKind::ToolResult && r.id == "tr_miss" && r.text == "bare row"));
        assert!(todo.iter().any(|r| r.kind == EmbedKind::Turn && r.id == "tn_miss" && r.text == "bare turn"));

        // Batch smaller than the tool-result backlog never reaches turns.
        let todo_one = db.rows_needing_embedding(1).unwrap();
        assert_eq!(todo_one.len(), 1);
        assert_eq!(todo_one[0].kind, EmbedKind::ToolResult);

        db.update_tool_result_embedding("tr_miss", &e).unwrap();
        db.update_turn_embedding("tn_miss", &e).unwrap();
        assert!(db.rows_needing_embedding(10).unwrap().is_empty());
        let c2 = db.embedded_counts().unwrap();
        assert_eq!((c2.tool_results_embedded, c2.turns_embedded), (2, 2));
    }

    // ── purge / stats / recency ────────────────────────────────────────────

    #[test]
    fn purge_removes_skip_listed_and_json_envelope_rows() {
        let db = mem_db();
        db.insert_tool_result(&make_tr("tr_read", "s", "p", 1, "purgeread payload", None), None).unwrap();
        db.conn
            .execute("UPDATE tool_results SET tool = 'Read' WHERE id = 'tr_read'", [])
            .unwrap();
        db.insert_tool_result(
            &make_tr("tr_json", "s", "p", 2, "{\"raw\": \"jsonenvelope\"}", None),
            None,
        )
        .unwrap();
        db.insert_tool_result(&make_tr("tr_clean", "s", "p", 3, "cleanrow survives", None), None).unwrap();

        let deleted = db.purge_legacy_tool_results(&["Read", "TodoWrite"]).unwrap();
        assert_eq!(deleted, 2);
        assert!(db.get_tool_result("tr_read").unwrap().is_none());
        assert!(db.get_tool_result("tr_json").unwrap().is_none());
        assert!(db.get_tool_result("tr_clean").unwrap().is_some());
        // FTS index scrubbed alongside.
        assert!(db.search("purgeread", None, None, 10).unwrap().is_empty());
        assert!(db.search("jsonenvelope", None, None, 10).unwrap().is_empty());
        assert_eq!(db.search("cleanrow", None, None, 10).unwrap().len(), 1);

        // Empty skip list still purges envelopes, and is a no-op here.
        assert_eq!(db.purge_legacy_tool_results(&[]).unwrap(), 0);
    }

    #[test]
    fn stats_counts_rows_and_sums_bytes() {
        let db = mem_db();
        let s = db.stats().unwrap();
        assert_eq!((s.tool_results, s.turns, s.bytes_stored), (0, 0, 0));

        db.insert_tool_result(&make_tr("tr_1", "s", "p", 1, "12345", None), None).unwrap();
        db.insert_tool_result(&make_tr("tr_2", "s", "p", 2, "1234567890", None), None).unwrap();
        db.insert_turn(&make_turn("tn_1", "s", "p", 3, "turn"), None).unwrap();
        let s = db.stats().unwrap();
        assert_eq!(s.tool_results, 2);
        assert_eq!(s.turns, 1);
        assert_eq!(s.bytes_stored, 15);
    }

    #[test]
    fn recent_tool_results_orders_by_ts_desc_and_filters_project() {
        let db = mem_db();
        db.insert_tool_result(&make_tr("tr_1", "s", "pa", 10, "one", None), None).unwrap();
        db.insert_tool_result(&make_tr("tr_2", "s", "pa", 30, "two", None), None).unwrap();
        db.insert_tool_result(&make_tr("tr_3", "s", "pb", 20, "three", None), None).unwrap();

        let all: Vec<String> = db.recent_tool_results(None, 10).unwrap().into_iter().map(|r| r.id).collect();
        assert_eq!(all, vec!["tr_2", "tr_3", "tr_1"]);

        let pa: Vec<String> = db.recent_tool_results(Some("pa"), 10).unwrap().into_iter().map(|r| r.id).collect();
        assert_eq!(pa, vec!["tr_2", "tr_1"]);

        assert_eq!(db.recent_tool_results(None, 1).unwrap().len(), 1);
    }

    // ── small helpers ──────────────────────────────────────────────────────

    #[test]
    fn first_n_chars_is_char_not_byte_based() {
        assert_eq!(first_n_chars("héllo wörld", 5), "héllo");
        assert_eq!(first_n_chars("abc", 10), "abc");
        assert_eq!(first_n_chars("", 3), "");
    }

    #[test]
    fn now_ts_is_positive() {
        assert!(now_ts() > 0);
    }
}
