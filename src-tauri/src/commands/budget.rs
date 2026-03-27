use crate::paths;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

#[derive(Serialize, Deserialize)]
pub struct BudgetSettings {
    pub daily_limit: Option<f64>,
    pub monthly_limit: Option<f64>,
    #[serde(default = "default_plan")]
    pub plan_type: String,
}

fn default_plan() -> String {
    "max".to_string()
}

impl Default for BudgetSettings {
    fn default() -> Self {
        Self {
            daily_limit: None,
            monthly_limit: None,
            plan_type: default_plan(),
        }
    }
}

#[derive(Serialize)]
pub struct CostSummary {
    pub today: f64,
    pub this_month: f64,
    pub last_7_days: Vec<f64>,
    pub daily_limit: Option<f64>,
    pub monthly_limit: Option<f64>,
    pub daily_exceeded: bool,
    pub monthly_exceeded: bool,
    pub monthly_projection: f64,
    pub per_project_month: Vec<ProjectCost>,
    pub plan_type: String,
}

#[derive(Serialize)]
pub struct ProjectCost {
    pub project: String,
    pub cost: f64,
    pub messages: u32,
}

// Pricing per million tokens
fn price_input(model: &str) -> f64 {
    if model.contains("haiku") { 1.0 }
    else if model.contains("sonnet") { 3.0 }
    else { 15.0 } // opus
}

fn price_output(model: &str) -> f64 {
    if model.contains("haiku") { 5.0 }
    else if model.contains("sonnet") { 15.0 }
    else { 75.0 }
}

fn price_cache_read(model: &str) -> f64 {
    if model.contains("haiku") { 0.1 }
    else if model.contains("sonnet") { 0.3 }
    else { 1.5 }
}

fn _price_cache_write(model: &str) -> f64 {
    if model.contains("haiku") { 1.25 }
    else if model.contains("sonnet") { 3.75 }
    else { 18.75 }
}

fn glyphic_settings_path() -> std::path::PathBuf {
    paths::claude_home().join("glyphic-settings.json")
}

#[tauri::command]
pub fn get_budget() -> Result<BudgetSettings, String> {
    let path = glyphic_settings_path();
    if !path.exists() {
        return Ok(BudgetSettings::default());
    }
    let content = fs::read_to_string(&path)
        .map_err(|e| format!("failed to read: {e}"))?;
    serde_json::from_str(&content)
        .map_err(|e| format!("failed to parse: {e}"))
}

#[tauri::command]
pub fn set_budget(daily_limit: Option<f64>, monthly_limit: Option<f64>, plan_type: Option<String>) -> Result<(), String> {
    let existing = get_budget().unwrap_or_default();
    let settings = BudgetSettings {
        daily_limit,
        monthly_limit,
        plan_type: plan_type.unwrap_or(existing.plan_type),
    };
    let content = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("failed to serialize: {e}"))?;
    fs::write(glyphic_settings_path(), content)
        .map_err(|e| format!("failed to write: {e}"))
}

#[tauri::command]
pub fn get_cost_summary() -> Result<CostSummary, String> {
    let cache_path = paths::stats_cache_path();

    let mut daily_model_tokens: Vec<(String, HashMap<String, f64>)> = Vec::new();
    let mut model_usage: HashMap<String, serde_json::Value> = HashMap::new();

    if cache_path.exists() {
        let content = fs::read_to_string(&cache_path)
            .map_err(|e| format!("failed to read cache: {e}"))?;
        let data: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| format!("failed to parse: {e}"))?;

        if let Some(dmt) = data.get("dailyModelTokens").and_then(|d| d.as_array()) {
            for entry in dmt {
                let date = entry.get("date").and_then(|d| d.as_str()).unwrap_or("").to_string();
                let tokens = entry.get("tokensByModel").and_then(|t| t.as_object());
                if let Some(tokens) = tokens {
                    let mut map = HashMap::new();
                    for (model, count) in tokens {
                        map.insert(model.clone(), count.as_f64().unwrap_or(0.0));
                    }
                    daily_model_tokens.push((date, map));
                }
            }
        }

        if let Some(mu) = data.get("modelUsage").and_then(|m| m.as_object()) {
            for (k, v) in mu {
                model_usage.insert(k.clone(), v.clone());
            }
        }
    }

    // Compute costs per day
    let today = {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        // Simple date computation
        let days = now / 86400;
        let era = if days + 719468 >= 0 { days + 719468 } else { days + 719468 - 146096 } / 146097;
        let doe = ((days + 719468) - era * 146097) as u32;
        let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
        let y = yoe as i64 + era * 400;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let d = doy - (153 * mp + 2) / 5 + 1;
        let m = if mp < 10 { mp + 3 } else { mp - 9 };
        let y = if m <= 2 { y + 1 } else { y };
        format!("{y:04}-{m:02}-{d:02}")
    };

    let month_prefix = &today[..7]; // "2026-03"

    fn day_cost(tokens: &HashMap<String, f64>) -> f64 {
        // daily model tokens only tracks output tokens roughly
        // use a simple estimate: avg cost per token across models
        tokens.iter().map(|(model, count)| {
            // These are total tokens for the day, roughly split
            let per_m = price_output(model) * 0.3 + price_input(model) * 0.1 + price_cache_read(model) * 0.6;
            (count / 1_000_000.0) * per_m
        }).sum()
    }

    let today_cost = daily_model_tokens.iter()
        .find(|(d, _)| d == &today)
        .map(|(_, t)| day_cost(t))
        .unwrap_or(0.0);

    let month_cost: f64 = daily_model_tokens.iter()
        .filter(|(d, _)| d.starts_with(month_prefix))
        .map(|(_, t)| day_cost(t))
        .sum();

    // Last 7 days
    let last_7: Vec<f64> = daily_model_tokens.iter()
        .rev()
        .take(7)
        .map(|(_, t)| day_cost(t))
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();

    // Budget
    let budget = get_budget().unwrap_or_default();

    // Monthly projection
    let day_of_month: f64 = today[8..].parse().unwrap_or(1.0);
    let monthly_projection = if day_of_month > 0.0 { month_cost / day_of_month * 30.0 } else { 0.0 };

    // Per-project costs from history.jsonl
    let mut project_costs: HashMap<String, (f64, u32)> = HashMap::new();
    let history_path = paths::claude_home().join("history.jsonl");
    if history_path.exists() {
        if let Ok(file) = fs::File::open(&history_path) {
            use std::io::BufRead;
            let reader = std::io::BufReader::new(file);
            for line in reader.lines() {
                let line = match line { Ok(l) => l, Err(_) => continue };
                let entry: serde_json::Value = match serde_json::from_str(&line) { Ok(v) => v, Err(_) => continue };
                let ts = entry.get("timestamp").and_then(|t| t.as_f64()).unwrap_or(0.0);
                let secs = (ts / 1000.0) as i64;
                // Quick date check for this month
                let entry_days = secs / 86400;
                let today_days = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64 / 86400;
                if today_days - entry_days > 30 { continue; }

                let project = entry.get("project").and_then(|p| p.as_str()).unwrap_or("unknown");
                let name = project.split('/').last().unwrap_or(project);
                let entry = project_costs.entry(name.to_string()).or_insert((0.0, 0));
                entry.0 += 0.05; // rough cost estimate per message
                entry.1 += 1;
            }
        }
    }

    let mut per_project_month: Vec<ProjectCost> = project_costs.into_iter()
        .map(|(project, (cost, messages))| ProjectCost { project, cost, messages })
        .collect();
    per_project_month.sort_by(|a, b| b.cost.partial_cmp(&a.cost).unwrap_or(std::cmp::Ordering::Equal));

    Ok(CostSummary {
        today: today_cost,
        this_month: month_cost,
        last_7_days: last_7,
        daily_limit: budget.daily_limit,
        monthly_limit: budget.monthly_limit,
        daily_exceeded: budget.daily_limit.map_or(false, |l| today_cost >= l),
        monthly_exceeded: budget.monthly_limit.map_or(false, |l| month_cost >= l),
        monthly_projection,
        per_project_month,
        plan_type: budget.plan_type,
    })
}
