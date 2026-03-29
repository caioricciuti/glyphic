pub mod builtin;
pub mod pipeline;
pub mod tracker;

pub use pipeline::{FilterDef, FilterPipeline};
pub use tracker::SavingsTracker;

/// Find the best matching filter for a command string.
pub fn find_filter(command: &str) -> Option<&'static FilterDef> {
    builtin::FILTERS.iter().find(|f| f.matches(command))
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
            // Passthrough — still strip ANSI as a baseline
            let cleaned = pipeline::strip_ansi(output);
            let filtered_len = cleaned.len();
            (cleaned, original_len, filtered_len)
        }
    }
}

/// Estimate token count from byte length (same heuristic as RTK).
pub fn estimate_tokens(byte_len: usize) -> u64 {
    (byte_len as f64 / 4.0).ceil() as u64
}
