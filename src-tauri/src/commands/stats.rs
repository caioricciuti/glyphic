use crate::paths;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::io::BufRead;

#[tauri::command]
pub fn get_stats() -> Result<serde_json::Value, String> {
    let path = paths::stats_cache_path();

    if !path.exists() {
        return Ok(serde_json::json!({}));
    }

    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("failed to read stats: {e}"))?;

    serde_json::from_str(&content)
        .map_err(|e| format!("failed to parse stats: {e}"))
}

#[derive(Serialize)]
pub struct LiveDailyActivity {
    pub date: String,
    #[serde(rename = "messageCount")]
    pub message_count: u32,
    #[serde(rename = "sessionCount")]
    pub session_count: u32,
    #[serde(rename = "toolCallCount")]
    pub tool_call_count: u32,
}

#[derive(Serialize)]
pub struct LiveStats {
    #[serde(rename = "dailyActivity")]
    pub daily_activity: Vec<LiveDailyActivity>,
    #[serde(rename = "totalSessions")]
    pub total_sessions: u32,
    #[serde(rename = "totalMessages")]
    pub total_messages: u32,
    #[serde(rename = "firstSessionDate")]
    pub first_session_date: String,
    #[serde(rename = "lastSessionDate")]
    pub last_session_date: String,
    #[serde(rename = "hourCounts")]
    pub hour_counts: HashMap<String, u32>,
    // Token data from cache (if available)
    #[serde(rename = "modelUsage")]
    pub model_usage: Option<serde_json::Value>,
    #[serde(rename = "dailyModelTokens")]
    pub daily_model_tokens: Option<serde_json::Value>,
    #[serde(rename = "longestSession")]
    pub longest_session: Option<serde_json::Value>,
    #[serde(rename = "lastComputedDate")]
    pub last_computed_date: String,
}

#[tauri::command]
pub fn compute_live_stats() -> Result<LiveStats, String> {
    let history_path = paths::claude_home().join("history.jsonl");

    if !history_path.exists() {
        return Err("history.jsonl not found".to_string());
    }

    let file = std::fs::File::open(&history_path)
        .map_err(|e| format!("failed to open history: {e}"))?;
    let reader = std::io::BufReader::new(file);

    let mut messages_by_date: HashMap<String, u32> = HashMap::new();
    let mut sessions_by_date: HashMap<String, HashSet<String>> = HashMap::new();
    let mut hour_counts: HashMap<String, u32> = HashMap::new();
    let mut all_sessions: HashSet<String> = HashSet::new();
    let mut total_messages: u32 = 0;
    let mut first_date = String::new();
    let mut last_date = String::new();

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };

        if line.trim().is_empty() {
            continue;
        }

        let entry: serde_json::Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let timestamp = entry.get("timestamp").and_then(|t| t.as_f64()).unwrap_or(0.0);
        let session_id = entry
            .get("sessionId")
            .and_then(|s| s.as_str())
            .unwrap_or("")
            .to_string();

        if timestamp == 0.0 {
            continue;
        }

        // Convert millisecond timestamp to date
        let secs = (timestamp / 1000.0) as i64;
        let dt = chrono_lite_date(secs);
        let date = dt.0.clone();
        let hour = dt.1;

        *messages_by_date.entry(date.clone()).or_insert(0) += 1;
        sessions_by_date
            .entry(date.clone())
            .or_default()
            .insert(session_id.clone());
        *hour_counts.entry(hour.to_string()).or_insert(0) += 1;
        all_sessions.insert(session_id);
        total_messages += 1;

        if first_date.is_empty() || date < first_date {
            first_date = date.clone();
        }
        if last_date.is_empty() || date > last_date {
            last_date = date.clone();
        }
    }

    // Build daily activity sorted by date
    let mut dates: Vec<String> = messages_by_date.keys().cloned().collect();
    dates.sort();

    let daily_activity: Vec<LiveDailyActivity> = dates
        .iter()
        .map(|date| LiveDailyActivity {
            date: date.clone(),
            message_count: *messages_by_date.get(date).unwrap_or(&0),
            session_count: sessions_by_date
                .get(date)
                .map(|s| s.len() as u32)
                .unwrap_or(0),
            tool_call_count: 0, // history.jsonl doesn't track tool calls
        })
        .collect();

    // Read cache for token data (we can't compute this from history.jsonl)
    let cache = get_stats().ok();
    let model_usage = cache
        .as_ref()
        .and_then(|c| c.get("modelUsage").cloned());
    let daily_model_tokens = cache
        .as_ref()
        .and_then(|c| c.get("dailyModelTokens").cloned());
    let longest_session = cache
        .as_ref()
        .and_then(|c| c.get("longestSession").cloned());

    // Merge tool call counts from cache where available
    let mut daily_activity = daily_activity;
    if let Some(cached_stats) = &cache {
        if let Some(cached_daily) = cached_stats.get("dailyActivity").and_then(|d| d.as_array()) {
            let cached_map: HashMap<String, u32> = cached_daily
                .iter()
                .filter_map(|d| {
                    let date = d.get("date")?.as_str()?.to_string();
                    let tool_calls = d.get("toolCallCount")?.as_u64()? as u32;
                    Some((date, tool_calls))
                })
                .collect();

            for activity in &mut daily_activity {
                if let Some(&tool_calls) = cached_map.get(&activity.date) {
                    activity.tool_call_count = tool_calls;
                }
            }
        }
    }

    let today = {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        chrono_lite_date(now).0
    };

    Ok(LiveStats {
        daily_activity,
        total_sessions: all_sessions.len() as u32,
        total_messages,
        first_session_date: first_date,
        last_session_date: last_date,
        hour_counts,
        model_usage,
        daily_model_tokens,
        longest_session,
        last_computed_date: today,
    })
}

/// Simple date extraction from unix timestamp without chrono crate
fn chrono_lite_date(secs: i64) -> (String, u32) {
    // Convert unix timestamp to date components
    let days = secs / 86400;
    let hour = ((secs % 86400) / 3600) as u32;

    // Algorithm from http://howardhinnant.github.io/date_algorithms.html
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };

    (format!("{y:04}-{m:02}-{d:02}"), hour)
}
