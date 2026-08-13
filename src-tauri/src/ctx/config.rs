use std::path::PathBuf;

/// Default threshold (bytes) above which a tool output is virtualized.
pub const VIRTUALIZE_THRESHOLD_BYTES: usize = 2048;

/// How many leading lines of an oversized output are kept inline as the head
/// preview. Tail keeps the final `TAIL_LINES`.
pub const HEAD_LINES: usize = 20;
pub const TAIL_LINES: usize = 5;

/// Retrieval: how many prior turns + tool refs to pull on UserPromptSubmit.
pub const RETRIEVE_TOP_K: usize = 5;

/// BM25 candidate pool size before embedding rerank. Larger = better recall
/// at the cost of more decode+cosine work per prompt. 30 is plenty at our
/// scale — rerank is sub-millisecond for a few hundred rows.
pub const RETRIEVE_CANDIDATES: usize = 30;

/// Tools whose PostToolUse events we skip entirely — no storage, no
/// virtualization, no retrieval. Reasons per group:
///   - Task/Todo: internal scheduler chatter, near-zero retrieval value
///   - Plan/Worktree: transitional events without durable content
///   - Write/Edit/NotebookEdit/Read: outputs are file contents or
///     acknowledgements; we already have the source of truth on disk
///   - Agent: the envelope contains our own prompt text; until we recurse into
///     content blocks cleanly, storing creates more noise than signal
///   - ToolSearch/Monitor/ScheduleWakeup: metadata-only
pub const SKIP_TOOLS: &[&str] = &[
    "TodoWrite",
    "TaskCreate", "TaskUpdate", "TaskList", "TaskGet", "TaskStop", "TaskOutput",
    "ExitPlanMode", "EnterPlanMode", "EnterWorktree", "ExitWorktree",
    "Write", "Edit", "NotebookEdit", "Read",
    "ScheduleWakeup", "Monitor", "ToolSearch",
    "Agent",
];

pub fn is_skipped_tool(tool: &str) -> bool {
    SKIP_TOOLS.contains(&tool)
}

/// Root data directory. Mirrors `SavingsTracker::data_dir()`.
pub fn data_dir() -> PathBuf {
    dirs::home_dir()
        .expect("could not resolve home directory")
        .join(".glyphic")
}

pub fn db_path() -> PathBuf {
    data_dir().join("ctx.db")
}

pub fn bin_path() -> PathBuf {
    data_dir().join("bin").join("glyphic-ctx")
}

/// Environment variable that disables the engine at runtime (kill switch).
/// If set to "1", the hook prints an allow response and exits.
pub const KILL_SWITCH_ENV: &str = "GLYPHIC_CTX_DISABLED";

pub fn is_disabled() -> bool {
    std::env::var(KILL_SWITCH_ENV).map(|v| v == "1").unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants_are_coherent() {
        // Compile-time checks; the test just forces evaluation of this block.
        const {
            assert!(VIRTUALIZE_THRESHOLD_BYTES > 0);
            assert!(HEAD_LINES > 0);
            assert!(TAIL_LINES > 0);
            // Summary must be smaller than the payloads it stands in for.
            assert!(HEAD_LINES + TAIL_LINES < VIRTUALIZE_THRESHOLD_BYTES);
            // Rerank pool must be at least as large as what we return from it.
            assert!(RETRIEVE_TOP_K > 0);
            assert!(RETRIEVE_TOP_K <= RETRIEVE_CANDIDATES);
        }
    }

    #[test]
    fn skip_tools_membership_and_uniqueness() {
        for tool in ["Read", "Write", "Edit", "TodoWrite", "Agent", "Monitor"] {
            assert!(is_skipped_tool(tool), "{tool} should be skipped");
        }
        for tool in ["Bash", "Grep", "Glob", "WebFetch", "read", "BASH", ""] {
            assert!(!is_skipped_tool(tool), "{tool:?} should not be skipped");
        }
        // Matching is exact and case-sensitive; duplicates would hint at a
        // botched list edit.
        let mut seen = std::collections::HashSet::new();
        for tool in SKIP_TOOLS {
            assert!(seen.insert(*tool), "duplicate SKIP_TOOLS entry: {tool}");
        }
    }

    #[test]
    fn paths_derive_from_data_dir() {
        let root = data_dir();
        assert!(root.ends_with(".glyphic"), "data dir should be ~/.glyphic, got {root:?}");
        assert!(root.is_absolute());
        assert_eq!(db_path(), root.join("ctx.db"));
        assert_eq!(bin_path(), root.join("bin").join("glyphic-ctx"));
    }

    #[test]
    fn kill_switch_reads_env_var() {
        // Serial within this test; no other test in the crate reads the var,
        // so mutating process env here is safe.
        let prev = std::env::var(KILL_SWITCH_ENV).ok();

        std::env::remove_var(KILL_SWITCH_ENV);
        assert!(!is_disabled(), "unset must mean enabled");
        std::env::set_var(KILL_SWITCH_ENV, "1");
        assert!(is_disabled());
        std::env::set_var(KILL_SWITCH_ENV, "0");
        assert!(!is_disabled(), "only the literal \"1\" disables");
        std::env::set_var(KILL_SWITCH_ENV, "true");
        assert!(!is_disabled());

        match prev {
            Some(v) => std::env::set_var(KILL_SWITCH_ENV, v),
            None => std::env::remove_var(KILL_SWITCH_ENV),
        }
    }
}
