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

/// Estimate token count from byte length (same heuristic as RTK).
pub fn estimate_tokens(byte_len: usize) -> u64 {
    (byte_len as f64 / 4.0).ceil() as u64
}
