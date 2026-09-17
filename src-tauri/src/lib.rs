use tauri::{App, Manager};

mod cmds;
mod core;
mod diff_commands;
mod formatter_commands;
mod help_commands;
mod platform;
mod settings_commands;
mod storage;
mod system_permissions;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        .manage(platform::PasteTargetState::default())
        .manage(platform::MainPanelState::default())
        .manage(core::main_panel_shortcuts::MainPanelShortcutState::default())
        .enable_macos_default_menu(false);
    let builder = builder
        .setup(|app| {
            set_up(app).map_err(|error| {
                Box::new(std::io::Error::other(error)) as Box<dyn std::error::Error>
            })?;
            Ok(())
        })
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build());
    platform::configure_builder(builder)
        .plugin(tauri_plugin_clipboard::init())
        .invoke_handler(tauri::generate_handler![
            cmds::toggle_window,
            cmds::hide_main_panel,
            cmds::show_settings_window,
            cmds::paste_into_active_app,
            diff_commands::show_text_diff,
            diff_commands::get_text_diff_input,
            diff_commands::get_text_diff_pinned,
            diff_commands::set_text_diff_pinned,
            diff_commands::close_text_diff,
            diff_commands::read_diff_file,
            formatter_commands::show_text_formatter,
            formatter_commands::close_text_formatter,
            formatter_commands::get_text_formatter_input,
            formatter_commands::get_text_formatter_pinned,
            formatter_commands::set_text_formatter_pinned,
            help_commands::show_shortcut_help,
            help_commands::hide_shortcut_help,
            settings_commands::get_app_settings,
            settings_commands::update_app_settings,
            settings_commands::update_quick_tools,
            settings_commands::get_active_quick_tools,
            settings_commands::reset_app_settings,
            settings_commands::get_clipboard_history,
            settings_commands::record_clipboard_history,
            settings_commands::get_clipboard_tags,
            settings_commands::create_clipboard_tag,
            settings_commands::update_clipboard_tag,
            settings_commands::delete_clipboard_tag,
            settings_commands::set_clipboard_history_tags,
            settings_commands::delete_clipboard_history,
            settings_commands::clear_clipboard_history,
            settings_commands::get_history_stats,
            system_permissions::get_system_permission_status,
            system_permissions::request_system_permission,
            system_permissions::open_system_permission_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn set_up(app: &mut App) -> Result<(), String> {
    let settings_store = storage::SettingsStore::new(app.handle())?;
    let settings = settings_store.load()?;
    core::handle::Handle::set_app_theme(app.handle(), settings.theme);
    let history_store = storage::HistoryStore::new(app.handle())?;
    let tag_store = storage::TagStore::new(app.handle())?;
    history_store.cleanup_history(&settings)?;
    app.manage(settings_store);
    app.manage(history_store);
    app.manage(tag_store);
    app.manage(storage::SettingsState::new(settings.clone()));
    app.manage(core::formatter_window::TextFormatterState::default());
    app.manage(core::diff_window::DiffState::default());
    core::handle::Handle::create_main_window(app)?;
    core::help_window::create(app)?;
    core::handle::Handle::create_setting_window(app)?;
    core::handle::Handle::register_shortcuts(app, &settings.main_shortcut)?;
    core::handle::Handle::create_tray_icon(app)?;
    platform::finish_setup(app);
    Ok(())
}
