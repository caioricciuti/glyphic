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
// Minimal MCP client with two transports: stdio (newline-delimited JSON-RPC,
// spawned child killed on drop) and streamable HTTP (POST per message, SSE
// responses parsed, Mcp-Session-Id tracked). Initialize, then list/call tools.

enum McpTransport {
    Stdio {
        child: std::process::Child,
        stdin: std::process::ChildStdin,
        rx: std::sync::mpsc::Receiver<String>,
    },
    Http {
        agent: ureq::Agent,
        url: String,
        headers: Vec<(String, String)>,
        session_id: Option<String>,
    },
    /// Legacy SSE transport: a GET stream delivers an `endpoint` event with
    /// the POST URL; requests go to that endpoint and responses arrive as
    /// `data:` events on the stream.
    Sse {
        agent: ureq::Agent,
        endpoint: String,
        headers: Vec<(String, String)>,
        rx: std::sync::mpsc::Receiver<String>,
    },
}

/// Resolve an SSE `endpoint` value against the base stream URL.
fn resolve_endpoint(base_url: &str, endpoint: &str) -> String {
    if endpoint.starts_with("http://") || endpoint.starts_with("https://") {
        return endpoint.to_string();
    }
    if let Some(rest) = endpoint.strip_prefix('/') {
        // scheme://host[:port] + /endpoint
        if let Some(scheme_end) = base_url.find("://") {
            let host_end = base_url[scheme_end + 3..]
                .find('/')
                .map(|i| scheme_end + 3 + i)
                .unwrap_or(base_url.len());
            return format!("{}/{}", &base_url[..host_end], rest);
        }
    }
    // relative to the stream URL's directory
    match base_url.rsplit_once('/') {
        Some((dir, _)) => format!("{dir}/{endpoint}"),
        None => endpoint.to_string(),
    }
}

struct McpClient {
    transport: McpTransport,
    next_id: u64,
}

fn extract_result(value: serde_json::Value) -> Result<serde_json::Value, String> {
    if let Some(err) = value.get("error") {
        return Err(format!("server error: {err}"));
    }
    Ok(value.get("result").cloned().unwrap_or(serde_json::Value::Null))
}

impl McpClient {
    fn connect(config: &serde_json::Value) -> Result<Self, String> {
        if config.get("url").is_some() {
            Self::connect_http(config)
        } else if config.get("command").is_some() {
            Self::spawn_stdio(config)
        } else {
            Err("server config has neither a \"command\" (stdio) nor a \"url\" (http)".to_string())
        }
    }

    fn connect_http(config: &serde_json::Value) -> Result<Self, String> {
        let url = config
            .get("url")
            .and_then(|u| u.as_str())
            .ok_or("missing server url")?
            .to_string();
        let headers: Vec<(String, String)> = config
            .get("headers")
            .and_then(|h| h.as_object())
            .map(|h| {
                h.iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect()
            })
            .unwrap_or_default();
        let agent = ureq::AgentBuilder::new()
            .timeout(std::time::Duration::from_secs(20))
            .build();
        Ok(Self {
            transport: McpTransport::Http { agent, url, headers, session_id: None },
            next_id: 1,
        })
    }

    fn spawn_stdio(config: &serde_json::Value) -> Result<Self, String> {
        use std::io::BufRead;
        use std::process::{Command, Stdio};

        let command = config
            .get("command")
            .and_then(|c| c.as_str())
            .ok_or("missing server command")?;
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

        Ok(Self {
            transport: McpTransport::Stdio { child, stdin, rx },
            next_id: 1,
        })
    }

    fn request(&mut self, method: &str, params: serde_json::Value) -> Result<serde_json::Value, String> {
        let id = self.next_id;
        self.next_id += 1;
        let msg = serde_json::json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params });
        match self.transport {
            McpTransport::Http { .. } => self
                .http_send(&msg, Some(id), method)?
                .ok_or_else(|| format!("empty response for {method}")),
            McpTransport::Sse { .. } => self.sse_request(&msg, id, method),
            McpTransport::Stdio { .. } => self.stdio_request(&msg, id, method),
        }
    }

    fn notify(&mut self, method: &str) {
        let msg = serde_json::json!({ "jsonrpc": "2.0", "method": method });
        if matches!(self.transport, McpTransport::Http { .. }) {
            let _ = self.http_send(&msg, None, method);
            return;
        }
        match &mut self.transport {
            McpTransport::Stdio { stdin, .. } => {
                use std::io::Write;
                let _ = writeln!(stdin, "{msg}");
                let _ = stdin.flush();
            }
            McpTransport::Sse { agent, endpoint, headers, .. } => {
                let mut req = agent.post(endpoint).set("Content-Type", "application/json");
                for (k, v) in headers.iter() {
                    req = req.set(k, v);
                }
                let _ = req.send_string(&msg.to_string());
            }
            McpTransport::Http { .. } => unreachable!(),
        }
    }

    /// Switch from streamable HTTP to the legacy SSE transport: open the GET
    /// stream, wait for the `endpoint` event, and route requests there.
    fn upgrade_to_sse(&mut self) -> Result<(), String> {
        use std::io::BufRead;

        let McpTransport::Http { url, headers, .. } = &self.transport else {
            return Err("not an http transport".to_string());
        };
        let url = url.clone();
        let headers = headers.clone();

        // No overall timeout: the SSE stream stays open for the client's life.
        let agent = ureq::AgentBuilder::new()
            .timeout_connect(std::time::Duration::from_secs(10))
            .timeout_read(std::time::Duration::from_secs(120))
            .build();

        let mut req = agent.get(&url).set("Accept", "text/event-stream");
        for (k, v) in headers.iter() {
            req = req.set(k, v);
        }
        let resp = req.call().map_err(|e| format!("SSE connect failed: {e}"))?;
        let reader = resp.into_reader();

        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let br = std::io::BufReader::new(reader);
            for line in br.lines().map_while(Result::ok) {
                if tx.send(line).is_err() {
                    break;
                }
            }
        });

        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        let mut in_endpoint_event = false;
        let endpoint = loop {
            let remaining = deadline
                .checked_duration_since(std::time::Instant::now())
                .ok_or("timed out waiting for the SSE endpoint event")?;
            let line = rx
                .recv_timeout(remaining)
                .map_err(|_| "timed out waiting for the SSE endpoint event".to_string())?;
            if let Some(ev) = line.strip_prefix("event:") {
                in_endpoint_event = ev.trim() == "endpoint";
                continue;
            }
            if let Some(data) = line.strip_prefix("data:") {
                if in_endpoint_event {
                    break resolve_endpoint(&url, data.trim());
                }
            }
        };

        self.transport = McpTransport::Sse { agent, endpoint, headers, rx };
        Ok(())
    }

    fn sse_request(
        &mut self,
        msg: &serde_json::Value,
        id: u64,
        method: &str,
    ) -> Result<serde_json::Value, String> {
        let McpTransport::Sse { agent, endpoint, headers, rx } = &mut self.transport else {
            return Err("not an sse transport".to_string());
        };

        let mut req = agent.post(endpoint).set("Content-Type", "application/json");
        for (k, v) in headers.iter() {
            req = req.set(k, v);
        }
        match req.send_string(&msg.to_string()) {
            Ok(_) => {}
            Err(ureq::Error::Status(code, r)) => {
                let body = r.into_string().unwrap_or_default();
                let snippet: String = body.chars().take(300).collect();
                return Err(format!("server returned HTTP {code} for {method}: {snippet}"));
            }
            Err(e) => return Err(format!("request failed: {e}")),
        }

        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
        loop {
            let remaining = deadline
                .checked_duration_since(std::time::Instant::now())
                .ok_or_else(|| format!("timed out waiting for {method} response"))?;
            let line = rx
                .recv_timeout(remaining)
                .map_err(|_| format!("timed out waiting for {method} response on the SSE stream"))?;
            let Some(data) = line.strip_prefix("data:") else { continue };
            let Ok(value) = serde_json::from_str::<serde_json::Value>(data.trim()) else {
                continue;
            };
            if value.get("id").and_then(|i| i.as_u64()) == Some(id) {
                return extract_result(value);
            }
        }
    }

    fn stdio_request(
        &mut self,
        msg: &serde_json::Value,
        id: u64,
        method: &str,
    ) -> Result<serde_json::Value, String> {
        use std::io::Write;
        let McpTransport::Stdio { stdin, rx, .. } = &mut self.transport else {
            return Err("not a stdio transport".to_string());
        };
        writeln!(stdin, "{msg}").map_err(|e| format!("write to server failed: {e}"))?;
        let _ = stdin.flush();

        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
        loop {
            let remaining = deadline
                .checked_duration_since(std::time::Instant::now())
                .ok_or_else(|| format!("timed out waiting for {method} response"))?;
            let line = rx
                .recv_timeout(remaining)
                .map_err(|_| format!("timed out waiting for {method} response (is this a valid MCP stdio server?)"))?;
            let Ok(value) = serde_json::from_str::<serde_json::Value>(&line) else {
                continue; // non-JSON noise on stdout
            };
            if value.get("id").and_then(|i| i.as_u64()) == Some(id) {
                return extract_result(value);
            }
            // notifications and unrelated ids are skipped
        }
    }

    /// POST one JSON-RPC message. `id: None` is a notification (no response
    /// expected). Handles plain JSON and SSE-formatted response bodies, and
    /// tracks the Mcp-Session-Id header across calls.
    fn http_send(
        &mut self,
        msg: &serde_json::Value,
        id: Option<u64>,
        method: &str,
    ) -> Result<Option<serde_json::Value>, String> {
        let McpTransport::Http { agent, url, headers, session_id } = &mut self.transport else {
            return Err("not an http transport".to_string());
        };

        let mut req = agent
            .post(url)
            .set("Content-Type", "application/json")
            .set("Accept", "application/json, text/event-stream");
        for (k, v) in headers.iter() {
            req = req.set(k, v);
        }
        if let Some(sid) = session_id.as_deref() {
            req = req.set("Mcp-Session-Id", sid);
        }

        let resp = match req.send_string(&msg.to_string()) {
            Ok(r) => r,
            Err(ureq::Error::Status(code, r)) => {
                let body = r.into_string().unwrap_or_default();
                let snippet: String = body.chars().take(300).collect();
                return Err(format!("server returned HTTP {code} for {method}: {snippet}"));
            }
            Err(e) => return Err(format!("request failed: {e}")),
        };

        if let Some(sid) = resp.header("mcp-session-id") {
            *session_id = Some(sid.to_string());
        }
        let content_type = resp.header("content-type").unwrap_or("").to_string();
        let body = resp
            .into_string()
            .map_err(|e| format!("failed to read response: {e}"))?;

        let Some(id) = id else { return Ok(None) };

        if content_type.contains("text/event-stream") {
            for line in body.lines() {
                let Some(data) = line.strip_prefix("data:") else { continue };
                let Ok(value) = serde_json::from_str::<serde_json::Value>(data.trim()) else {
                    continue;
                };
                if value.get("id").and_then(|i| i.as_u64()) == Some(id) {
                    return extract_result(value).map(Some);
                }
            }
            Err(format!("no response for {method} in event stream"))
        } else {
            let value: serde_json::Value = serde_json::from_str(&body)
                .map_err(|e| format!("invalid JSON response for {method}: {e}"))?;
            extract_result(value).map(Some)
        }
    }

    fn initialize(&mut self) -> Result<serde_json::Value, String> {
        let params = serde_json::json!({
            "protocolVersion": "2025-06-18",
            "capabilities": {},
            "clientInfo": { "name": "glyphic", "version": env!("CARGO_PKG_VERSION") }
        });
        let init = match self.request("initialize", params.clone()) {
            Ok(init) => init,
            // Streamable HTTP rejected the POST: retry over legacy SSE
            Err(e)
                if matches!(self.transport, McpTransport::Http { .. })
                    && (e.contains("HTTP 405") || e.contains("HTTP 404")) =>
            {
                self.upgrade_to_sse()?;
                self.request("initialize", params)?
            }
            Err(e) => return Err(e),
        };
        self.notify("notifications/initialized");
        Ok(init)
    }
}

impl Drop for McpClient {
    fn drop(&mut self) {
        if let McpTransport::Stdio { child, .. } = &mut self.transport {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

/// Connect to a server, initialize, and list its tools.
#[tauri::command(async)]
pub fn test_mcp_server(config: serde_json::Value) -> Result<serde_json::Value, String> {
    let mut client = McpClient::connect(&config)?;
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
    let mut client = McpClient::connect(&config)?;
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
    #[ignore] // network test: GLYPHIC_TEST_MCP_CONFIG='{"url":...}' cargo test -- --ignored
    fn mcp_client_live_http_roundtrip() {
        let config: serde_json::Value = if let Ok(raw) = std::env::var("GLYPHIC_TEST_MCP_CONFIG") {
            serde_json::from_str(&raw).expect("GLYPHIC_TEST_MCP_CONFIG must be JSON")
        } else if let Ok(url) = std::env::var("GLYPHIC_TEST_MCP_URL") {
            serde_json::json!({ "url": url })
        } else {
            return;
        };
        let result = super::test_mcp_server(config).expect("live roundtrip should succeed");
        let tools = result["tools"].as_array().expect("tools array");
        println!(
            "connected to {:?} ({} tools): {:?}",
            result["serverInfo"],
            tools.len(),
            tools.iter().filter_map(|t| t["name"].as_str()).take(8).collect::<Vec<_>>()
        );
    }

    #[test]
    fn resolve_endpoint_variants() {
        assert_eq!(
            super::resolve_endpoint("http://x.io/mcp", "/messages?sid=1"),
            "http://x.io/messages?sid=1"
        );
        assert_eq!(
            super::resolve_endpoint("http://x.io/a/mcp", "msg"),
            "http://x.io/a/msg"
        );
        assert_eq!(
            super::resolve_endpoint("http://x.io/mcp", "https://y.io/m"),
            "https://y.io/m"
        );
    }

    #[test]
    fn mcp_client_rejects_config_without_command_or_url() {
        let config = serde_json::json!({ "type": "http" });
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
