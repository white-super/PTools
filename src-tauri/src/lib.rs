use tauri::{App, Manager};

mod cmds;
mod core;
mod settings_commands;
mod storage;
mod system_permissions;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(cmds::PasteTargetProcessId::default())
        .enable_macos_default_menu(false)
        .setup(|app| {
            set_up(app).map_err(|error| {
                Box::new(std::io::Error::other(error)) as Box<dyn std::error::Error>
            })?;
            Ok(())
        })
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_nspanel::init())
        .plugin(tauri_plugin_clipboard::init())
        .invoke_handler(tauri::generate_handler![
            cmds::toggle_window,
            cmds::hide_main_panel,
            cmds::paste_into_active_app,
            settings_commands::get_app_settings,
            settings_commands::update_app_settings,
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
    let history_store = storage::HistoryStore::new(app.handle())?;
    let tag_store = storage::TagStore::new(app.handle())?;
    history_store.list(&settings)?;
    app.manage(settings_store);
    app.manage(history_store);
    app.manage(tag_store);
    app.manage(storage::SettingsState::new(settings.clone()));
    core::handle::Handle::create_main_window(app)?;
    core::handle::Handle::create_setting_window(app)?;
    core::handle::Handle::register_shortcuts(app, &settings.main_shortcut)?;
    core::handle::Handle::create_tray_icon(app)?;
    app.set_activation_policy(tauri::ActivationPolicy::Accessory);
    Ok(())
}
