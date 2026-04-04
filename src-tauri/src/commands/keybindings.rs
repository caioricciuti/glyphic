use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Serialize, Deserialize, Clone)]
pub struct KeybindingEntry {
    pub key: String,
    pub command: String,
    pub description: String,
    #[serde(default)]
    pub when: Option<String>,
}

fn keybindings_path() -> std::path::PathBuf {
    let home = dirs::home_dir().expect("no home dir");
    home.join(".claude").join("keybindings.json")
}

#[tauri::command]
pub fn read_keybindings() -> Result<Vec<KeybindingEntry>, String> {
    let path = keybindings_path();
    if !path.exists() {
        return Ok(vec![]);
    }
    let content = fs::read_to_string(&path)
        .map_err(|e| format!("failed to read keybindings: {e}"))?;
    let bindings: Vec<KeybindingEntry> = serde_json::from_str(&content)
        .map_err(|e| format!("failed to parse keybindings: {e}"))?;
    Ok(bindings)
}

#[tauri::command]
pub fn write_keybindings(bindings: Vec<KeybindingEntry>) -> Result<(), String> {
    let path = keybindings_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("failed to create dir: {e}"))?;
    }
    let json = serde_json::to_string_pretty(&bindings)
        .map_err(|e| format!("failed to serialize: {e}"))?;
    fs::write(&path, json)
        .map_err(|e| format!("failed to write keybindings: {e}"))
}

#[tauri::command]
pub fn get_default_keybindings() -> Vec<KeybindingEntry> {
    vec![
        KeybindingEntry {
            key: "Enter".to_string(),
            command: "submit".to_string(),
            description: "Submit prompt".to_string(),
            when: None,
        },
        KeybindingEntry {
            key: "Escape".to_string(),
            command: "interrupt".to_string(),
            description: "Interrupt Claude".to_string(),
            when: Some("isGenerating".to_string()),
        },
        KeybindingEntry {
            key: "Escape Escape".to_string(),
            command: "undo_last_tool_use".to_string(),
            description: "Undo last tool use".to_string(),
            when: None,
        },
        KeybindingEntry {
            key: "Ctrl+C".to_string(),
            command: "interrupt".to_string(),
            description: "Interrupt (alternative)".to_string(),
            when: Some("isGenerating".to_string()),
        },
        KeybindingEntry {
            key: "Ctrl+L".to_string(),
            command: "clear".to_string(),
            description: "Clear conversation".to_string(),
            when: None,
        },
        KeybindingEntry {
            key: "Ctrl+J".to_string(),
            command: "newline".to_string(),
            description: "Insert newline".to_string(),
            when: None,
        },
        KeybindingEntry {
            key: "Shift+Tab".to_string(),
            command: "cycle_permission_mode".to_string(),
            description: "Cycle permission mode".to_string(),
            when: None,
        },
        KeybindingEntry {
            key: "Up".to_string(),
            command: "history_prev".to_string(),
            description: "Previous message".to_string(),
            when: Some("isInputEmpty".to_string()),
        },
        KeybindingEntry {
            key: "Down".to_string(),
            command: "history_next".to_string(),
            description: "Next message".to_string(),
            when: Some("isInputEmpty".to_string()),
        },
        KeybindingEntry {
            key: "/".to_string(),
            command: "search_transcript".to_string(),
            description: "Search transcript".to_string(),
            when: Some("isTranscriptMode".to_string()),
        },
    ]
}
