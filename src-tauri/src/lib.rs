mod commands;
pub mod ctx;
pub mod filter;
mod paths;
mod pty;

use tauri::{
    image::Image,
    menu::{MenuBuilder, MenuItemBuilder},
    tray::TrayIconBuilder,
    Manager, WindowEvent,
};

/// Show the main window and restore Dock/Cmd+Tab presence on macOS.
fn show_window(window: &tauri::WebviewWindow) {
    #[cfg(target_os = "macos")]
    {
        use cocoa::appkit::{NSApp, NSApplication, NSApplicationActivationPolicy};
        unsafe {
            NSApp().setActivationPolicy_(
                NSApplicationActivationPolicy::NSApplicationActivationPolicyRegular,
            );
            NSApp().activateIgnoringOtherApps_(true);
        }
    }
    let _ = window.show();
    let _ = window.set_focus();
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_notification::init())
        .manage(pty::PtyState::default())
        .setup(|app| {
            // Build tray menu
            let show = MenuItemBuilder::with_id("show", "Show Glyphic").build(app)?;
            let quit = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
            let menu = MenuBuilder::new(app).item(&show).separator().item(&quit).build()?;

            // Create tray icon
            let icon = Image::from_path("icons/32x32.png")
                .or_else(|_| Image::from_bytes(include_bytes!("../icons/32x32.png")))
                .expect("failed to load tray icon");

            TrayIconBuilder::new()
                .icon(icon)
                .menu(&menu)
                .tooltip("Glyphic")
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            show_window(&window);
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::Click { .. } = event {
                        if let Some(window) = tray.app_handle().get_webview_window("main") {
                            show_window(&window);
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| {
            // Hide window instead of closing — keeps app in tray
            if let WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
                // Remove from Dock and Cmd+Tab on macOS
                #[cfg(target_os = "macos")]
                {
                    use cocoa::appkit::{NSApp, NSApplication, NSApplicationActivationPolicy};
                    unsafe {
                        NSApp().setActivationPolicy_(
                            NSApplicationActivationPolicy::NSApplicationActivationPolicyAccessory,
                        );
                    }
                }
            }
        })
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
            commands::pipelines::list_pipeline_history,
            commands::pipelines::delete_pipeline_history,
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
            // Token Savings
            commands::token_savings::get_optimizer_status,
            commands::token_savings::enable_optimizer,
            commands::token_savings::disable_optimizer,
            commands::token_savings::get_savings_data,
            commands::token_savings::discover_opportunities,
            commands::token_savings::get_filter_rules,
            commands::token_savings::save_filter_rules,
            // Keybindings
            commands::keybindings::read_keybindings,
            commands::keybindings::write_keybindings,
            commands::keybindings::get_default_keybindings,
            // Context Engine
            commands::context_engine::ctx_get_status,
            commands::context_engine::ctx_enable,
            commands::context_engine::ctx_disable,
            commands::context_engine::ctx_recent_tool_results,
            commands::context_engine::ctx_reindex_embeddings,
            commands::context_engine::ctx_purge_legacy,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
