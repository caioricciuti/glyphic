//! Lazy, process-wide embedding model. The first call triggers an ONNX model
//! download (~130 MB, one-time per machine) and load; subsequent calls reuse
//! the same instance. On any failure we return `None` so retrieval gracefully
//! degrades to BM25-only — embeddings are a rerank booster, never load-bearing.
//!
//! Chosen model: `BGESmallENV15` (384-dim, top of MTEB small-model leaderboard,
//! fast on CPU). Normalised L2 output — cosine == dot product.
//!
//! Cache dir: `~/.glyphic/models/`. Survives app upgrades.

use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};

use super::config::data_dir;

pub const EMBED_DIM: usize = 384;

/// One model instance shared across threads. TextEmbedding holds an ONNX
/// session which is not cheap to recreate.
static MODEL: OnceLock<Mutex<Option<TextEmbedding>>> = OnceLock::new();

fn models_dir() -> PathBuf {
    data_dir().join("models")
}

fn try_init() -> Option<TextEmbedding> {
    let cache = models_dir();
    let _ = std::fs::create_dir_all(&cache);
    TextEmbedding::try_new(
        InitOptions::new(EmbeddingModel::BGESmallENV15)
            .with_cache_dir(cache)
            .with_show_download_progress(false),
    )
    .ok()
}

fn with_model<R>(f: impl FnOnce(&TextEmbedding) -> R) -> Option<R> {
    let slot = MODEL.get_or_init(|| Mutex::new(try_init()));
    let mut guard = slot.lock().ok()?;
    if guard.is_none() {
        *guard = try_init();
    }
    guard.as_ref().map(f)
}

/// Returns `true` once the model has successfully loaded at least once.
pub fn is_ready() -> bool {
    MODEL
        .get()
        .and_then(|m| m.lock().ok().map(|g| g.is_some()))
        .unwrap_or(false)
}

/// Embed a single text. Returns `None` on any error (model not available,
/// encode failure). Caller should treat as "skip semantic rerank for this row".
pub fn embed_one(text: &str) -> Option<Vec<f32>> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }
    with_model(|m| m.embed(vec![trimmed], None).ok())
        .flatten()
        .and_then(|mut v| v.pop())
}

/// Batch embed. Preserves input order. Returns `None` on failure — we prefer
/// all-or-nothing over partial batches to keep callers simple.
pub fn embed_batch(texts: &[&str]) -> Option<Vec<Vec<f32>>> {
    if texts.is_empty() {
        return Some(Vec::new());
    }
    let owned: Vec<&str> = texts.iter().map(|s| s.trim()).collect();
    with_model(|m| m.embed(owned, None).ok()).flatten()
}

/// Pack a `&[f32]` as little-endian bytes for SQLite BLOB storage.
pub fn encode(v: &[f32]) -> Vec<u8> {
    let mut out = Vec::with_capacity(v.len() * 4);
    for x in v {
        out.extend_from_slice(&x.to_le_bytes());
    }
    out
}

/// Inverse of `encode`. Returns `None` if length is not a multiple of 4.
pub fn decode(bytes: &[u8]) -> Option<Vec<f32>> {
    if !bytes.len().is_multiple_of(4) {
        return None;
    }
    let mut out = Vec::with_capacity(bytes.len() / 4);
    for chunk in bytes.chunks_exact(4) {
        out.push(f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]));
    }
    Some(out)
}

/// Cosine similarity. BGE outputs are already L2-normalised so this is a
/// plain dot product, but we normalise defensively — cost is trivial and it
/// keeps us correct if the model ever changes.
pub fn cosine(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let mut dot = 0.0f32;
    let mut na = 0.0f32;
    let mut nb = 0.0f32;
    for i in 0..a.len() {
        dot += a[i] * b[i];
        na += a[i] * a[i];
        nb += b[i] * b[i];
    }
    let denom = (na.sqrt() * nb.sqrt()).max(f32::EPSILON);
    dot / denom
}
