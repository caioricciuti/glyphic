use crate::paths;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use tauri::Emitter;

#[derive(Serialize, Deserialize, Clone)]
pub struct PipelineNode {
    pub id: String,
    #[serde(rename = "type")]
    pub node_type: String,
    pub label: String,
    pub x: f64,
    pub y: f64,
    pub config: serde_json::Value,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PipelineConnection {
    pub id: String,
    pub from_node: String,
    pub to_node: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Pipeline {
    pub id: String,
    pub name: String,
    pub nodes: Vec<PipelineNode>,
    pub connections: Vec<PipelineConnection>,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default)]
    pub schedule: Option<String>,
    #[serde(default)]
    pub schedule_enabled: bool,
    #[serde(default)]
    pub last_run: Option<String>,
    #[serde(default)]
    pub last_run_status: Option<String>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct PipelineStore {
    pub pipelines: Vec<Pipeline>,
}

// --- Run History ---

const MAX_RUNS_PER_PIPELINE: usize = 50;
const MAX_OUTPUT_LEN: usize = 10_000;
const MAX_INPUT_LEN: usize = 5_000;

#[derive(Serialize, Deserialize, Clone)]
pub struct PipelineRunRecord {
    pub id: String,
    pub pipeline_id: String,
    pub pipeline_name: String,
    pub started_at: String,
    pub completed_at: String,
    pub status: String,
    pub duration_ms: u64,
    pub node_results: Vec<NodeRunResult>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct NodeRunResult {
    pub node_id: String,
    pub label: String,
    pub node_type: String,
    pub status: String,
    pub duration_ms: u64,
    pub input: String,
    pub output: String,
    pub config: String,
}

#[derive(Serialize, Deserialize, Default)]
pub struct PipelineHistoryStore {
    pub runs: HashMap<String, Vec<PipelineRunRecord>>,
}

fn history_path() -> std::path::PathBuf {
    paths::claude_home().join("glyphic-pipeline-history.json")
}

fn load_history() -> PipelineHistoryStore {
    let path = history_path();
    if !path.exists() { return PipelineHistoryStore::default(); }
    fs::read_to_string(&path).ok()
        .and_then(|c| serde_json::from_str(&c).ok())
        .unwrap_or_default()
}

fn save_history(store: &PipelineHistoryStore) -> Result<(), String> {
    let content = serde_json::to_string_pretty(store).map_err(|e| format!("{e}"))?;
    fs::write(history_path(), content).map_err(|e| format!("{e}"))
}

fn truncate_str(s: &str, max: usize) -> String {
    if s.len() <= max { return s.to_string(); }
    s.chars().take(max).collect::<String>() + "...(truncated)"
}

fn save_run_record(pipeline: &Pipeline, status: &str, duration_ms: u64, node_results: Vec<NodeRunResult>) {
    let record = PipelineRunRecord {
        id: format!("run-{}", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()),
        pipeline_id: pipeline.id.clone(),
        pipeline_name: pipeline.name.clone(),
        started_at: iso_now_offset(-(duration_ms as i64)),
        completed_at: iso_now_offset(0),
        status: status.to_string(),
        duration_ms,
        node_results,
    };

    let mut history = load_history();
    let runs = history.runs.entry(pipeline.id.clone()).or_default();
    runs.insert(0, record);
    runs.truncate(MAX_RUNS_PER_PIPELINE);
    let _ = save_history(&history);

    // Also update last_run / last_run_status on the pipeline
    let mut store = load_store();
    if let Some(p) = store.pipelines.iter_mut().find(|p| p.id == pipeline.id) {
        p.last_run = Some(iso_now_offset(0));
        p.last_run_status = Some(status.to_string());
        let _ = save_store(&store);
    }
}

fn iso_now_offset(offset_ms: i64) -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as i64 + offset_ms;
    let total_secs = secs / 1000;
    let d = total_secs / 86400;
    let rem = total_secs % 86400;
    // Approximate ISO — good enough for display
    let h = rem / 3600;
    let m = (rem % 3600) / 60;
    let s = rem % 60;
    // Days since epoch → rough date
    let days = d as u64;
    let (y, mo, day) = days_to_date(days);
    format!("{y:04}-{mo:02}-{day:02}T{h:02}:{m:02}:{s:02}Z")
}

fn days_to_date(days: u64) -> (u64, u64, u64) {
    // Algorithm from http://howardhinnant.github.io/date_algorithms.html
    let z = days + 719468;
    let era = z / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

// --- Pipeline Storage ---

fn store_path() -> std::path::PathBuf {
    paths::claude_home().join("glyphic-pipelines.json")
}

pub fn load_store() -> PipelineStore {
    let path = store_path();
    if !path.exists() { return PipelineStore::default(); }
    fs::read_to_string(&path).ok()
        .and_then(|c| serde_json::from_str(&c).ok())
        .unwrap_or_default()
}

pub fn save_store(store: &PipelineStore) -> Result<(), String> {
    let content = serde_json::to_string_pretty(store).map_err(|e| format!("{e}"))?;
    fs::write(store_path(), content).map_err(|e| format!("{e}"))
}

#[tauri::command]
pub fn list_pipelines() -> Result<Vec<Pipeline>, String> {
    Ok(load_store().pipelines)
}

#[tauri::command]
pub fn save_pipeline(pipeline: Pipeline) -> Result<(), String> {
    let mut store = load_store();
    if let Some(existing) = store.pipelines.iter_mut().find(|p| p.id == pipeline.id) {
        *existing = pipeline;
    } else {
        store.pipelines.push(pipeline);
    }
    save_store(&store)
}

#[tauri::command]
pub fn delete_pipeline(id: String) -> Result<(), String> {
    let mut store = load_store();
    store.pipelines.retain(|p| p.id != id);
    save_store(&store)?;
    // Clean up history
    let mut history = load_history();
    history.runs.remove(&id);
    let _ = save_history(&history);
    Ok(())
}

#[tauri::command]
pub fn list_pipeline_history(pipeline_id: String) -> Result<Vec<PipelineRunRecord>, String> {
    let store = load_history();
    Ok(store.runs.get(&pipeline_id).cloned().unwrap_or_default())
}

#[tauri::command]
pub fn delete_pipeline_history(pipeline_id: String) -> Result<(), String> {
    let mut store = load_history();
    store.runs.remove(&pipeline_id);
    save_history(&store)
}

// Static cancel flag
static CANCEL_FLAG: AtomicBool = AtomicBool::new(false);

#[tauri::command]
pub fn cancel_pipeline_run() -> Result<(), String> {
    CANCEL_FLAG.store(true, Ordering::SeqCst);
    Ok(())
}

fn substitute_vars(text: &str, ctx: Option<&str>, all_outputs: &HashMap<String, String>) -> String {
    let mut result = text.to_string();
    if let Some(c) = ctx {
        result = result.replace("{{input}}", c).replace("$INPUT", c);
    }
    for (label, output) in all_outputs {
        result = result.replace(&format!("{{{{{}}}}}", label), output);
    }
    result
}

fn resolve_claude_prompt(node: &PipelineNode, context: &Option<String>, all_outputs: &HashMap<String, String>) -> String {
    let ctx = context.as_deref();
    let raw_prompt = node.config.get("prompt").and_then(|p| p.as_str()).unwrap_or("hello");
    let prompt = substitute_vars(raw_prompt, ctx, all_outputs);
    if let Some(c) = ctx {
        format!("Context:\n{}\n\n{}", &c[..c.len().min(2000)], prompt)
    } else {
        prompt
    }
}

fn execute_node(node: &PipelineNode, context: &Option<String>, all_outputs: &HashMap<String, String>) -> Result<String, String> {
    let ctx = context.as_deref();

    match node.node_type.as_str() {
        "bash" | "github" => {
            let raw = node.config.get("command").and_then(|c| c.as_str()).unwrap_or("echo 'no command'");
            let command = substitute_vars(raw, ctx, all_outputs);
            let output = std::process::Command::new("sh").args(["-c", &command])
                .env("PATH", paths::enriched_path())
                .output()
                .map_err(|e| format!("failed: {e}"))?;
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            if output.status.success() {
                Ok(if stdout.is_empty() { stderr } else { stdout })
            } else {
                Err(format!("Exit {}: {}", output.status.code().unwrap_or(-1), if stderr.is_empty() { stdout } else { stderr }))
            }
        }
        "claude" => {
            let full = resolve_claude_prompt(node, context, all_outputs);
            let skip = node.config.get("dangerouslySkipPermissions")
                .and_then(|v| v.as_str()) == Some("true");
            let mut args: Vec<&str> = vec![];
            if skip { args.push("--dangerously-skip-permissions"); }
            args.extend(["--print", &full]);
            let output = std::process::Command::new(paths::claude_bin())
                .args(&args)
                .env("PATH", paths::enriched_path())
                .env("CLAUDE_NO_TELEMETRY", "1")
                .output()
                .map_err(|e| format!("failed: {e}"))?;
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            if stdout.is_empty() && !stderr.is_empty() { Err(stderr) } else { Ok(stdout) }
        }
        "http" => {
            let raw_url = node.config.get("url").and_then(|u| u.as_str()).unwrap_or("");
            let url = substitute_vars(raw_url, ctx, all_outputs);
            let method = node.config.get("method").and_then(|m| m.as_str()).unwrap_or("GET");
            let raw_body = node.config.get("body").and_then(|b| b.as_str()).unwrap_or("");
            let body = substitute_vars(raw_body, ctx, all_outputs);
            let mut args = vec!["-s".to_string(), "-X".to_string(), method.to_string()];
            // Parse custom headers from JSON array
            if let Some(headers_str) = node.config.get("headers").and_then(|h| h.as_str()) {
                let headers_raw = substitute_vars(headers_str, ctx, all_outputs);
                if let Ok(headers) = serde_json::from_str::<Vec<serde_json::Value>>(&headers_raw) {
                    for h in headers {
                        let key = h.get("key").and_then(|k| k.as_str()).unwrap_or("");
                        let val = h.get("value").and_then(|v| v.as_str()).unwrap_or("");
                        if !key.is_empty() {
                            args.push("-H".to_string());
                            args.push(format!("{key}: {val}"));
                        }
                    }
                }
            }
            if !body.is_empty() && method != "GET" && method != "HEAD" {
                args.push("-d".to_string());
                args.push(body);
                // Auto-add Content-Type if not already set via headers
                let has_content_type = args.iter().any(|a| a.to_lowercase().starts_with("content-type:"));
                if !has_content_type {
                    args.push("-H".to_string());
                    args.push("Content-Type: application/json".to_string());
                }
            }
            if method == "HEAD" {
                args.push("-I".to_string());
            }
            args.push(url.to_string());
            let output = std::process::Command::new("curl").args(&args)
                .env("PATH", paths::enriched_path())
                .output()
                .map_err(|e| format!("failed: {e}"))?;
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            if stdout.is_empty() && !stderr.is_empty() { Err(stderr) } else { Ok(stdout) }
        }
        "transform" => {
            let operation = node.config.get("operation").and_then(|o| o.as_str()).unwrap_or("passthrough");
            let input = ctx.unwrap_or("");
            match operation {
                "uppercase" => Ok(input.to_uppercase()),
                "lowercase" => Ok(input.to_lowercase()),
                "trim" => Ok(input.trim().to_string()),
                "line_count" => Ok(input.lines().count().to_string()),
                "word_count" => Ok(input.split_whitespace().count().to_string()),
                "first_line" => Ok(input.lines().next().unwrap_or("").to_string()),
                "json_pretty" => {
                    serde_json::from_str::<serde_json::Value>(input)
                        .map(|v| serde_json::to_string_pretty(&v).unwrap_or_else(|_| input.to_string()))
                        .map_err(|e| format!("JSON parse error: {e}"))
                }
                _ => Ok(input.to_string()),
            }
        }
        "delay" => {
            let secs: u64 = node.config.get("seconds").and_then(|s| s.as_str())
                .and_then(|s| s.parse().ok()).unwrap_or(1);
            std::thread::sleep(std::time::Duration::from_secs(secs));
            Ok(ctx.unwrap_or("").to_string())
        }
        "git" => {
            let raw_path = node.config.get("path").and_then(|p| p.as_str()).unwrap_or(".");
            let path = substitute_vars(raw_path, ctx, all_outputs);
            let operation = node.config.get("operation").and_then(|o| o.as_str()).unwrap_or("status");
            let branch = node.config.get("branch").and_then(|b| b.as_str()).unwrap_or("");
            let message = node.config.get("message").and_then(|m| m.as_str()).unwrap_or("");
            let mut args: Vec<String> = vec![operation.to_string()];
            match operation {
                "commit" => {
                    args = vec!["commit".to_string(), "-am".to_string(), message.to_string()];
                }
                "checkout" => {
                    if !branch.is_empty() { args.push(branch.to_string()); }
                }
                "clone" => {
                    if !branch.is_empty() { args.push(branch.to_string()); }
                    args.push(path.clone());
                }
                "log" => {
                    args = vec!["log".to_string(), "--oneline".to_string(), "-20".to_string()];
                }
                _ => {}
            }
            let mut cmd = std::process::Command::new("git");
            cmd.args(&args).env("PATH", paths::enriched_path());
            if operation != "clone" {
                cmd.current_dir(&path);
            }
            let output = cmd.output().map_err(|e| format!("failed: {e}"))?;
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            if output.status.success() {
                Ok(if stdout.is_empty() { stderr } else { stdout })
            } else {
                Err(format!("git error: {}", if stderr.is_empty() { stdout } else { stderr }))
            }
        }
        "filter" => {
            let input = ctx.unwrap_or("");
            let condition = node.config.get("condition").and_then(|c| c.as_str()).unwrap_or("not_empty");
            let raw_value = node.config.get("value").and_then(|v| v.as_str()).unwrap_or("");
            let value = substitute_vars(raw_value, ctx, all_outputs);
            let passes = match condition {
                "not_empty" => !input.trim().is_empty(),
                "empty" => input.trim().is_empty(),
                "contains" => input.contains(&value),
                "not_contains" => !input.contains(&value),
                "equals" => input.trim() == value.trim(),
                "not_equals" => input.trim() != value.trim(),
                "regex" => {
                    // Simple regex matching via grep
                    let output = std::process::Command::new("sh")
                        .args(["-c", &format!("echo {} | grep -qE {}", shell_escape(input), shell_escape(&value))])
                        .output();
                    output.map(|o| o.status.success()).unwrap_or(false)
                }
                _ => true,
            };
            if passes {
                Ok(input.to_string())
            } else {
                Err(format!("Filter condition '{}' not met", condition))
            }
        }
        "readfile" => {
            let raw_path = node.config.get("path").and_then(|p| p.as_str()).unwrap_or("");
            let path = expand_tilde(&substitute_vars(raw_path, ctx, all_outputs));
            fs::read_to_string(&path).map_err(|e| format!("Read file error: {e}"))
        }
        "writefile" => {
            let raw_path = node.config.get("path").and_then(|p| p.as_str()).unwrap_or("");
            let path = expand_tilde(&substitute_vars(raw_path, ctx, all_outputs));
            let mode = node.config.get("mode").and_then(|m| m.as_str()).unwrap_or("overwrite");
            let content = ctx.unwrap_or("");
            // Create parent directories if they don't exist
            if let Some(parent) = std::path::Path::new(&path).parent() {
                if !parent.as_os_str().is_empty() {
                    fs::create_dir_all(parent).map_err(|e| format!("Create dir error: {e}"))?;
                }
            }
            if mode == "append" {
                use std::io::Write;
                let mut file = std::fs::OpenOptions::new().create(true).append(true).open(&path)
                    .map_err(|e| format!("Write file error: {e}"))?;
                file.write_all(content.as_bytes()).map_err(|e| format!("Write error: {e}"))?;
            } else {
                fs::write(&path, content).map_err(|e| format!("Write file error: {e}"))?;
            }
            Ok(format!("Written to {path}"))
        }
        "notification" => {
            let raw_title = node.config.get("title").and_then(|t| t.as_str()).unwrap_or("Pipeline");
            let title = substitute_vars(raw_title, ctx, all_outputs);
            let raw_body = node.config.get("body").and_then(|b| b.as_str()).unwrap_or("");
            let body_text = substitute_vars(raw_body, ctx, all_outputs);
            let body_final = if body_text.is_empty() { ctx.unwrap_or("").to_string() } else { body_text };
            // Use osascript for macOS notification
            let script = format!(
                "display notification \"{}\" with title \"{}\"",
                body_final.replace('\"', "\\\"").chars().take(200).collect::<String>(),
                title.replace('\"', "\\\"")
            );
            std::process::Command::new("osascript").args(["-e", &script]).output()
                .map_err(|e| format!("Notification failed: {e}"))?;
            Ok(ctx.unwrap_or("").to_string())
        }
        "jsonextract" => {
            let input = ctx.unwrap_or("{}");
            let json_path = node.config.get("path").and_then(|p| p.as_str()).unwrap_or("");
            let fallback = node.config.get("fallback").and_then(|f| f.as_str()).unwrap_or("");
            let parsed: serde_json::Value = serde_json::from_str(input)
                .map_err(|e| format!("JSON parse error: {e}"))?;
            let result = traverse_json_path(&parsed, json_path);
            match result {
                Some(v) => Ok(match v {
                    serde_json::Value::String(s) => s.clone(),
                    other => other.to_string(),
                }),
                None => {
                    if fallback.is_empty() {
                        Err(format!("Path '{}' not found in JSON", json_path))
                    } else {
                        Ok(fallback.to_string())
                    }
                }
            }
        }
        _ => Ok(ctx.unwrap_or("").to_string()),
    }
}

fn expand_tilde(path: &str) -> String {
    if path.starts_with("~/") || path == "~" {
        if let Some(home) = dirs::home_dir() {
            return format!("{}/{}", home.display(), &path[2..]);
        }
    }
    path.to_string()
}

fn shell_escape(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

/// Traverse a JSON value using dot-path notation like "data.items[0].name"
fn traverse_json_path<'a>(value: &'a serde_json::Value, path: &str) -> Option<&'a serde_json::Value> {
    if path.is_empty() { return Some(value); }
    let mut current = value;
    for segment in path.split('.') {
        // Handle array indexing like "items[0]"
        if let Some(bracket_pos) = segment.find('[') {
            let key = &segment[..bracket_pos];
            if !key.is_empty() {
                current = current.get(key)?;
            }
            // Parse all bracket indices
            let mut rest = &segment[bracket_pos..];
            while rest.starts_with('[') {
                let end = rest.find(']')?;
                let idx: usize = rest[1..end].parse().ok()?;
                current = current.get(idx)?;
                rest = &rest[end + 1..];
            }
        } else {
            current = current.get(segment)?;
        }
    }
    Some(current)
}

// Topological sort
pub fn topo_sort(nodes: &[PipelineNode], connections: &[PipelineConnection]) -> Vec<String> {
    use std::collections::{HashMap, VecDeque};
    let mut adj: HashMap<String, Vec<String>> = HashMap::new();
    let mut in_deg: HashMap<String, usize> = HashMap::new();
    for n in nodes { adj.insert(n.id.clone(), vec![]); in_deg.insert(n.id.clone(), 0); }
    for c in connections {
        adj.get_mut(&c.from_node).map(|v| v.push(c.to_node.clone()));
        *in_deg.entry(c.to_node.clone()).or_insert(0) += 1;
    }
    let mut queue: VecDeque<String> = in_deg.iter().filter(|(_, &d)| d == 0).map(|(id, _)| id.clone()).collect();
    let mut sorted = Vec::new();
    while let Some(id) = queue.pop_front() {
        sorted.push(id.clone());
        for next in adj.get(&id).cloned().unwrap_or_default() {
            let d = in_deg.get_mut(&next).unwrap();
            *d -= 1;
            if *d == 0 { queue.push_back(next); }
        }
    }
    sorted
}

// Channel for pipeline thread to wait while frontend handles interactive Claude nodes
static INTERACTIVE_SENDER: Mutex<Option<std::sync::mpsc::Sender<String>>> = Mutex::new(None);

#[tauri::command]
pub fn resume_pipeline_node(output: String) -> Result<(), String> {
    let sender = INTERACTIVE_SENDER.lock()
        .map_err(|e| format!("lock: {e}"))?
        .take()
        .ok_or("No interactive node waiting")?;
    sender.send(output).map_err(|e| format!("send: {e}"))?;
    Ok(())
}

#[tauri::command]
pub fn start_pipeline_run(pipeline: Pipeline, app_handle: tauri::AppHandle) -> Result<(), String> {
    CANCEL_FLAG.store(false, Ordering::SeqCst);

    let sorted = topo_sort(&pipeline.nodes, &pipeline.connections);

    std::thread::spawn(move || {
        let _ = app_handle.emit("pipeline-event", serde_json::json!({
            "type": "started",
            "message": format!("Started at {}", chrono_now()),
        }));

        let run_start = std::time::Instant::now();
        let mut last_output: Option<String> = None;
        let mut all_outputs: HashMap<String, String> = HashMap::new();
        let mut node_run_results: Vec<NodeRunResult> = Vec::new();
        let mut run_status = "success";

        for node_id in &sorted {
            if CANCEL_FLAG.load(Ordering::SeqCst) {
                let _ = app_handle.emit("pipeline-event", serde_json::json!({
                    "type": "cancelled",
                    "node_id": node_id,
                }));
                run_status = "cancelled";
                break;
            }

            let node = match pipeline.nodes.iter().find(|n| &n.id == node_id) {
                Some(n) => n,
                None => continue,
            };

            if node.node_type == "input" || node.node_type == "output" {
                let _ = app_handle.emit("pipeline-event", serde_json::json!({
                    "type": "node_done",
                    "node_id": node.id,
                    "label": node.label,
                    "output": "",
                    "duration": 0,
                }));
                continue;
            }

            let node_input = last_output.clone().unwrap_or_default();

            let _ = app_handle.emit("pipeline-event", serde_json::json!({
                "type": "node_start",
                "node_id": node.id,
                "label": node.label,
                "input": node_input,
                "config": serde_json::to_string(&node.config).unwrap_or_default(),
                "node_type": node.node_type,
            }));

            let start = std::time::Instant::now();

            // For Claude nodes, check interactive mode
            let is_interactive = node.node_type == "claude"
                && node.config.get("interactive").and_then(|v| v.as_str()) != Some("false");

            let result = if is_interactive {
                // Resolve prompt and working dir
                let prompt = resolve_claude_prompt(node, &last_output, &all_outputs);
                let working_dir = node.config.get("path")
                    .and_then(|p| p.as_str())
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| dirs::home_dir().unwrap_or_default().to_string_lossy().to_string());
                let session_id = format!("pipeline-{}", node.id);

                // Create channel — frontend will send output when Claude exits
                let (tx, rx) = std::sync::mpsc::channel::<String>();
                if let Ok(mut sender) = INTERACTIVE_SENDER.lock() {
                    *sender = Some(tx);
                }

                let skip_perms = node.config.get("dangerouslySkipPermissions")
                    .and_then(|v| v.as_str()) == Some("true");

                // Tell frontend to spawn terminal with this prompt
                let _ = app_handle.emit("pipeline-event", serde_json::json!({
                    "type": "node_interactive_start",
                    "node_id": node.id,
                    "label": node.label,
                    "session_id": session_id,
                    "prompt": prompt,
                    "working_dir": working_dir,
                    "dangerously_skip_permissions": skip_perms,
                }));

                // Block until frontend calls resume_pipeline_node or cancel
                let interactive_result: Result<String, String> = loop {
                    match rx.recv_timeout(std::time::Duration::from_millis(200)) {
                        Ok(output) => {
                            // Strip ANSI escape codes
                            let stripped = strip_ansi_escapes::strip(output.as_bytes());
                            let clean = String::from_utf8_lossy(&stripped).trim().to_string();
                            break Ok(if clean.is_empty() { "(no output)".to_string() } else { clean });
                        }
                        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                            if CANCEL_FLAG.load(Ordering::SeqCst) {
                                break Err("Cancelled".to_string());
                            }
                        }
                        Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                            break Err("Interactive session disconnected".to_string());
                        }
                    }
                };
                // Clean up sender
                if let Ok(mut sender) = INTERACTIVE_SENDER.lock() {
                    *sender = None;
                }
                interactive_result
            } else {
                execute_node(node, &last_output, &all_outputs)
            };

            match result {
                Ok(output) => {
                    let duration = start.elapsed().as_millis() as u64;
                    all_outputs.insert(node.label.clone(), output.clone());
                    last_output = Some(output.clone());
                    node_run_results.push(NodeRunResult {
                        node_id: node.id.clone(),
                        label: node.label.clone(),
                        node_type: node.node_type.clone(),
                        status: "done".to_string(),
                        duration_ms: duration,
                        input: truncate_str(&node_input, MAX_INPUT_LEN),
                        output: truncate_str(&output, MAX_OUTPUT_LEN),
                        config: serde_json::to_string(&node.config).unwrap_or_default(),
                    });
                    let _ = app_handle.emit("pipeline-event", serde_json::json!({
                        "type": "node_done",
                        "node_id": node.id,
                        "label": node.label,
                        "output": output,
                        "duration": duration,
                    }));
                }
                Err(err) => {
                    let duration = start.elapsed().as_millis() as u64;
                    node_run_results.push(NodeRunResult {
                        node_id: node.id.clone(),
                        label: node.label.clone(),
                        node_type: node.node_type.clone(),
                        status: "error".to_string(),
                        duration_ms: duration,
                        input: truncate_str(&node_input, MAX_INPUT_LEN),
                        output: truncate_str(&err, MAX_OUTPUT_LEN),
                        config: serde_json::to_string(&node.config).unwrap_or_default(),
                    });
                    run_status = "error";
                    let _ = app_handle.emit("pipeline-event", serde_json::json!({
                        "type": "node_error",
                        "node_id": node.id,
                        "label": node.label,
                        "output": err,
                        "duration": duration,
                    }));
                    break;
                }
            }
        }

        // Save run history
        let total_duration = run_start.elapsed().as_millis() as u64;
        save_run_record(&pipeline, run_status, total_duration, node_run_results);

        let _ = app_handle.emit("pipeline-event", serde_json::json!({
            "type": "completed",
            "message": format!("Completed at {}", chrono_now()),
        }));
    });

    Ok(())
}

#[tauri::command]
pub fn run_single_node(node: PipelineNode, context: Option<String>) -> Result<String, String> {
    let all_outputs = HashMap::new();
    execute_node(&node, &context, &all_outputs)
}

fn chrono_now() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
    let h = (secs % 86400) / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    format!("{h:02}:{m:02}:{s:02}")
}
