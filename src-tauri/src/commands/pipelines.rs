use crate::paths;
use serde::{Deserialize, Serialize};
use std::fs;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
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
}

#[derive(Serialize, Deserialize, Default)]
struct PipelineStore {
    pub pipelines: Vec<Pipeline>,
}

fn store_path() -> std::path::PathBuf {
    paths::claude_home().join("glyphic-pipelines.json")
}

fn load_store() -> PipelineStore {
    let path = store_path();
    if !path.exists() { return PipelineStore::default(); }
    fs::read_to_string(&path).ok()
        .and_then(|c| serde_json::from_str(&c).ok())
        .unwrap_or_default()
}

fn save_store(store: &PipelineStore) -> Result<(), String> {
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
    save_store(&store)
}

// Static cancel flag
static CANCEL_FLAG: AtomicBool = AtomicBool::new(false);

#[tauri::command]
pub fn cancel_pipeline_run() -> Result<(), String> {
    CANCEL_FLAG.store(true, Ordering::SeqCst);
    Ok(())
}

fn execute_node(node: &PipelineNode, context: &Option<String>, all_outputs: &std::collections::HashMap<String, String>) -> Result<String, String> {
    let ctx = context.as_deref();

    // Replace all {{node_label}} references with their outputs
    fn substitute_vars(text: &str, ctx: Option<&str>, all_outputs: &std::collections::HashMap<String, String>) -> String {
        let mut result = text.to_string();
        // Replace {{input}} with immediate previous output
        if let Some(c) = ctx {
            result = result.replace("{{input}}", c).replace("$INPUT", c);
        }
        // Replace {{NodeName}} with that node's output
        for (label, output) in all_outputs {
            result = result.replace(&format!("{{{{{}}}}}", label), output);
        }
        result
    }

    match node.node_type.as_str() {
        "bash" | "github" => {
            let raw = node.config.get("command").and_then(|c| c.as_str()).unwrap_or("echo 'no command'");
            let command = substitute_vars(raw, ctx, all_outputs);
            let output = std::process::Command::new("sh").args(["-c", &command]).output()
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
            let raw_prompt = node.config.get("prompt").and_then(|p| p.as_str()).unwrap_or("hello");
            let prompt = substitute_vars(raw_prompt, ctx, all_outputs);
            let full = if let Some(c) = ctx {
                format!("Context:\n{}\n\n{}", &c[..c.len().min(2000)], prompt)
            } else { prompt };
            let output = std::process::Command::new("claude")
                .args(["--print", &full])
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
            // Use curl for simplicity
            let mut args = vec!["-s".to_string(), "-X".to_string(), method.to_string()];
            if !body.is_empty() && method != "GET" {
                args.push("-d".to_string());
                args.push(body);
                args.push("-H".to_string());
                args.push("Content-Type: application/json".to_string());
            }
            args.push(url.to_string());
            let output = std::process::Command::new("curl").args(&args).output()
                .map_err(|e| format!("failed: {e}"))?;
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
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
        _ => Ok(ctx.unwrap_or("").to_string()),
    }
}

// Topological sort
fn topo_sort(nodes: &[PipelineNode], connections: &[PipelineConnection]) -> Vec<String> {
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

#[tauri::command]
pub fn start_pipeline_run(pipeline: Pipeline, app_handle: tauri::AppHandle) -> Result<(), String> {
    CANCEL_FLAG.store(false, Ordering::SeqCst);

    let sorted = topo_sort(&pipeline.nodes, &pipeline.connections);

    std::thread::spawn(move || {
        let _ = app_handle.emit("pipeline-event", serde_json::json!({
            "type": "started",
            "message": format!("Started at {}", chrono_now()),
        }));

        let mut last_output: Option<String> = None;
        let mut all_outputs: std::collections::HashMap<String, String> = std::collections::HashMap::new();

        for node_id in &sorted {
            if CANCEL_FLAG.load(Ordering::SeqCst) {
                let _ = app_handle.emit("pipeline-event", serde_json::json!({
                    "type": "cancelled",
                    "node_id": node_id,
                }));
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

            let _ = app_handle.emit("pipeline-event", serde_json::json!({
                "type": "node_start",
                "node_id": node.id,
                "label": node.label,
            }));

            let start = std::time::Instant::now();
            match execute_node(node, &last_output, &all_outputs) {
                Ok(output) => {
                    let duration = start.elapsed().as_millis() as u64;
                    all_outputs.insert(node.label.clone(), output.clone());
                    last_output = Some(output.clone());
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

        let _ = app_handle.emit("pipeline-event", serde_json::json!({
            "type": "completed",
            "message": format!("Completed at {}", chrono_now()),
        }));
    });

    Ok(())
}

fn chrono_now() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
    let h = (secs % 86400) / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    format!("{h:02}:{m:02}:{s:02}")
}
