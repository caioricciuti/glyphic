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
            truncate_lines_at: def.truncate_lines_at,
            head_lines: def.head_lines,
            tail_lines: def.tail_lines,
            max_lines: def.max_lines,
            on_empty: def.on_empty.map(|s| s.to_string()),
        }
    }

    /// Apply the full 8-stage pipeline to the input text.
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

        // Stage 3: match_output — short-circuit if full output matches
        if let Some(ref re) = self.match_output {
            if re.is_match(&text) {
                return self.on_empty.clone().unwrap_or_default();
            }
        }

        // Stage 4: unless — skip filtering if output contains this pattern (preserve errors)
        if let Some(ref re) = self.unless {
            if re.is_match(&text) {
                return text;
            }
        }

        // Stage 5: strip_lines / keep_lines
        let lines: Vec<&str> = text.lines().collect();
        let lines = if let Some(ref re) = self.keep_lines {
            lines.into_iter().filter(|l| re.is_match(l)).collect()
        } else if let Some(ref re) = self.strip_lines {
            lines.into_iter().filter(|l| !re.is_match(l)).collect()
        } else {
            lines
        };

        // Stage 6: truncate_lines_at
        let lines: Vec<String> = if let Some(max_width) = self.truncate_lines_at {
            lines
                .into_iter()
                .map(|l| truncate_str(l, max_width))
                .collect()
        } else {
            lines.into_iter().map(|s| s.to_string()).collect()
        };

        // Stage 7: head_lines / tail_lines
        let total = lines.len();
        let lines = apply_head_tail(&lines, self.head_lines, self.tail_lines, total);

        // Stage 8: max_lines — absolute cap
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

        // Stage 9: on_empty fallback
        let result = lines.join("\n");
        if result.trim().is_empty() {
            self.on_empty.clone().unwrap_or_default()
        } else {
            result
        }
    }
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
