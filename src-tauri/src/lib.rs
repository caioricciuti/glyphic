mod commands;
mod paths;
mod pty;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_notification::init())
        .manage(pty::PtyState::default())
        .invoke_handler(tauri::generate_handler![
            // Settings
            commands::settings::read_settings,
            commands::settings::write_settings,
            // Stats
            commands::stats::get_stats,
            commands::stats::compute_live_stats,
            // Projects
            commands::projects::list_projects,
            // Hooks
            commands::hooks::get_hooks,
            commands::hooks::set_hooks,
            // Memory
            commands::memory::list_memory_files,
            commands::memory::read_memory_file,
            commands::memory::write_memory_file,
            commands::memory::delete_memory_file,
            // Instructions
            commands::instructions::read_instructions,
            commands::instructions::write_instructions,
            commands::instructions::read_referenced_file,
            // MCP
            commands::mcp::list_mcp_servers,
            commands::mcp::upsert_mcp_server,
            commands::mcp::delete_mcp_server,
            commands::mcp::get_cloud_mcps,
            // Skills & Agents
            commands::skills::list_skills,
            commands::skills::list_agents,
            commands::skills::write_skill,
            commands::skills::write_agent,
            commands::skills::delete_skill,
            commands::skills::delete_agent,
            // Rules
            commands::rules::list_rules,
            commands::rules::write_rule,
            commands::rules::delete_rule,
            // Plugins
            commands::plugins::get_installed_plugins,
            commands::plugins::get_blocked_plugins,
            commands::plugins::get_marketplace_plugins,
            commands::plugins::get_install_counts,
            commands::plugins::install_plugin,
            // Git
            commands::git::git_status,
            commands::git::git_log,
            commands::git::git_diff,
            commands::git::git_commit,
            commands::git::git_push,
            commands::git::git_pull,
            commands::git::git_branches,
            commands::git::git_checkout,
            commands::git::git_init,
            commands::git::open_in_terminal,
            // Pipelines
            commands::pipelines::list_pipelines,
            commands::pipelines::save_pipeline,
            commands::pipelines::delete_pipeline,
            commands::pipelines::start_pipeline_run,
            commands::pipelines::cancel_pipeline_run,
            commands::pipelines::run_single_node,
            commands::pipelines::resume_pipeline_node,
            // Scheduler
            commands::scheduler::enable_pipeline_schedule,
            commands::scheduler::disable_pipeline_schedule,
            commands::scheduler::list_pipeline_logs,
            // Maintenance
            commands::maintenance::get_disk_usage,
            commands::maintenance::cleanup_directory,
            // Budget
            commands::budget::get_budget,
            commands::budget::set_budget,
            commands::budget::get_cost_summary,
            // Sessions
            commands::sessions::list_sessions,
            commands::sessions::load_session,
            commands::sessions::search_sessions,
            commands::sessions::get_session_tags,
            commands::sessions::set_session_tag,
            commands::sessions::export_session_markdown,
            commands::sessions::detect_live_sessions,
            // Terminal PTY
            pty::spawn_terminal,
            pty::write_terminal,
            pty::resize_terminal,
            pty::kill_terminal,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
