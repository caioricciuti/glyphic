use base64::Engine;
use portable_pty::{native_pty_system, CommandBuilder, MasterPty, PtySize};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use tauri::Emitter;

pub struct PtySession {
    master: Box<dyn MasterPty + Send>,
    writer: Box<dyn Write + Send>,
}

pub struct PtyState {
    pub sessions: Arc<Mutex<HashMap<String, PtySession>>>,
}

impl Default for PtyState {
    fn default() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[tauri::command]
pub fn spawn_terminal(
    id: String,
    path: String,
    cols: u16,
    rows: u16,
    state: tauri::State<PtyState>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let pty_system = native_pty_system();

    let pair = pty_system
        .openpty(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|e| format!("failed to open pty: {e}"))?;

    let mut cmd = CommandBuilder::new("claude");
    cmd.cwd(&path);

    // Set TERM for proper color support
    cmd.env("TERM", "xterm-256color");
    cmd.env("COLORTERM", "truecolor");

    let _child = pair
        .slave
        .spawn_command(cmd)
        .map_err(|e| format!("failed to spawn claude: {e}"))?;

    // Drop slave — we only interact via master
    drop(pair.slave);

    let writer = pair
        .master
        .take_writer()
        .map_err(|e| format!("failed to get writer: {e}"))?;

    let mut reader = pair
        .master
        .try_clone_reader()
        .map_err(|e| format!("failed to get reader: {e}"))?;

    // Store session
    {
        let mut sessions = state.sessions.lock().map_err(|e| format!("lock error: {e}"))?;
        sessions.insert(
            id.clone(),
            PtySession {
                master: pair.master,
                writer,
            },
        );
    }

    // Spawn reader thread — streams PTY output to frontend
    let event_id = id.clone();
    let sessions_ref = state.sessions.clone();
    std::thread::spawn(move || {
        let mut buf = [0u8; 4096];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => break, // EOF
                Ok(n) => {
                    let encoded = base64::engine::general_purpose::STANDARD.encode(&buf[..n]);
                    let _ = app_handle.emit(
                        "terminal-output",
                        serde_json::json!({ "id": event_id, "data": encoded }),
                    );
                }
                Err(_) => break,
            }
        }

        // Cleanup on exit
        let mut sessions = sessions_ref.lock().unwrap();
        sessions.remove(&event_id);
        let _ = app_handle.emit(
            "terminal-exit",
            serde_json::json!({ "id": event_id }),
        );
    });

    Ok(())
}

#[tauri::command]
pub fn write_terminal(
    id: String,
    data: String,
    state: tauri::State<PtyState>,
) -> Result<(), String> {
    let mut sessions = state.sessions.lock().map_err(|e| format!("lock error: {e}"))?;
    let session = sessions.get_mut(&id).ok_or("session not found")?;

    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&data)
        .map_err(|e| format!("decode error: {e}"))?;

    session
        .writer
        .write_all(&bytes)
        .map_err(|e| format!("write error: {e}"))?;

    session
        .writer
        .flush()
        .map_err(|e| format!("flush error: {e}"))?;

    Ok(())
}

#[tauri::command]
pub fn resize_terminal(
    id: String,
    cols: u16,
    rows: u16,
    state: tauri::State<PtyState>,
) -> Result<(), String> {
    let sessions = state.sessions.lock().map_err(|e| format!("lock error: {e}"))?;
    let session = sessions.get(&id).ok_or("session not found")?;

    session
        .master
        .resize(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|e| format!("resize error: {e}"))?;

    Ok(())
}

#[tauri::command]
pub fn kill_terminal(
    id: String,
    state: tauri::State<PtyState>,
) -> Result<(), String> {
    let mut sessions = state.sessions.lock().map_err(|e| format!("lock error: {e}"))?;
    sessions.remove(&id);
    Ok(())
}
