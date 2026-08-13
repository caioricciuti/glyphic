use crate::commands::settings;
use crate::paths;

/// MCP servers live inside settings.json under the "mcpServers" key,
/// or in .mcp.json at the project root (local scope, the default for `claude mcp add`).
///
/// Read .mcp.json from project root. Handles both formats:
/// - Format A (standard): { "server-name": { config } }
/// - Format B (legacy):   { "mcpServers": { "server-name": { config } } }
fn read_mcp_json(project_path: &str) -> Result<serde_json::Value, String> {
    let path = paths::project_mcp_json_path(project_path);
    if !path.exists() {
        return Ok(serde_json::json!({}));
    }
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("failed to read {}: {e}", path.display()))?;
    let data: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("failed to parse {}: {e}", path.display()))?;

    if let Some(mcp_servers) = data.get("mcpServers") {
        Ok(mcp_servers.clone())
    } else {
        Ok(data)
    }
}

/// Write .mcp.json in Format A (direct server map, no wrapper).
fn write_mcp_json(project_path: &str, servers: serde_json::Value) -> Result<(), String> {
    let path = paths::project_mcp_json_path(project_path);
    let content = serde_json::to_string_pretty(&servers)
        .map_err(|e| format!("failed to serialize: {e}"))?;
    paths::write_atomic(&path, &content)
}

/// Read Claude Desktop config and extract mcpServers.
fn read_desktop_config() -> Result<(serde_json::Value, serde_json::Value), String> {
    let path = paths::claude_desktop_config_path();
    if !path.exists() {
        return Ok((serde_json::json!({}), serde_json::json!({})));
    }
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("failed to read {}: {e}", path.display()))?;
    let full: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("failed to parse {}: {e}", path.display()))?;
    let servers = full.get("mcpServers").cloned().unwrap_or(serde_json::json!({}));
    Ok((full, servers))
}

/// Write mcpServers back into Claude Desktop config, preserving other keys.
fn write_desktop_config(mut full: serde_json::Value, servers: serde_json::Value) -> Result<(), String> {
    let path = paths::claude_desktop_config_path();
    let obj = full.as_object_mut().ok_or("desktop config is not an object")?;
    if servers.as_object().is_none_or(|s| s.is_empty()) {
        obj.remove("mcpServers");
    } else {
        obj.insert("mcpServers".to_string(), servers);
    }
    let content = serde_json::to_string_pretty(&full)
        .map_err(|e| format!("failed to serialize: {e}"))?;
    paths::write_atomic(&path, &content)
}

#[tauri::command]
pub fn list_mcp_servers(
    scope: String,
    project_path: Option<String>,
) -> Result<serde_json::Value, String> {
    if scope == "mcp-local" {
        let pp = project_path.as_deref().ok_or("project_path required for local scope")?;
        return read_mcp_json(pp);
    }
    if scope == "desktop" {
        let (_full, servers) = read_desktop_config()?;
        return Ok(servers);
    }
    let settings = settings::read_settings(scope, project_path)?;
    Ok(settings.get("mcpServers").cloned().unwrap_or(serde_json::json!({})))
}

#[tauri::command]
pub fn upsert_mcp_server(
    scope: String,
    project_path: Option<String>,
    name: String,
    config: serde_json::Value,
) -> Result<(), String> {
    if scope == "mcp-local" {
        let pp = project_path.as_deref().ok_or("project_path required for local scope")?;
        let mut servers = read_mcp_json(pp)?;
        let obj = servers.as_object_mut().ok_or("mcp.json is not an object")?;
        obj.insert(name, config);
        return write_mcp_json(pp, servers);
    }
    if scope == "desktop" {
        let (full, mut servers) = read_desktop_config()?;
        let obj = servers.as_object_mut().ok_or("desktop mcpServers is not an object")?;
        obj.insert(name, config);
        return write_desktop_config(full, servers);
    }

    let mut current = settings::read_settings(scope.clone(), project_path.clone())?;
    let obj = current.as_object_mut().ok_or("settings is not an object")?;

    let servers = obj
        .entry("mcpServers")
        .or_insert_with(|| serde_json::json!({}));

    let servers_obj = servers.as_object_mut().ok_or("mcpServers is not an object")?;
    servers_obj.insert(name, config);

    settings::write_settings(scope, project_path, current)
}

#[tauri::command]
pub fn delete_mcp_server(
    scope: String,
    project_path: Option<String>,
    name: String,
) -> Result<(), String> {
    if scope == "mcp-local" {
        let pp = project_path.as_deref().ok_or("project_path required for local scope")?;
        let mut servers = read_mcp_json(pp)?;
        if let Some(obj) = servers.as_object_mut() {
            obj.remove(&name);
        }
        return write_mcp_json(pp, servers);
    }
    if scope == "desktop" {
        let (full, mut servers) = read_desktop_config()?;
        if let Some(obj) = servers.as_object_mut() {
            obj.remove(&name);
        }
        return write_desktop_config(full, servers);
    }

    let mut current = settings::read_settings(scope.clone(), project_path.clone())?;
    let obj = current.as_object_mut().ok_or("settings is not an object")?;

    if let Some(servers) = obj.get_mut("mcpServers") {
        if let Some(servers_obj) = servers.as_object_mut() {
            servers_obj.remove(&name);
            if servers_obj.is_empty() {
                obj.remove("mcpServers");
            }
        }
    }

    settings::write_settings(scope, project_path, current)
}

// ── Live testing ────────────────────────────────────────────────────────────
// Minimal MCP client over stdio (newline-delimited JSON-RPC 2.0): spawn the
// configured server, initialize, then list or call tools. The child is killed
// when the client drops.

struct McpClient {
    child: std::process::Child,
    stdin: std::process::ChildStdin,
    rx: std::sync::mpsc::Receiver<String>,
    next_id: u64,
}

impl McpClient {
    fn spawn(config: &serde_json::Value) -> Result<Self, String> {
        use std::io::BufRead;
        use std::process::{Command, Stdio};

        let command = config
            .get("command")
            .and_then(|c| c.as_str())
            .ok_or("only stdio servers (with a \"command\") can be live-tested for now")?;
        let args: Vec<String> = config
            .get("args")
            .and_then(|a| a.as_array())
            .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();

        let mut cmd = Command::new(command);
        cmd.args(&args)
            .env("PATH", paths::enriched_path())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        if let Some(env) = config.get("env").and_then(|e| e.as_object()) {
            for (k, v) in env {
                if let Some(val) = v.as_str() {
                    cmd.env(k, val);
                }
            }
        }

        let mut child = cmd.spawn().map_err(|e| format!("failed to start server: {e}"))?;
        let stdin = child.stdin.take().ok_or("failed to open server stdin")?;
        let stdout = child.stdout.take().ok_or("failed to open server stdout")?;

        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let reader = std::io::BufReader::new(stdout);
            for line in reader.lines().map_while(Result::ok) {
                if tx.send(line).is_err() {
                    break;
                }
            }
        });

        Ok(Self { child, stdin, rx, next_id: 1 })
    }

    fn request(&mut self, method: &str, params: serde_json::Value) -> Result<serde_json::Value, String> {
        use std::io::Write;

        let id = self.next_id;
        self.next_id += 1;
        let msg = serde_json::json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params });
        writeln!(self.stdin, "{msg}").map_err(|e| format!("write to server failed: {e}"))?;
        let _ = self.stdin.flush();

        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
        loop {
            let remaining = deadline
                .checked_duration_since(std::time::Instant::now())
                .ok_or_else(|| format!("timed out waiting for {method} response"))?;
            let line = self
                .rx
                .recv_timeout(remaining)
                .map_err(|_| format!("timed out waiting for {method} response (is this a valid MCP stdio server?)"))?;
            let Ok(value) = serde_json::from_str::<serde_json::Value>(&line) else {
                continue; // non-JSON noise on stdout
            };
            if value.get("id").and_then(|i| i.as_u64()) == Some(id) {
                if let Some(err) = value.get("error") {
                    return Err(format!("server error: {err}"));
                }
                return Ok(value.get("result").cloned().unwrap_or(serde_json::Value::Null));
            }
            // notifications and unrelated ids are skipped
        }
    }

    fn initialize(&mut self) -> Result<serde_json::Value, String> {
        use std::io::Write;

        let init = self.request(
            "initialize",
            serde_json::json!({
                "protocolVersion": "2025-06-18",
                "capabilities": {},
                "clientInfo": { "name": "glyphic", "version": env!("CARGO_PKG_VERSION") }
            }),
        )?;
        let note = serde_json::json!({ "jsonrpc": "2.0", "method": "notifications/initialized" });
        let _ = writeln!(self.stdin, "{note}");
        let _ = self.stdin.flush();
        Ok(init)
    }
}

impl Drop for McpClient {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Connect to a server, initialize, and list its tools.
#[tauri::command(async)]
pub fn test_mcp_server(config: serde_json::Value) -> Result<serde_json::Value, String> {
    let mut client = McpClient::spawn(&config)?;
    let init = client.initialize()?;
    let tools = client.request("tools/list", serde_json::json!({}))?;
    Ok(serde_json::json!({
        "serverInfo": init.get("serverInfo"),
        "protocolVersion": init.get("protocolVersion"),
        "tools": tools.get("tools").cloned().unwrap_or(serde_json::json!([])),
    }))
}

/// Connect to a server and execute one tool call.
#[tauri::command(async)]
pub fn call_mcp_tool(
    config: serde_json::Value,
    tool: String,
    args: serde_json::Value,
) -> Result<serde_json::Value, String> {
    let mut client = McpClient::spawn(&config)?;
    client.initialize()?;
    client.request("tools/call", serde_json::json!({ "name": tool, "arguments": args }))
}

#[cfg(all(test, unix))]
mod tests {
    #[test]
    fn mcp_client_roundtrip_against_fake_stdio_server() {
        // A fake MCP server: answers initialize (id 1) and tools/list (id 2),
        // with a stray notification in between that the client must skip.
        let script = concat!(
            "while read line; do case \"$line\" in ",
            "*initialize*) echo '{\"jsonrpc\":\"2.0\",\"method\":\"notifications/log\",\"params\":{}}'; ",
            "echo '{\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{\"protocolVersion\":\"2025-06-18\",\"serverInfo\":{\"name\":\"fake\",\"version\":\"1.0\"}}}';; ",
            "*tools/list*) echo '{\"jsonrpc\":\"2.0\",\"id\":2,\"result\":{\"tools\":[{\"name\":\"echo\",\"description\":\"Echo tool\"}]}}';; ",
            "esac; done"
        );
        let config = serde_json::json!({ "command": "sh", "args": ["-c", script] });
        let result = super::test_mcp_server(config).expect("roundtrip should succeed");
        assert_eq!(result["serverInfo"]["name"], "fake");
        assert_eq!(result["tools"][0]["name"], "echo");
    }

    #[test]
    fn mcp_client_rejects_url_only_config() {
        let config = serde_json::json!({ "url": "https://example.com/mcp" });
        assert!(super::test_mcp_server(config).is_err());
    }
}

#[tauri::command]
pub fn get_cloud_mcps() -> Result<Vec<String>, String> {
    let path = paths::claude_home().join("mcp-needs-auth-cache.json");
    if !path.exists() {
        return Ok(vec![]);
    }
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("failed to read: {e}"))?;
    let data: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("failed to parse: {e}"))?;

    Ok(data
        .as_object()
        .map(|obj| obj.keys().cloned().collect())
        .unwrap_or_default())
}
