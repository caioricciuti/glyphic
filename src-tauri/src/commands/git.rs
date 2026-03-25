use serde::Serialize;
use std::process::Command;

#[derive(Serialize)]
pub struct GitStatus {
    pub branch: String,
    pub is_repo: bool,
    pub clean: bool,
    pub files: Vec<GitFileChange>,
    pub ahead: i32,
    pub behind: i32,
}

#[derive(Serialize)]
pub struct GitFileChange {
    pub status: String,
    pub path: String,
}

#[derive(Serialize)]
pub struct GitLogEntry {
    pub hash: String,
    pub message: String,
    pub author: String,
    pub date: String,
}

fn run_git(path: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(path)
        .output()
        .map_err(|e| format!("failed to run git: {e}"))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(format!("git error: {stderr}"))
    }
}

#[tauri::command]
pub fn git_status(path: String) -> Result<GitStatus, String> {
    // Check if it's a git repo
    if run_git(&path, &["rev-parse", "--git-dir"]).is_err() {
        return Ok(GitStatus {
            branch: String::new(),
            is_repo: false,
            clean: true,
            files: vec![],
            ahead: 0,
            behind: 0,
        });
    }

    let branch = run_git(&path, &["branch", "--show-current"]).unwrap_or_default();

    let porcelain = run_git(&path, &["status", "--porcelain"]).unwrap_or_default();
    let files: Vec<GitFileChange> = porcelain
        .lines()
        .filter(|l| !l.is_empty())
        .map(|line| {
            let status = line[..2].trim().to_string();
            let file_path = line[3..].to_string();
            GitFileChange {
                status,
                path: file_path,
            }
        })
        .collect();

    let clean = files.is_empty();

    // Get ahead/behind
    let mut ahead = 0;
    let mut behind = 0;
    if let Ok(ab) = run_git(&path, &["rev-list", "--left-right", "--count", "HEAD...@{upstream}"]) {
        let parts: Vec<&str> = ab.split('\t').collect();
        if parts.len() == 2 {
            ahead = parts[0].parse().unwrap_or(0);
            behind = parts[1].parse().unwrap_or(0);
        }
    }

    Ok(GitStatus {
        branch,
        is_repo: true,
        clean,
        files,
        ahead,
        behind,
    })
}

#[tauri::command]
pub fn git_log(path: String, count: Option<u32>) -> Result<Vec<GitLogEntry>, String> {
    let n = count.unwrap_or(20).to_string();
    let format = "--format=%H%n%s%n%an%n%ar";
    let output = run_git(&path, &["log", &format!("-{n}"), format])?;

    let lines: Vec<&str> = output.lines().collect();
    let mut entries = Vec::new();

    for chunk in lines.chunks(4) {
        if chunk.len() >= 4 {
            entries.push(GitLogEntry {
                hash: chunk[0].to_string(),
                message: chunk[1].to_string(),
                author: chunk[2].to_string(),
                date: chunk[3].to_string(),
            });
        }
    }

    Ok(entries)
}

#[tauri::command]
pub fn git_diff(path: String) -> Result<String, String> {
    run_git(&path, &["diff", "--stat"])
}

#[tauri::command]
pub fn git_commit(path: String, message: String) -> Result<String, String> {
    run_git(&path, &["add", "-A"])?;
    run_git(&path, &["commit", "-m", &message])
}

#[tauri::command]
pub fn git_push(path: String) -> Result<String, String> {
    run_git(&path, &["push"])
}

#[tauri::command]
pub fn git_pull(path: String) -> Result<String, String> {
    run_git(&path, &["pull"])
}

#[tauri::command]
pub fn git_branches(path: String) -> Result<Vec<String>, String> {
    let output = run_git(&path, &["branch", "-a", "--format=%(refname:short)"])?;
    Ok(output.lines().map(|l| l.to_string()).collect())
}

#[tauri::command]
pub fn git_checkout(path: String, branch: String) -> Result<String, String> {
    run_git(&path, &["checkout", &branch])
}

#[tauri::command]
pub fn git_init(path: String) -> Result<String, String> {
    run_git(&path, &["init"])
}

#[tauri::command]
pub fn open_in_terminal(path: String) -> Result<(), String> {
    // macOS: open Terminal.app with a script that cd's to the path and runs claude
    let script = format!("cd '{}' && claude", path);
    Command::new("osascript")
        .args([
            "-e",
            &format!(
                "tell application \"Terminal\"
                    activate
                    do script \"{}\"
                end tell",
                script.replace('"', "\\\"")
            ),
        ])
        .spawn()
        .map_err(|e| format!("failed to open terminal: {e}"))?;
    Ok(())
}
