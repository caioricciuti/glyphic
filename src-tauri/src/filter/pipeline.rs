use regex::Regex;

/// A single filter definition, typically parsed from TOML or defined as a built-in.
pub struct FilterDef {
    pub name: &'static str,
    pub match_command: &'static str,
    pub strip_ansi: bool,
    pub replace: &'static [(&'static str, &'static str)],
    pub match_output: Option<&'static str>,
    pub unless: Option<&'static str>,
    pub strip_lines: Option<&'static str>,
    pub keep_lines: Option<&'static str>,
    pub truncate_lines_at: Option<usize>,
    pub head_lines: Option<usize>,
    pub tail_lines: Option<usize>,
    pub max_lines: Option<usize>,
    pub on_empty: Option<&'static str>,
}

impl FilterDef {
    /// Check if this filter matches a given command string.
    pub fn matches(&self, command: &str) -> bool {
        Regex::new(self.match_command)
            .map(|re| re.is_match(command.trim()))
            .unwrap_or(false)
    }
}

/// Compiled pipeline ready to apply to text.
pub struct FilterPipeline {
    strip_ansi: bool,
    replace: Vec<(Regex, String)>,
    match_output: Option<Regex>,
    unless: Option<Regex>,
    strip_lines: Option<Regex>,
    keep_lines: Option<Regex>,
    dedup_consecutive: bool,
    normalize_whitespace: bool,
    truncate_lines_at: Option<usize>,
    head_lines: Option<usize>,
    tail_lines: Option<usize>,
    max_lines: Option<usize>,
    on_empty: Option<String>,
}

impl FilterPipeline {
    /// Compile a FilterDef into an executable pipeline.
    pub fn from_def(def: &FilterDef) -> Self {
        Self {
            strip_ansi: def.strip_ansi,
            replace: def
                .replace
                .iter()
                .filter_map(|(pat, rep)| Regex::new(pat).ok().map(|re| (re, rep.to_string())))
                .collect(),
            match_output: def.match_output.and_then(|p| Regex::new(p).ok()),
            unless: def.unless.and_then(|p| Regex::new(p).ok()),
            strip_lines: def.strip_lines.and_then(|p| Regex::new(p).ok()),
            keep_lines: def.keep_lines.and_then(|p| Regex::new(p).ok()),
            dedup_consecutive: true,
            normalize_whitespace: true,
            truncate_lines_at: def.truncate_lines_at,
            head_lines: def.head_lines,
            tail_lines: def.tail_lines,
            max_lines: def.max_lines,
            on_empty: def.on_empty.map(|s| s.to_string()),
        }
    }

    /// Universal default pipeline for commands without a specific filter.
    /// Strips ANSI, normalizes whitespace, deduplicates, and applies generous limits.
    pub fn default_pipeline() -> Self {
        Self {
            strip_ansi: true,
            replace: Vec::new(),
            match_output: None,
            unless: None,
            strip_lines: None,
            keep_lines: None,
            dedup_consecutive: true,
            normalize_whitespace: true,
            truncate_lines_at: Some(300),
            head_lines: Some(150),
            tail_lines: Some(20),
            max_lines: None,
            on_empty: None,
        }
    }

    /// Apply the full pipeline to the input text.
    pub fn apply(&self, input: &str) -> String {
        // Stage 1: Strip ANSI escape codes
        let text = if self.strip_ansi {
            strip_ansi(input)
        } else {
            input.to_string()
        };

        // Stage 2: Regex replacements (line-by-line)
        let text = if self.replace.is_empty() {
            text
        } else {
            text.lines()
                .map(|line| {
                    let mut l = line.to_string();
                    for (re, rep) in &self.replace {
                        l = re.replace_all(&l, rep.as_str()).to_string();
                    }
                    l
                })
                .collect::<Vec<_>>()
                .join("\n")
        };

        // Stage 3: unless — skip filtering if output contains this pattern (preserve errors).
        // Checked before match_output so an error is never swallowed by a
        // success-pattern short-circuit.
        if let Some(ref re) = self.unless {
            if re.is_match(&text) {
                // Still apply ANSI strip + whitespace normalization on error output
                return if self.normalize_whitespace {
                    normalize_whitespace(&text)
                } else {
                    text
                };
            }
        }

        // Stage 4: match_output — short-circuit if full output matches
        if let Some(ref re) = self.match_output {
            if re.is_match(&text) {
                return self.on_empty.clone().unwrap_or_default();
            }
        }

        // Stage 5: Whitespace normalization (collapse blank line runs, trim trailing)
        let text = if self.normalize_whitespace {
            normalize_whitespace(&text)
        } else {
            text
        };

        // Stage 6: strip_lines / keep_lines
        let lines: Vec<&str> = text.lines().collect();
        let lines = if let Some(ref re) = self.keep_lines {
            lines.into_iter().filter(|l| re.is_match(l)).collect()
        } else if let Some(ref re) = self.strip_lines {
            lines.into_iter().filter(|l| !re.is_match(l)).collect()
        } else {
            lines
        };

        // Stage 7: Deduplicate consecutive identical lines
        let lines: Vec<&str> = if self.dedup_consecutive {
            dedup_consecutive(lines)
        } else {
            lines
        };

        // Stage 8: truncate_lines_at
        let lines: Vec<String> = if let Some(max_width) = self.truncate_lines_at {
            lines
                .into_iter()
                .map(|l| truncate_str(l, max_width))
                .collect()
        } else {
            lines.into_iter().map(|s| s.to_string()).collect()
        };

        // Stage 9: head_lines / tail_lines
        let total = lines.len();
        let lines = apply_head_tail(&lines, self.head_lines, self.tail_lines, total);

        // Stage 10: max_lines — absolute cap
        let lines = if let Some(max) = self.max_lines {
            if lines.len() > max {
                let mut truncated: Vec<String> = lines.into_iter().take(max).collect();
                truncated.push("...".to_string());
                truncated
            } else {
                lines
            }
        } else {
            lines
        };

        // Stage 11: on_empty fallback
        let result = lines.join("\n");
        if result.trim().is_empty() {
            self.on_empty.clone().unwrap_or_default()
        } else {
            result
        }
    }
}

/// Collapse runs of 3+ blank lines into a single blank line, trim trailing whitespace per line.
fn normalize_whitespace(input: &str) -> String {
    let mut result = Vec::new();
    let mut blank_count = 0u32;

    for line in input.lines() {
        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            blank_count += 1;
            if blank_count <= 1 {
                result.push(String::new());
            }
        } else {
            blank_count = 0;
            result.push(trimmed.to_string());
        }
    }

    result.join("\n")
}

/// Collapse consecutive identical lines: keeps the first of each run,
/// drops duplicates, and inserts a static marker for runs of 3+.
fn dedup_consecutive(lines: Vec<&str>) -> Vec<&str> {
    if lines.len() < 2 {
        return lines;
    }

    let mut result: Vec<&str> = Vec::with_capacity(lines.len());
    let mut run_count: usize = 1;

    for i in 0..lines.len() {
        if i + 1 < lines.len() && lines[i] == lines[i + 1] {
            run_count += 1;
            continue;
        }
        result.push(lines[i]);
        if run_count > 2 {
            result.push("  ... (repeated lines omitted)");
        }
        run_count = 1;
    }

    result
}

/// Strip ANSI escape codes from text.
pub fn strip_ansi(input: &str) -> String {
    // Match CSI sequences, OSC sequences, and other common ANSI escapes
    let re = Regex::new(r"\x1b\[[0-9;]*[a-zA-Z]|\x1b\][^\x07]*\x07|\x1b\([AB012]").unwrap();
    re.replace_all(input, "").to_string()
}

/// Truncate a string to max_width characters (Unicode-safe).
fn truncate_str(s: &str, max_width: usize) -> String {
    if s.chars().count() <= max_width {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max_width.saturating_sub(3)).collect();
        format!("{truncated}...")
    }
}

/// Apply head/tail line limits with "..." markers.
fn apply_head_tail(
    lines: &[String],
    head: Option<usize>,
    tail: Option<usize>,
    total: usize,
) -> Vec<String> {
    match (head, tail) {
        (Some(h), Some(t)) if total > h + t => {
            let mut result: Vec<String> = lines.iter().take(h).cloned().collect();
            let omitted = total - h - t;
            result.push(format!("... ({omitted} lines omitted)"));
            result.extend(lines.iter().skip(total - t).cloned());
            result
        }
        (Some(h), None) if total > h => {
            let mut result: Vec<String> = lines.iter().take(h).cloned().collect();
            let omitted = total - h;
            result.push(format!("... ({omitted} more lines)"));
            result
        }
        (None, Some(t)) if total > t => {
            let omitted = total - t;
            let mut result = vec![format!("... ({omitted} lines omitted)")];
            result.extend(lines.iter().skip(total - t).cloned());
            result
        }
        _ => lines.to_vec(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A permissive base def: no filtering stages enabled. Tests enable one
    /// stage at a time via struct-update syntax.
    fn base_def() -> FilterDef {
        FilterDef {
            name: "test",
            match_command: r"^test\b",
            strip_ansi: false,
            replace: &[],
            match_output: None,
            unless: None,
            strip_lines: None,
            keep_lines: None,
            truncate_lines_at: None,
            head_lines: None,
            tail_lines: None,
            max_lines: None,
            on_empty: None,
        }
    }

    fn apply(def: &FilterDef, input: &str) -> String {
        FilterPipeline::from_def(def).apply(input)
    }

    // ── FilterDef::matches ──────────────────────────────────────────────

    #[test]
    fn matches_trims_leading_whitespace() {
        let def = base_def();
        assert!(def.matches("test foo"));
        assert!(def.matches("   test foo"));
        assert!(!def.matches("testing foo"));
        assert!(!def.matches("run test"));
    }

    #[test]
    fn matches_invalid_regex_is_false_not_panic() {
        let def = FilterDef {
            match_command: r"([unclosed",
            ..base_def()
        };
        assert!(!def.matches("anything"));
    }

    // ── strip_ansi ──────────────────────────────────────────────────────

    #[test]
    fn strip_ansi_removes_csi_sequences() {
        assert_eq!(strip_ansi("\x1b[31mred\x1b[0m plain"), "red plain");
        assert_eq!(strip_ansi("\x1b[1;32;40mbold\x1b[m"), "bold");
    }

    #[test]
    fn strip_ansi_removes_osc_and_charset_sequences() {
        assert_eq!(strip_ansi("\x1b]0;window title\x07rest"), "rest");
        assert_eq!(strip_ansi("\x1b(Btext"), "text");
    }

    #[test]
    fn strip_ansi_leaves_plain_and_utf8_text_alone() {
        assert_eq!(strip_ansi("héllo wörld ✓ 日本語"), "héllo wörld ✓ 日本語");
    }

    // ── normalize_whitespace ────────────────────────────────────────────

    #[test]
    fn normalize_collapses_blank_runs_to_one() {
        assert_eq!(normalize_whitespace("a\n\n\n\nb"), "a\n\nb");
    }

    #[test]
    fn normalize_trims_trailing_whitespace_per_line() {
        assert_eq!(normalize_whitespace("a  \nb\t\n  c   "), "a\nb\n  c");
    }

    #[test]
    fn normalize_treats_whitespace_only_lines_as_blank() {
        assert_eq!(normalize_whitespace("a\n   \n\t\nb"), "a\n\nb");
    }

    // ── dedup_consecutive ───────────────────────────────────────────────

    #[test]
    fn dedup_handles_empty_and_single() {
        assert_eq!(dedup_consecutive(vec![]), Vec::<&str>::new());
        assert_eq!(dedup_consecutive(vec!["a"]), vec!["a"]);
    }

    #[test]
    fn dedup_run_of_two_keeps_one_no_marker() {
        assert_eq!(dedup_consecutive(vec!["a", "a", "b"]), vec!["a", "b"]);
    }

    #[test]
    fn dedup_run_of_three_adds_marker() {
        assert_eq!(
            dedup_consecutive(vec!["x", "a", "a", "a", "y"]),
            vec!["x", "a", "  ... (repeated lines omitted)", "y"]
        );
    }

    #[test]
    fn dedup_keeps_non_consecutive_duplicates() {
        assert_eq!(
            dedup_consecutive(vec!["a", "b", "a", "b"]),
            vec!["a", "b", "a", "b"]
        );
    }

    // ── truncate_str ────────────────────────────────────────────────────

    #[test]
    fn truncate_str_at_or_under_limit_unchanged() {
        assert_eq!(truncate_str("abcde", 5), "abcde");
        assert_eq!(truncate_str("abc", 5), "abc");
    }

    #[test]
    fn truncate_str_over_limit_ends_with_ellipsis() {
        assert_eq!(truncate_str("abcdefgh", 6), "abc...");
        assert_eq!(truncate_str("abcdefgh", 6).chars().count(), 6);
    }

    #[test]
    fn truncate_str_multibyte_no_panic_and_char_safe() {
        // 10 two-byte chars, limit 5 -> 2 chars + "..."
        let s = "éééééééééé";
        assert_eq!(truncate_str(s, 5), "éé...");
        // emoji (4-byte) mixed in
        let s = "🎉🎉🎉🎉🎉🎉";
        assert_eq!(truncate_str(s, 4), "🎉...");
        // exactly at limit passes through
        assert_eq!(truncate_str("日本語", 3), "日本語");
    }

    #[test]
    fn truncate_str_tiny_limit_does_not_underflow() {
        assert_eq!(truncate_str("abcdef", 2), "...");
        assert_eq!(truncate_str("abcdef", 0), "...");
    }

    // ── apply_head_tail ─────────────────────────────────────────────────

    fn nlines(n: usize) -> Vec<String> {
        (1..=n).map(|i| format!("line{i}")).collect()
    }

    #[test]
    fn head_tail_under_limit_untouched() {
        let lines = nlines(5);
        assert_eq!(apply_head_tail(&lines, Some(3), Some(2), 5), lines);
        assert_eq!(apply_head_tail(&lines, Some(10), None, 5), lines);
        assert_eq!(apply_head_tail(&lines, None, Some(10), 5), lines);
        assert_eq!(apply_head_tail(&lines, None, None, 5), lines);
    }

    #[test]
    fn head_and_tail_with_accurate_omitted_count() {
        let out = apply_head_tail(&nlines(10), Some(3), Some(2), 10);
        assert_eq!(
            out,
            vec![
                "line1",
                "line2",
                "line3",
                "... (5 lines omitted)",
                "line9",
                "line10"
            ]
        );
    }

    #[test]
    fn head_only_with_accurate_count() {
        let out = apply_head_tail(&nlines(5), Some(3), None, 5);
        assert_eq!(out, vec!["line1", "line2", "line3", "... (2 more lines)"]);
    }

    #[test]
    fn tail_only_with_accurate_count() {
        let out = apply_head_tail(&nlines(5), None, Some(2), 5);
        assert_eq!(out, vec!["... (3 lines omitted)", "line4", "line5"]);
    }

    // ── apply(): stage behavior ─────────────────────────────────────────

    #[test]
    fn passthrough_when_no_stages_enabled() {
        let def = base_def();
        let input = "alpha\nbeta\ngamma";
        assert_eq!(apply(&def, input), input);
    }

    #[test]
    fn empty_input_without_on_empty_yields_empty_string() {
        assert_eq!(apply(&base_def(), ""), "");
        assert_eq!(apply(&base_def(), "   \n \n"), "");
    }

    #[test]
    fn empty_input_with_on_empty_yields_fallback() {
        let def = FilterDef {
            on_empty: Some("(no output)"),
            ..base_def()
        };
        assert_eq!(apply(&def, ""), "(no output)");
    }

    #[test]
    fn on_empty_when_all_lines_filtered_out() {
        let def = FilterDef {
            keep_lines: Some(r"^NEVER_MATCHES$"),
            on_empty: Some("(all filtered)"),
            ..base_def()
        };
        assert_eq!(apply(&def, "a\nb\nc"), "(all filtered)");
    }

    #[test]
    fn replace_applies_per_line() {
        let def = FilterDef {
            replace: &[(r"\s*\(\d+ ms\)", "")],
            ..base_def()
        };
        assert_eq!(
            apply(&def, "test one (12 ms)\ntest two (5 ms)"),
            "test one\ntest two"
        );
    }

    #[test]
    fn match_output_short_circuits_to_on_empty() {
        let def = FilterDef {
            match_output: Some(r"working tree clean"),
            on_empty: Some("(clean)"),
            ..base_def()
        };
        assert_eq!(apply(&def, "nothing to commit, working tree clean"), "(clean)");
    }

    #[test]
    fn match_output_without_on_empty_yields_empty() {
        let def = FilterDef {
            match_output: Some(r"all good"),
            ..base_def()
        };
        assert_eq!(apply(&def, "all good here"), "");
    }

    #[test]
    fn unless_preserves_full_error_output() {
        let def = FilterDef {
            unless: Some(r"(?i)error"),
            keep_lines: Some(r"^NEVER_MATCHES$"),
            head_lines: Some(1),
            on_empty: Some("(ok)"),
            ..base_def()
        };
        let input = "line one\nerror: something broke\nline three";
        // With an error present, every line survives despite keep_lines/head_lines.
        assert_eq!(apply(&def, input), input);
        // Without the error, aggressive filtering applies.
        assert_eq!(apply(&def, "line one\nline two"), "(ok)");
    }

    #[test]
    fn unless_wins_over_match_output() {
        // Regression: an error must never be swallowed by a success-pattern
        // short-circuit. unless is checked before match_output.
        let def = FilterDef {
            match_output: Some(r"working tree clean"),
            unless: Some(r"(?i)error:"),
            on_empty: Some("(clean)"),
            ..base_def()
        };
        let input = "error: index corrupt\nnothing to commit, working tree clean";
        let out = apply(&def, input);
        assert!(out.contains("error: index corrupt"), "error was hidden: {out}");
        assert_ne!(out, "(clean)");
    }

    #[test]
    fn strip_lines_removes_only_matching_lines() {
        let def = FilterDef {
            strip_lines: Some(r"^\s*Compiling\s"),
            ..base_def()
        };
        assert_eq!(
            apply(&def, "   Compiling foo v1.0\nreal output\n   Compiling bar v2.0"),
            "real output"
        );
    }

    #[test]
    fn keep_lines_takes_precedence_over_strip_lines() {
        let def = FilterDef {
            keep_lines: Some(r"^keep"),
            strip_lines: Some(r"^keep"), // would remove the same lines if applied
            ..base_def()
        };
        assert_eq!(apply(&def, "keep me\ndrop me"), "keep me");
    }

    #[test]
    fn max_lines_caps_with_ellipsis_marker() {
        let def = FilterDef {
            max_lines: Some(2),
            ..base_def()
        };
        assert_eq!(apply(&def, "a\nb\nc\nd"), "a\nb\n...");
        // At or under the cap: no marker.
        assert_eq!(apply(&def, "a\nb"), "a\nb");
    }

    #[test]
    fn head_tail_boundary_exact_sum_not_truncated() {
        let def = FilterDef {
            head_lines: Some(2),
            tail_lines: Some(2),
            ..base_def()
        };
        assert_eq!(apply(&def, "a\nb\nc\nd"), "a\nb\nc\nd");
        assert_eq!(apply(&def, "a\nb\nc\nd\ne"), "a\nb\n... (1 lines omitted)\nd\ne");
    }

    #[test]
    fn truncate_lines_at_via_apply_multibyte_no_panic() {
        let def = FilterDef {
            truncate_lines_at: Some(10),
            ..base_def()
        };
        let long_emoji = "🚀".repeat(50);
        let out = apply(&def, &long_emoji);
        assert_eq!(out.chars().count(), 10);
        assert!(out.ends_with("..."));
    }

    #[test]
    fn crlf_line_endings_handled_without_stray_cr() {
        let out = apply(&base_def(), "line one\r\nline two\r\nline three\r\n");
        assert_eq!(out, "line one\nline two\nline three");
        assert!(!out.contains('\r'));
    }

    #[test]
    fn input_resembling_pipeline_markers_passes_through() {
        let input = "... (5 lines omitted)\nreal line\n  ... (repeated lines omitted)";
        assert_eq!(apply(&base_def(), input), input);
    }

    #[test]
    fn dedup_via_apply_preserves_count_marker() {
        let out = apply(&base_def(), "same\nsame\nsame\nsame\nother");
        assert_eq!(out, "same\n  ... (repeated lines omitted)\nother");
    }

    #[test]
    fn ansi_stripped_before_line_matching() {
        let def = FilterDef {
            strip_ansi: true,
            keep_lines: Some(r"^PASS"),
            on_empty: Some("(none)"),
            ..base_def()
        };
        // Without ANSI stripping, the escape prefix would defeat ^PASS.
        assert_eq!(apply(&def, "\x1b[32mPASS test_a\x1b[0m\nnoise"), "PASS test_a");
    }

    // ── default_pipeline ────────────────────────────────────────────────

    #[test]
    fn default_pipeline_passes_small_output_unchanged() {
        let p = FilterPipeline::default_pipeline();
        let input = "hello\nworld";
        assert_eq!(p.apply(input), input);
    }

    #[test]
    fn default_pipeline_empty_input_yields_empty() {
        assert_eq!(FilterPipeline::default_pipeline().apply(""), "");
    }

    #[test]
    fn default_pipeline_truncates_huge_output_head_and_tail() {
        let p = FilterPipeline::default_pipeline();
        let input: Vec<String> = (1..=500).map(|i| format!("row {i}")).collect();
        let out = p.apply(&input.join("\n"));
        let out_lines: Vec<&str> = out.lines().collect();
        // head 150 + marker + tail 20
        assert_eq!(out_lines.len(), 171);
        assert_eq!(out_lines[0], "row 1");
        assert_eq!(out_lines[149], "row 150");
        assert_eq!(out_lines[150], "... (330 lines omitted)");
        assert_eq!(out_lines[170], "row 500");
    }

    #[test]
    fn default_pipeline_strips_ansi_and_collapses_blanks() {
        let p = FilterPipeline::default_pipeline();
        assert_eq!(p.apply("\x1b[1ma\x1b[0m\n\n\n\nb"), "a\n\nb");
    }

    #[test]
    fn from_def_invalid_optional_regexes_degrade_to_noop() {
        // A bad pattern in any optional stage must not panic or filter anything.
        let def = FilterDef {
            replace: &[(r"([bad", "x")],
            match_output: Some(r"([bad"),
            unless: Some(r"([bad"),
            strip_lines: Some(r"([bad"),
            keep_lines: Some(r"([bad"),
            ..base_def()
        };
        assert_eq!(apply(&def, "a\nb"), "a\nb");
    }
}
