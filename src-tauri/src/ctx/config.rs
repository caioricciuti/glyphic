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
