use crate::paths;
use serde::{Deserialize, Serialize};
use std::fs;

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
    if !path.exists() {
        return PipelineStore::default();
    }
    fs::read_to_string(&path)
        .ok()
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

#[tauri::command]
pub fn run_pipeline_node(node_type: String, config: serde_json::Value, context: Option<String>) -> Result<String, String> {
    match node_type.as_str() {
        "bash" => {
            let raw_command = config.get("command").and_then(|c| c.as_str()).unwrap_or("echo 'no command'");
            // Replace {{input}} with previous node's output
            let command = if let Some(ref ctx) = context {
                raw_command
                    .replace("{{input}}", ctx)
                    .replace("$INPUT", ctx)
            } else {
                raw_command.to_string()
            };
            let output = std::process::Command::new("sh")
                .args(["-c", &command])
                .output()
                .map_err(|e| format!("failed to run: {e}"))?;
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            if output.status.success() {
                Ok(if stdout.is_empty() { stderr } else { stdout })
            } else {
                Err(format!("Exit code {}: {}", output.status.code().unwrap_or(-1), if stderr.is_empty() { stdout } else { stderr }))
            }
        }
        "claude" => {
            let prompt = config.get("prompt").and_then(|p| p.as_str()).unwrap_or("hello");
            // Build full prompt including context from previous node
            let full_prompt = if let Some(ctx) = context {
                format!("Context from previous step:\n{}\n\n{}", ctx.chars().take(2000).collect::<String>(), prompt)
            } else {
                prompt.to_string()
            };
            // Use --print for non-interactive output
            let output = std::process::Command::new("claude")
                .args(["--print", &full_prompt])
                .env("CLAUDE_NO_TELEMETRY", "1")
                .output()
                .map_err(|e| format!("failed to run claude: {e}"))?;
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            if stdout.is_empty() && !stderr.is_empty() {
                Err(stderr)
            } else {
                Ok(stdout)
            }
        }
        "github" => {
            let raw_command = config.get("command").and_then(|c| c.as_str()).unwrap_or("gh --help");
            let command = if let Some(ref ctx) = context {
                raw_command
                    .replace("{{input}}", ctx)
                    .replace("$INPUT", ctx)
            } else {
                raw_command.to_string()
            };
            let output = std::process::Command::new("sh")
                .args(["-c", &command])
                .output()
                .map_err(|e| format!("failed to run: {e}"))?;
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            Ok(if stdout.is_empty() { stderr } else { stdout })
        }
        _ => Ok(String::new()),
    }
}
