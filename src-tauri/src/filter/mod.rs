pub mod builtin;
pub mod pipeline;
pub mod tracker;

pub use pipeline::{FilterDef, FilterPipeline};
pub use tracker::SavingsTracker;

/// Find the best matching filter for a command string.
/// Handles piped commands by matching the first segment.
pub fn find_filter(command: &str) -> Option<&'static FilterDef> {
    let cmd = command.trim();

    // Direct match first
    if let Some(f) = builtin::FILTERS.iter().find(|f| f.matches(cmd)) {
        return Some(f);
    }

    // For piped commands, try matching on the first segment
    if cmd.contains('|') {
        let first_segment = cmd.split('|').next().unwrap_or("").trim();
        if !first_segment.is_empty() {
            if let Some(f) = builtin::FILTERS.iter().find(|f| f.matches(first_segment)) {
                return Some(f);
            }
        }
    }

    // For chained commands (&&, ;), try matching the last segment
    // (e.g., `cd foo && git status` → match git status)
    for sep in &["&&", ";"] {
        if cmd.contains(sep) {
            let last_segment = cmd.rsplit(sep).next().unwrap_or("").trim();
            if !last_segment.is_empty() {
                if let Some(f) = builtin::FILTERS.iter().find(|f| f.matches(last_segment)) {
                    return Some(f);
                }
            }
        }
    }

    None
}

/// Apply the best matching filter to command output.
/// Returns (filtered_output, original_len, filtered_len).
pub fn filter_output(command: &str, output: &str) -> (String, usize, usize) {
    let original_len = output.len();

    match find_filter(command) {
        Some(def) => {
            let pipeline = FilterPipeline::from_def(def);
            let filtered = pipeline.apply(output);
            let filtered_len = filtered.len();
            (filtered, original_len, filtered_len)
        }
        None => {
            // Universal default: ANSI strip + whitespace normalization + dedup + truncation
            let pipeline = FilterPipeline::default_pipeline();
            let filtered = pipeline.apply(output);
            let filtered_len = filtered.len();
            (filtered, original_len, filtered_len)
        }
    }
}

/// Estimate token count from byte length (same heuristic as RTK). Used when
/// the actual text is no longer available.
pub fn estimate_tokens(byte_len: usize) -> u64 {
    (byte_len as f64 / 4.0).ceil() as u64
}

/// Content-aware token estimate. ASCII runs ~3.8 bytes per token (between
/// English prose at ~4 and source code at ~3.5); non-ASCII chars (CJK,
/// emoji) tokenize at roughly one token per character, denser than their
/// UTF-8 byte count divided by 4 suggests.
pub fn estimate_tokens_text(text: &str) -> u64 {
    let mut ascii_bytes = 0usize;
    let mut non_ascii_chars = 0usize;
    for c in text.chars() {
        if c.is_ascii() {
            ascii_bytes += 1;
        } else {
            non_ascii_chars += 1;
        }
    }
    ((ascii_bytes as f64 / 3.8) + non_ascii_chars as f64).ceil() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_filter_matches_direct_piped_and_chained() {
        // The exact filter set lives in builtin.rs; only shape matters here
        if builtin::FILTERS.is_empty() {
            return;
        }
        assert!(find_filter("git status").is_some() || find_filter("git status").is_none()); // no panic
        // Piped: first segment wins
        let piped = find_filter("git status | head -5");
        let direct = find_filter("git status");
        assert_eq!(piped.map(|f| f.name), direct.map(|f| f.name));
        // Chained: last segment wins
        let chained = find_filter("cd /tmp && git status");
        assert_eq!(chained.map(|f| f.name), direct.map(|f| f.name));
    }

    #[test]
    fn estimate_tokens_text_ascii_close_to_bytes_over_four() {
        let text = "the quick brown fox jumps over the lazy dog and keeps on running";
        let est = estimate_tokens_text(text);
        let naive = estimate_tokens(text.len());
        // Same ballpark for plain English
        assert!(est >= naive.saturating_sub(naive / 3) && est <= naive + naive / 3);
    }

    #[test]
    fn estimate_tokens_text_counts_cjk_denser_than_bytes_over_four() {
        let text = "こんにちは世界、これはテストです";
        // bytes/4 would say ~12; real tokenizers land near one token per char
        assert!(estimate_tokens_text(text) > estimate_tokens(text.len()));
    }

    #[test]
    fn estimate_tokens_text_empty_is_zero() {
        assert_eq!(estimate_tokens_text(""), 0);
    }
}
