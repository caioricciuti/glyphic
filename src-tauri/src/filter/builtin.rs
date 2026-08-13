use crate::filter::pipeline::FilterDef;

/// Built-in filter definitions for the top 15 most common commands.
/// These are compiled into the binary — no file I/O needed at runtime.
pub static FILTERS: &[FilterDef] = &[
    // ── Git ─────────────────────────────────────────────────────────────
    FilterDef {
        name: "git_status",
        match_command: r"^git\s+status",
        strip_ansi: true,
        replace: &[],
        match_output: Some(r"(?i)nothing to commit,?\s*working\s+tree\s+clean"),
        unless: Some(r"(?i)fatal:|error:"),
        strip_lines: None,
        keep_lines: Some(
            r"(?i)^(On branch|Your branch|Changes|Untracked|modified:|new file:|deleted:|renamed:|\t|no changes)"
        ),
        truncate_lines_at: Some(120),
        head_lines: Some(30),
        tail_lines: None,
        max_lines: None,
        on_empty: Some("(clean working tree)"),
    },
    FilterDef {
        name: "git_log",
        match_command: r"^git\s+log",
        strip_ansi: true,
        replace: &[],
        match_output: None,
        unless: Some(r"(?i)fatal:|error:"),
        strip_lines: None,
        keep_lines: Some(r"^(commit |Author:|Date:|Merge:|\s{4}\S|$)"),
        truncate_lines_at: Some(120),
        head_lines: Some(60),
        tail_lines: None,
        max_lines: None,
        on_empty: Some("(no commits)"),
    },
    FilterDef {
        name: "git_diff",
        match_command: r"^git\s+diff",
        strip_ansi: true,
        replace: &[],
        match_output: None,
        unless: Some(r"(?i)fatal:|error:"),
        strip_lines: Some(r"^index [0-9a-f]"),
        keep_lines: None,
        truncate_lines_at: Some(200),
        head_lines: Some(100),
        tail_lines: Some(10),
        max_lines: None,
        on_empty: Some("(no diff)"),
    },
    FilterDef {
        name: "git_show",
        match_command: r"^git\s+show",
        strip_ansi: true,
        replace: &[],
        match_output: None,
        unless: Some(r"(?i)fatal:|error:"),
        strip_lines: Some(r"^index [0-9a-f]"),
        keep_lines: None,
        truncate_lines_at: Some(200),
        head_lines: Some(80),
        tail_lines: Some(5),
        max_lines: None,
        on_empty: None,
    },
    // ── File listing ────────────────────────────────────────────────────
    FilterDef {
        name: "ls",
        match_command: r"^(ls|ll|la)\b",
        strip_ansi: true,
        replace: &[],
        match_output: None,
        unless: None,
        strip_lines: None,
        keep_lines: None,
        truncate_lines_at: Some(120),
        head_lines: Some(40),
        tail_lines: Some(5),
        max_lines: None,
        on_empty: Some("(empty directory)"),
    },
    FilterDef {
        name: "tree",
        match_command: r"^tree\b",
        strip_ansi: true,
        replace: &[],
        match_output: None,
        unless: None,
        strip_lines: None,
        keep_lines: None,
        truncate_lines_at: Some(120),
        head_lines: Some(50),
        tail_lines: Some(3),
        max_lines: None,
        on_empty: Some("(empty tree)"),
    },
    FilterDef {
        name: "find",
        match_command: r"^find\b",
        strip_ansi: true,
        replace: &[],
        match_output: None,
        unless: Some(r"(?i)permission denied|error:"),
        strip_lines: None,
        keep_lines: None,
        truncate_lines_at: Some(150),
        head_lines: Some(30),
        tail_lines: None,
        max_lines: None,
        on_empty: Some("(no results)"),
    },
    // ── Search ──────────────────────────────────────────────────────────
    FilterDef {
        name: "grep",
        match_command: r"^(grep|rg|ripgrep)\b",
        strip_ansi: true,
        replace: &[],
        match_output: None,
        unless: None,
        strip_lines: None,
        keep_lines: None,
        truncate_lines_at: Some(200),
        head_lines: Some(30),
        tail_lines: None,
        max_lines: None,
        on_empty: Some("(no matches)"),
    },
    // ── JavaScript / Node ───────────────────────────────────────────────
    FilterDef {
        name: "npm_test",
        match_command: r"^(npm\s+test|bun\s+test|vitest|jest|npx\s+vitest|npx\s+jest)",
        strip_ansi: true,
        replace: &[
            // Strip timing noise
            (r"\s*\(\d+(\.\d+)?\s*(ms|s)\)", ""),
        ],
        match_output: None,
        unless: Some(r"(?i)FAIL|ERROR|error:|failed"),
        strip_lines: Some(r"^(\s*$|Downloading|Resolving|✓.*\d+\s*(ms|s))"),
        keep_lines: None,
        truncate_lines_at: Some(150),
        head_lines: Some(10),
        tail_lines: Some(20),
        max_lines: None,
        on_empty: Some("(all tests passed)"),
    },
    FilterDef {
        name: "npm_install",
        match_command: r"^(npm\s+install|npm\s+i\b|bun\s+install|bun\s+i\b|bun\s+add|npm\s+ci)",
        strip_ansi: true,
        replace: &[],
        match_output: None,
        unless: Some(r"(?i)ERR!|error|WARN|warn|deprecated"),
        strip_lines: Some(r"^(\s*$|npm\s+notice|Progress:|⠙|⠹|⠸|⠼|⠴|⠦|⠧|⠇|⠏)"),
        keep_lines: None,
        truncate_lines_at: Some(120),
        head_lines: None,
        tail_lines: Some(15),
        max_lines: Some(20),
        on_empty: Some("(install completed successfully)"),
    },
    // ── Rust / Cargo ────────────────────────────────────────────────────
    FilterDef {
        name: "cargo_test",
        match_command: r"^cargo\s+test",
        strip_ansi: true,
        replace: &[],
        match_output: None,
        // Case-sensitive on purpose: every successful run's summary contains
        // lowercase "0 failed;", which a (?i) pattern would match, making the
        // filter never apply. Cargo prints "FAILED"/"failures:" exactly.
        unless: Some(r"FAILED|failures:|error\["),
        strip_lines: Some(r"^(\s*Compiling\s|\s*Downloading\s|\s*Fresh\s|\s*Running\s)"),
        keep_lines: None,
        truncate_lines_at: Some(150),
        head_lines: Some(10),
        tail_lines: Some(20),
        max_lines: None,
        on_empty: Some("(all tests passed)"),
    },
    FilterDef {
        name: "cargo_build",
        match_command: r"^cargo\s+(build|check|clippy)",
        strip_ansi: true,
        replace: &[],
        match_output: None,
        unless: Some(r"(?i)error\[|warning:"),
        strip_lines: Some(r"^(\s*Compiling\s|\s*Downloading\s|\s*Fresh\s|\s*Updating\s|\s*Locking\s)"),
        keep_lines: None,
        truncate_lines_at: Some(200),
        head_lines: None,
        tail_lines: Some(30),
        max_lines: Some(50),
        on_empty: Some("(build succeeded)"),
    },
    // ── Network ─────────────────────────────────────────────────────────
    FilterDef {
        name: "curl",
        match_command: r"^curl\b",
        strip_ansi: true,
        replace: &[],
        match_output: None,
        unless: Some(r"(?i)curl: \(\d+\)|error|failed"),
        strip_lines: Some(r"^\s*(%\s*Total|Dload|Upload|Xferd|\d+\s+\d+)"),
        keep_lines: None,
        truncate_lines_at: Some(200),
        head_lines: Some(100),
        tail_lines: Some(5),
        max_lines: None,
        on_empty: None,
    },
    // ── Docker ──────────────────────────────────────────────────────────
    FilterDef {
        name: "docker_ps",
        match_command: r"^docker\s+(ps|container\s+ls)",
        strip_ansi: true,
        replace: &[],
        match_output: None,
        unless: None,
        strip_lines: None,
        keep_lines: None,
        truncate_lines_at: Some(150),
        head_lines: Some(21), // header + 20 containers
        tail_lines: None,
        max_lines: None,
        on_empty: Some("(no containers)"),
    },
    // ── Cat / read (large file guard) ───────────────────────────────────
    FilterDef {
        name: "cat",
        match_command: r"^(cat|bat|less|more)\b",
        strip_ansi: true,
        replace: &[],
        match_output: None,
        unless: Some(r"(?i)No such file|error:|permission denied"),
        strip_lines: None,
        keep_lines: None,
        truncate_lines_at: Some(200),
        head_lines: Some(80),
        tail_lines: Some(10),
        max_lines: None,
        on_empty: Some("(empty file)"),
    },
    // ── Python ──────────────────────────────────────────────────────────
    // NOTE: pytest must come BEFORE python_run — filter selection is
    // first-match, and `python3 -m pytest` matches both.
    FilterDef {
        name: "pytest",
        match_command: r"^(pytest|python3?\s+-m\s+pytest)",
        strip_ansi: true,
        replace: &[
            (r"\s*\[\s*\d+%\]", ""),
        ],
        match_output: None,
        unless: Some(r"(?i)FAILED|ERROR|ERRORS|failures"),
        strip_lines: Some(r"^(\s*$|platform |cachedir:|rootdir:|plugins:|collecting)"),
        keep_lines: None,
        truncate_lines_at: Some(150),
        head_lines: Some(10),
        tail_lines: Some(15),
        max_lines: None,
        on_empty: Some("(all tests passed)"),
    },
    FilterDef {
        name: "python_run",
        match_command: r"^(python3?|uv\s+run)\b",
        strip_ansi: true,
        replace: &[
            // Strip timing/progress noise
            (r"\s*\d+(\.\d+)?s\s*$", ""),
        ],
        match_output: None,
        unless: Some(r"(?i)Traceback|Error:|Exception:|FAILED|assert"),
        strip_lines: Some(r"^(\s*$|Downloading|Installing|Using |━+|  +━)"),
        keep_lines: None,
        truncate_lines_at: Some(200),
        head_lines: Some(50),
        tail_lines: Some(20),
        max_lines: None,
        on_empty: Some("(no output)"),
    },
    FilterDef {
        name: "pip_install",
        match_command: r"^(pip3?\s+install|uv\s+(pip\s+install|add|sync))",
        strip_ansi: true,
        replace: &[],
        match_output: None,
        unless: Some(r"(?i)ERROR:|error:|Could not|conflict"),
        strip_lines: Some(r"^(\s*$|Downloading|Using cached|━+|  +━|\s+Preparing)"),
        keep_lines: None,
        truncate_lines_at: Some(150),
        head_lines: None,
        tail_lines: Some(10),
        max_lines: Some(15),
        on_empty: Some("(install completed successfully)"),
    },
    // ── Go ──────────────────────────────────────────────────────────────
    FilterDef {
        name: "go_build",
        match_command: r"^go\s+(build|vet|install)",
        strip_ansi: true,
        replace: &[],
        match_output: Some(r"^\s*$"),
        unless: Some(r"(?i)error:|cannot |undefined:|imported and not used"),
        strip_lines: None,
        keep_lines: None,
        truncate_lines_at: Some(200),
        head_lines: Some(30),
        tail_lines: Some(10),
        max_lines: Some(50),
        on_empty: Some("(build succeeded)"),
    },
    FilterDef {
        name: "go_test",
        match_command: r"^go\s+test",
        strip_ansi: true,
        replace: &[],
        match_output: None,
        unless: Some(r"(?i)FAIL\s|--- FAIL|panic:"),
        strip_lines: Some(r"^=== RUN\s"),
        keep_lines: None,
        truncate_lines_at: Some(150),
        head_lines: Some(20),
        tail_lines: Some(20),
        max_lines: None,
        on_empty: Some("(all tests passed)"),
    },
    // ── Bun / Make / general build ──────────────────────────────────────
    FilterDef {
        name: "bun_run",
        match_command: r"^bun\s+run\b",
        strip_ansi: true,
        replace: &[],
        match_output: None,
        unless: Some(r"(?i)error|Error:|ERR!|FAIL|failed|warning:"),
        strip_lines: Some(r"^(\s*$|\$\s)"),
        keep_lines: None,
        truncate_lines_at: Some(200),
        head_lines: Some(40),
        tail_lines: Some(20),
        max_lines: None,
        on_empty: Some("(completed successfully)"),
    },
    FilterDef {
        name: "make",
        match_command: r"^make\b",
        strip_ansi: true,
        replace: &[],
        match_output: None,
        unless: Some(r"(?i)Error \d+|error:|Stop\.|make\[.+Error"),
        strip_lines: Some(r"^make\[\d+\]: (Entering|Leaving) directory"),
        keep_lines: None,
        truncate_lines_at: Some(200),
        head_lines: Some(30),
        tail_lines: Some(20),
        max_lines: Some(60),
        on_empty: Some("(build succeeded)"),
    },
    // ── Kubernetes ──────────────────────────────────────────────────────
    FilterDef {
        name: "kubectl_get",
        match_command: r"^kubectl\s+(get|describe|logs)",
        strip_ansi: true,
        replace: &[],
        match_output: None,
        unless: Some(r"(?i)error|forbidden|not found|couldn't"),
        strip_lines: None,
        keep_lines: None,
        truncate_lines_at: Some(200),
        head_lines: Some(60),
        tail_lines: Some(10),
        max_lines: None,
        on_empty: Some("(no resources found)"),
    },
    // ── Docker build/compose ────────────────────────────────────────────
    FilterDef {
        name: "docker_build",
        match_command: r"^docker\s+(build|compose)",
        strip_ansi: true,
        replace: &[],
        match_output: None,
        unless: Some(r"(?i)error|ERROR|failed to"),
        strip_lines: Some(r"^(\s*$|#\d+\s+(CACHED|DONE)|Sending build context|Step \d+/\d+ : (FROM|WORKDIR|COPY|RUN|ENV|EXPOSE|CMD|ENTRYPOINT))"),
        keep_lines: None,
        truncate_lines_at: Some(200),
        head_lines: Some(20),
        tail_lines: Some(15),
        max_lines: Some(40),
        on_empty: Some("(build succeeded)"),
    },
    // ── Wc / du / df (small output, just truncate long lines) ───────────
    FilterDef {
        name: "disk_stats",
        match_command: r"^(wc|du|df)\b",
        strip_ansi: true,
        replace: &[],
        match_output: None,
        unless: None,
        strip_lines: None,
        keep_lines: None,
        truncate_lines_at: Some(120),
        head_lines: Some(30),
        tail_lines: Some(5),
        max_lines: None,
        on_empty: None,
    },
];

#[cfg(test)]
mod tests {
    use super::FILTERS;
    use crate::filter::pipeline::FilterPipeline;
    use regex::Regex;
    use std::collections::HashSet;

    /// Mirrors find_filter's first-match selection over the builtin table.
    fn first_match(cmd: &str) -> Option<&'static str> {
        FILTERS.iter().find(|f| f.matches(cmd)).map(|f| f.name)
    }

    /// Apply the first matching builtin filter to output.
    fn apply(cmd: &str, output: &str) -> String {
        let def = FILTERS
            .iter()
            .find(|f| f.matches(cmd))
            .unwrap_or_else(|| panic!("no builtin filter matches: {cmd}"));
        FilterPipeline::from_def(def).apply(output)
    }

    // ── Table sanity ────────────────────────────────────────────────────

    #[test]
    fn all_patterns_compile() {
        for def in FILTERS {
            assert!(
                Regex::new(def.match_command).is_ok(),
                "{}: bad match_command",
                def.name
            );
            for (pat, _) in def.replace {
                assert!(Regex::new(pat).is_ok(), "{}: bad replace pattern", def.name);
            }
            for (label, pat) in [
                ("match_output", def.match_output),
                ("unless", def.unless),
                ("strip_lines", def.strip_lines),
                ("keep_lines", def.keep_lines),
            ] {
                if let Some(p) = pat {
                    assert!(Regex::new(p).is_ok(), "{}: bad {label} pattern", def.name);
                }
            }
        }
    }

    #[test]
    fn filter_names_are_unique() {
        let mut seen = HashSet::new();
        for def in FILTERS {
            assert!(seen.insert(def.name), "duplicate filter name: {}", def.name);
        }
    }

    // ── Command matching / precedence ───────────────────────────────────

    #[test]
    fn each_filter_is_reachable_by_a_canonical_command() {
        let cases = [
            ("git status", "git_status"),
            ("git log --oneline -20", "git_log"),
            ("git diff HEAD~1", "git_diff"),
            ("git show abc1234", "git_show"),
            ("ls -la", "ls"),
            ("ll", "ls"),
            ("la", "ls"),
            ("tree -L 2", "tree"),
            ("find . -name '*.rs'", "find"),
            ("grep -rn TODO src/", "grep"),
            ("rg 'fn main'", "grep"),
            ("npm test", "npm_test"),
            ("bun test", "npm_test"),
            ("npx vitest run", "npm_test"),
            ("npm install", "npm_install"),
            ("npm ci", "npm_install"),
            ("bun add react", "npm_install"),
            ("cargo test --lib", "cargo_test"),
            ("cargo build --release", "cargo_build"),
            ("cargo clippy --all-targets", "cargo_build"),
            ("curl -s https://example.com", "curl"),
            ("docker ps -a", "docker_ps"),
            ("docker container ls", "docker_ps"),
            ("cat Cargo.toml", "cat"),
            ("pytest -q", "pytest"),
            ("python3 -m pytest tests/", "pytest"),
            ("python -m pytest", "pytest"),
            ("python3 script.py", "python_run"),
            ("uv run main.py", "python_run"),
            ("pip install requests", "pip_install"),
            ("pip3 install requests", "pip_install"),
            ("uv add polars", "pip_install"),
            ("uv sync", "pip_install"),
            ("uv pip install numpy", "pip_install"),
            ("go build ./...", "go_build"),
            ("go vet ./...", "go_build"),
            ("go test ./...", "go_test"),
            ("bun run dev", "bun_run"),
            ("make -j4", "make"),
            ("kubectl get pods", "kubectl_get"),
            ("kubectl logs my-pod", "kubectl_get"),
            ("docker build -t app .", "docker_build"),
            ("docker compose up -d", "docker_build"),
            ("wc -l src/main.rs", "disk_stats"),
            ("du -sh .", "disk_stats"),
            ("df -h", "disk_stats"),
        ];
        for (cmd, expected) in cases {
            assert_eq!(
                first_match(cmd),
                Some(expected),
                "command {cmd:?} matched wrong filter"
            );
        }
    }

    #[test]
    fn unrelated_commands_do_not_match() {
        for cmd in [
            "echo hi",
            "gitk",
            "lsof -i :8080",
            "curling",
            "made-up-tool",
            "gofmt -w .",
            "catalog list",
            "pipx run black",
            "treeview",
            "finder",
            "wcgrep",
        ] {
            assert_eq!(first_match(cmd), None, "command {cmd:?} unexpectedly matched");
        }
    }

    #[test]
    fn pytest_wins_over_python_run_for_module_invocation() {
        // Regression: first-match selection means pytest must precede
        // python_run in FILTERS, or `python3 -m pytest` gets the wrong filter.
        assert_eq!(first_match("python3 -m pytest -x"), Some("pytest"));
        assert_eq!(first_match("python3 app.py"), Some("python_run"));
    }

    // ── git ─────────────────────────────────────────────────────────────

    #[test]
    fn git_status_clean_collapses_to_marker() {
        let out = apply(
            "git status",
            "On branch main\nYour branch is up to date with 'origin/main'.\n\nnothing to commit, working tree clean\n",
        );
        assert_eq!(out, "(clean working tree)");
    }

    #[test]
    fn git_status_dirty_keeps_essentials_drops_hints() {
        let input = "On branch main\n\
Your branch is up to date with 'origin/main'.\n\
\n\
Changes not staged for commit:\n\
  (use \"git add <file>...\" to update what will be committed)\n\
  (use \"git restore <file>...\" to discard changes in working directory)\n\
\tmodified:   src/main.rs\n\
\n\
Untracked files:\n\
  (use \"git add <file>...\" to include in what will be committed)\n\
\tnew_file.txt\n\
\n\
no changes added to commit (use \"git add\" and/or \"git commit -a\")\n";
        let out = apply("git status", input);
        assert!(out.contains("On branch main"));
        assert!(out.contains("modified:   src/main.rs"));
        assert!(out.contains("new_file.txt"));
        assert!(out.contains("Untracked files:"));
        // Hint lines like `  (use "git add <file>..." ...)` are dropped; the
        // kept "no changes added" line has no "<file>" placeholder.
        assert!(!out.contains("<file>"), "hint lines should be dropped: {out}");
    }

    #[test]
    fn git_status_fatal_error_fully_preserved() {
        let input = "fatal: not a git repository (or any of the parent directories): .git";
        assert_eq!(apply("git status", input), input);
    }

    #[test]
    fn git_log_keeps_commit_metadata_and_message() {
        let input = "commit 0a1b2c3d4e5f\nAuthor: Caio <c@example.com>\nDate:   Mon Aug 11 10:00:00 2026 +0200\n\n    fix(filter): keep error lines\n";
        let out = apply("git log", input);
        assert!(out.contains("commit 0a1b2c3d4e5f"));
        assert!(out.contains("Author: Caio"));
        assert!(out.contains("    fix(filter): keep error lines"));
    }

    #[test]
    fn git_diff_strips_index_lines_keeps_hunks() {
        let input = "diff --git a/src/main.rs b/src/main.rs\nindex 0a1b2c3..4d5e6f7 100644\n--- a/src/main.rs\n+++ b/src/main.rs\n@@ -1,3 +1,4 @@\n fn main() {\n+    println!(\"hi\");\n }\n";
        let out = apply("git diff", input);
        assert!(!out.contains("index 0a1b2c3"));
        assert!(out.contains("@@ -1,3 +1,4 @@"));
        assert!(out.contains("+    println!(\"hi\");"));
    }

    #[test]
    fn git_diff_empty_reports_no_diff() {
        assert_eq!(apply("git diff", ""), "(no diff)");
    }

    // ── cargo ───────────────────────────────────────────────────────────

    #[test]
    fn cargo_test_success_keeps_counts_drops_build_noise() {
        let input = "   Compiling glyphic v0.22.0\n    Finished `test` profile [unoptimized]\n     Running unittests src/lib.rs\n\nrunning 3 tests\ntest filter::a ... ok\ntest filter::b ... ok\ntest filter::c ... ok\n\ntest result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s\n";
        let out = apply("cargo test", input);
        assert!(out.contains("running 3 tests"));
        assert!(
            out.contains("test result: ok. 3 passed; 0 failed"),
            "summary counts must survive: {out}"
        );
        assert!(!out.contains("Compiling"));
        assert!(!out.contains("Running unittests"));
    }

    #[test]
    fn cargo_test_failure_preserves_assertion_detail() {
        let input = "running 2 tests\ntest a ... ok\ntest b ... FAILED\n\nfailures:\n\n---- b stdout ----\nassertion `left == right` failed\n  left: 1\n right: 2\n\nfailures:\n    b\n\ntest result: FAILED. 1 passed; 1 failed; 0 ignored\n";
        let out = apply("cargo test", input);
        assert!(out.contains("assertion `left == right` failed"));
        assert!(out.contains(" left: 1"));
        assert!(out.contains("test result: FAILED. 1 passed; 1 failed"));
    }

    #[test]
    fn cargo_build_error_preserved_clean_build_summarized() {
        let err = "error[E0308]: mismatched types\n --> src/main.rs:2:5\n";
        let out = apply("cargo build", err);
        assert!(out.contains("error[E0308]: mismatched types"));
        assert!(out.contains("--> src/main.rs:2:5"));

        assert_eq!(apply("cargo build", ""), "(build succeeded)");
    }

    // ── node / npm / bun ────────────────────────────────────────────────

    #[test]
    fn npm_test_strips_timing_keeps_test_names_and_summary() {
        let input = "✓ renders header (12 ms)\n✓ handles click (5 ms)\n\nTest Files  2 passed (2)\n     Tests  8 passed (8)\n";
        let out = apply("npm test", input);
        assert!(out.contains("Tests  8 passed (8)"), "counts must survive: {out}");
        assert!(!out.contains("(12 ms)"), "timing noise should be stripped: {out}");
        assert!(out.contains("✓ renders header"));
    }

    #[test]
    fn npm_test_failure_preserved_via_unless() {
        let input = "FAIL src/app.test.ts\n  ✕ renders header (30 ms)\n    Expected: 1\n    Received: 2\n";
        let out = apply("npm test", input);
        assert!(out.contains("FAIL src/app.test.ts"));
        assert!(out.contains("Expected: 1"));
        assert!(out.contains("Received: 2"));
        // replace runs before the unless check, so timing noise is stripped
        // even on the error-preserving path
        assert!(!out.contains("(30 ms)"));
    }

    #[test]
    fn npm_install_keeps_package_count_summary() {
        let input = "npm notice created a lockfile\n\nadded 120 packages, and audited 121 packages in 3s\n\nfound 0 vulnerabilities\n";
        let out = apply("npm install", input);
        assert!(out.contains("added 120 packages"), "count must survive: {out}");
        assert!(!out.contains("npm notice"));
    }

    #[test]
    fn npm_install_error_preserved() {
        let input = "npm ERR! code ERESOLVE\nnpm ERR! Could not resolve dependency\n";
        let out = apply("npm install", input);
        assert!(out.contains("ERESOLVE"));
        assert!(out.contains("Could not resolve dependency"));
    }

    #[test]
    fn bun_run_strips_command_echo_keeps_output() {
        let out = apply("bun run build", "$ vite build\nbuilt in 420ms\n");
        assert!(!out.contains("$ vite build"));
        assert!(out.contains("built in 420ms"));
    }

    // ── python ──────────────────────────────────────────────────────────

    #[test]
    fn pytest_success_strips_header_and_percent_markers() {
        let input = "platform darwin -- Python 3.12.0, pytest-8.0.0\nrootdir: /Users/caio/proj\nplugins: cov-4.1.0\ncollecting ... collected 3 items\n\ntests/test_app.py ...                                    [100%]\n\n============ 3 passed in 0.12s ============\n";
        let out = apply("pytest", input);
        assert!(out.contains("3 passed in 0.12s"), "counts must survive: {out}");
        assert!(!out.contains("[100%]"));
        assert!(!out.contains("platform darwin"));
        assert!(!out.contains("rootdir:"));
    }

    #[test]
    fn pytest_failure_preserved() {
        let input = "tests/test_app.py F\n\nFAILED tests/test_app.py::test_x - AssertionError: assert 1 == 2\n============ 1 failed in 0.10s ============\n";
        let out = apply("pytest", input);
        assert!(out.contains("AssertionError: assert 1 == 2"));
        assert!(out.contains("1 failed in 0.10s"));
    }

    #[test]
    fn python_traceback_fully_preserved() {
        let input = "Traceback (most recent call last):\n  File \"app.py\", line 3, in <module>\n    main()\nValueError: bad value\n";
        let out = apply("python3 app.py", input);
        assert!(out.contains("Traceback (most recent call last):"));
        assert!(out.contains("ValueError: bad value"));
        assert!(out.contains("File \"app.py\", line 3"));
    }

    #[test]
    fn pip_install_keeps_success_summary_and_errors() {
        let ok = "Collecting requests\nDownloading requests-2.31.0-py3-none-any.whl\nInstalling collected packages: requests\nSuccessfully installed requests-2.31.0\n";
        let out = apply("pip install requests", ok);
        assert!(out.contains("Successfully installed requests-2.31.0"));
        assert!(!out.contains("Downloading"));

        let err = "ERROR: Could not find a version that satisfies the requirement nope\n";
        assert!(apply("pip install nope", err).contains("Could not find a version"));
    }

    // ── go ──────────────────────────────────────────────────────────────

    #[test]
    fn go_build_silent_success_and_loud_failure() {
        assert_eq!(apply("go build ./...", ""), "(build succeeded)");
        let err = "./main.go:10:2: undefined: helper\n";
        assert!(apply("go build ./...", err).contains("undefined: helper"));
    }

    #[test]
    fn go_test_strips_run_lines_keeps_results() {
        let input = "=== RUN   TestFoo\n--- PASS: TestFoo (0.00s)\n=== RUN   TestBar\n--- PASS: TestBar (0.00s)\nPASS\nok  \texample.com/pkg\t0.512s\n";
        let out = apply("go test ./...", input);
        assert!(!out.contains("=== RUN"));
        assert!(out.contains("ok  \texample.com/pkg"));
    }

    #[test]
    fn go_test_failure_preserved_including_run_lines() {
        let input = "=== RUN   TestBar\n--- FAIL: TestBar (0.01s)\n    bar_test.go:12: got 1, want 2\nFAIL\nFAIL\texample.com/pkg\t0.020s\n";
        let out = apply("go test ./...", input);
        assert!(out.contains("--- FAIL: TestBar"));
        assert!(out.contains("bar_test.go:12: got 1, want 2"));
    }

    // ── misc commands ───────────────────────────────────────────────────

    #[test]
    fn make_error_preserved() {
        let input = "cc -o app main.c\nmain.c:5:1: error: expected ';' before '}'\nmake: *** [Makefile:3: app] Error 1\n";
        let out = apply("make", input);
        assert!(out.contains("error: expected ';'"));
        assert!(out.contains("Error 1"), "exit status line must survive: {out}");
    }

    #[test]
    fn curl_strips_progress_meter_keeps_body() {
        let input = "  % Total    % Received % Xferd  Average Speed\n100  1256  100  1256    0     0   9k      0\n{\"status\":\"ok\",\"count\":42}\n";
        let out = apply("curl -s https://api.example.com", input);
        assert!(out.contains("{\"status\":\"ok\",\"count\":42}"));
        assert!(!out.contains("% Total"));
    }

    #[test]
    fn curl_failure_preserved() {
        let input = "curl: (7) Failed to connect to localhost port 8080 after 3 ms: Connection refused\n";
        let out = apply("curl http://localhost:8080", input);
        assert!(out.contains("curl: (7) Failed to connect"));
    }

    #[test]
    fn kubectl_error_preserved() {
        let input = "Error from server (NotFound): pods \"web-1\" not found\n";
        assert!(apply("kubectl get pods", input).contains("Error from server (NotFound)"));
    }

    #[test]
    fn docker_ps_empty_and_docker_build_noise() {
        assert_eq!(apply("docker ps", ""), "(no containers)");
        let build = "#1 [internal] load build definition\n#2 CACHED\nSuccessfully built 0a1b2c3d\n";
        let out = apply("docker build -t app .", build);
        assert!(out.contains("Successfully built 0a1b2c3d"));
        assert!(!out.contains("#2 CACHED"));
    }

    #[test]
    fn cat_missing_file_error_preserved() {
        let input = "cat: nope.txt: No such file or directory";
        assert_eq!(apply("cat nope.txt", input), input);
    }

    #[test]
    fn grep_no_matches_and_head_truncation() {
        assert_eq!(apply("grep -r TODO src/", ""), "(no matches)");
        let many: Vec<String> = (1..=50).map(|i| format!("src/f{i}.rs:1:match")).collect();
        let out = apply("rg match", &many.join("\n"));
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines.len(), 31);
        assert_eq!(lines[30], "... (20 more lines)");
        assert_eq!(lines[0], "src/f1.rs:1:match");
    }

    #[test]
    fn ls_head_tail_omitted_count_is_accurate() {
        let entries: Vec<String> = (1..=60).map(|i| format!("file{i:03}.txt")).collect();
        let out = apply("ls", &entries.join("\n"));
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines.len(), 46); // 40 head + marker + 5 tail
        assert_eq!(lines[40], "... (15 lines omitted)");
        assert_eq!(lines[45], "file060.txt");
        assert_eq!(apply("ls", ""), "(empty directory)");
    }

    #[test]
    fn disk_stats_counts_survive() {
        let input = "     142 src/main.rs\n      58 src/lib.rs\n     200 total";
        let out = apply("wc -l src/*.rs", input);
        assert!(out.contains("142 src/main.rs"));
        assert!(out.contains("200 total"));
    }

    #[test]
    fn multibyte_output_through_builtin_filter_no_panic() {
        let long = "变量名称非常长的一行内容".repeat(30);
        let out = apply("cat notes.txt", &long);
        assert!(out.ends_with("..."));
        assert_eq!(out.chars().count(), 200); // cat truncate_lines_at
    }

    #[test]
    fn crlf_output_through_builtin_filter() {
        let out = apply("git status", "fatal: bad revision\r\nerror: details here\r\n");
        assert_eq!(out, "fatal: bad revision\nerror: details here");
    }
}
