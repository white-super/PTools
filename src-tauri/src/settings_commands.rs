use crate::{
    core::{
        diff_window::DiffState,
        formatter_window::{TextFormat, TextFormatterState},
        handle::Handle,
        main_panel_shortcuts,
    },
    storage::{
        AppSettings, ClipboardHistoryInput, ClipboardHistoryPage, ClipboardHistoryQuery,
        ClipboardTag, ClipboardTagInput, HistoryStats, HistoryStore, QuickToolId, SettingsState,
        SettingsStore, TagStore,
    },
};
use tauri::{AppHandle, Emitter, Manager};

type CmdResult<T = ()> = Result<T, String>;

const HISTORY_UPDATED_EVENT: &str = "clipboard-history-updated";
const SETTINGS_UPDATED_EVENT: &str = "app-settings-updated";
const TAGS_UPDATED_EVENT: &str = "clipboard-tags-updated";

#[tauri::command]
pub fn get_app_settings(app_handle: AppHandle) -> CmdResult<AppSettings> {
    app_handle.state::<SettingsState>().get()
}

#[tauri::command]
pub fn update_app_settings(app_handle: AppHandle, settings: AppSettings) -> CmdResult<AppSettings> {
    apply_settings(&app_handle, settings)
}

#[tauri::command]
pub fn update_quick_tools(
    app_handle: AppHandle,
    quick_tool_ids: Vec<QuickToolId>,
) -> CmdResult<AppSettings> {
    let settings_state = app_handle.state::<SettingsState>();
    let mut settings = settings_state.get()?;
    settings.quick_tool_ids = quick_tool_ids;
    settings.validate()?;
    app_handle.state::<SettingsStore>().save(&settings)?;
    settings_state.replace(settings.clone())?;
    emit_settings_update(&app_handle, &settings)?;
    refresh_main_panel_shortcuts(&app_handle);
    Ok(settings)
}

#[tauri::command]
pub fn get_active_quick_tools(app_handle: AppHandle) -> CmdResult<Vec<QuickToolId>> {
    let mut tool_ids = app_handle
        .state::<TextFormatterState>()
        .active_formats()?
        .into_iter()
        .map(formatter_quick_tool_id)
        .collect::<Vec<_>>();
    if app_handle.state::<DiffState>().has_windows()? {
        tool_ids.push(QuickToolId::TextDiff);
    }
    tool_ids.sort();
    tool_ids.dedup();
    Ok(tool_ids)
}

#[tauri::command]
pub fn reset_app_settings(app_handle: AppHandle) -> CmdResult<AppSettings> {
    apply_settings(&app_handle, AppSettings::default())
}

#[tauri::command]
pub async fn get_clipboard_history(
    app_handle: AppHandle,
    query: ClipboardHistoryQuery,
) -> CmdResult<ClipboardHistoryPage> {
    tauri::async_runtime::spawn_blocking(move || {
        let settings = app_handle.state::<SettingsState>().get()?;
        app_handle.state::<HistoryStore>().query(&settings, &query)
    })
    .await
    .map_err(|error| format!("failed to query clipboard history in background: {error}"))?
}

#[tauri::command]
pub fn record_clipboard_history(app_handle: AppHandle, entry: ClipboardHistoryInput) -> CmdResult {
    let settings = app_handle.state::<SettingsState>().get()?;
    app_handle
        .state::<HistoryStore>()
        .record(entry, &settings)?;
    emit_history_update(&app_handle)
}

#[tauri::command]
pub fn get_clipboard_tags(app_handle: AppHandle) -> CmdResult<Vec<ClipboardTag>> {
    app_handle.state::<TagStore>().list()
}

#[tauri::command]
pub fn create_clipboard_tag(
    app_handle: AppHandle,
    input: ClipboardTagInput,
) -> CmdResult<Vec<ClipboardTag>> {
    let tags = app_handle.state::<TagStore>().create(input)?;
    emit_tag_update(&app_handle, tags.clone())?;
    Ok(tags)
}

#[tauri::command]
pub fn update_clipboard_tag(
    app_handle: AppHandle,
    id: i64,
    input: ClipboardTagInput,
) -> CmdResult<Vec<ClipboardTag>> {
    let tags = app_handle.state::<TagStore>().update(id, input)?;
    emit_tag_update(&app_handle, tags.clone())?;
    Ok(tags)
}

#[tauri::command]
pub fn delete_clipboard_tag(app_handle: AppHandle, id: i64) -> CmdResult<Vec<ClipboardTag>> {
    let tags = app_handle.state::<TagStore>().delete(id)?;
    emit_tag_update(&app_handle, tags.clone())?;
    emit_history_update(&app_handle)?;
    Ok(tags)
}

#[tauri::command]
pub fn set_clipboard_history_tags(app_handle: AppHandle, id: i64, tag_ids: Vec<i64>) -> CmdResult {
    app_handle
        .state::<TagStore>()
        .set_history_tags(id, &tag_ids)?;
    Ok(())
}

#[tauri::command]
pub fn delete_clipboard_history(app_handle: AppHandle, id: i64) -> CmdResult {
    let settings = app_handle.state::<SettingsState>().get()?;
    app_handle.state::<HistoryStore>().delete(id, &settings)?;
    emit_history_update(&app_handle)
}

#[tauri::command]
pub fn clear_clipboard_history(app_handle: AppHandle) -> CmdResult<HistoryStats> {
    let settings = app_handle.state::<SettingsState>().get()?;
    let history_store = app_handle.state::<HistoryStore>();
    history_store.clear()?;
    let stats = history_store.stats(&settings)?;
    emit_history_update(&app_handle)?;
    Ok(stats)
}

#[tauri::command]
pub fn get_history_stats(app_handle: AppHandle) -> CmdResult<HistoryStats> {
    let settings = app_handle.state::<SettingsState>().get()?;
    app_handle.state::<HistoryStore>().stats(&settings)
}

fn apply_settings(app_handle: &AppHandle, settings: AppSettings) -> CmdResult<AppSettings> {
    settings.validate()?;
    let settings_state = app_handle.state::<SettingsState>();
    let current_settings = settings_state.get()?;
    let shortcut_changed = current_settings.main_shortcut != settings.main_shortcut;
    if shortcut_changed {
        Handle::replace_main_shortcut(
            app_handle,
            &current_settings.main_shortcut,
            &settings.main_shortcut,
        )?;
    }

    let settings_store = app_handle.state::<SettingsStore>();
    if let Err(error) = settings_store.save(&settings) {
        if shortcut_changed {
            Handle::replace_main_shortcut(
                app_handle,
                &settings.main_shortcut,
                &current_settings.main_shortcut,
            )?;
        }
        return Err(error);
    }

    settings_state.replace(settings.clone())?;
    refresh_main_panel_shortcuts(app_handle);
    Handle::set_app_theme(app_handle, settings.theme);
    app_handle
        .state::<HistoryStore>()
        .cleanup_history(&settings)?;
    emit_history_update(app_handle)?;
    emit_settings_update(app_handle, &settings)?;
    Ok(settings)
}

fn emit_settings_update(app_handle: &AppHandle, settings: &AppSettings) -> CmdResult {
    app_handle
        .emit(SETTINGS_UPDATED_EVENT, settings)
        .map_err(|error| format!("failed to notify application windows about settings: {error}"))
}

fn refresh_main_panel_shortcuts(app_handle: &AppHandle) {
    if let Err(error) = main_panel_shortcuts::refresh_if_visible(app_handle) {
        eprintln!("failed to refresh main panel shortcuts: {error}");
    }
}

fn formatter_quick_tool_id(format: TextFormat) -> QuickToolId {
    match format {
        TextFormat::Json => QuickToolId::Json,
        TextFormat::Xml => QuickToolId::Xml,
        TextFormat::Html => QuickToolId::Html,
        TextFormat::Url => QuickToolId::Url,
        TextFormat::Base64 => QuickToolId::Base64,
        TextFormat::Date => QuickToolId::Date,
    }
}

fn emit_history_update(app_handle: &AppHandle) -> CmdResult {
    app_handle
        .emit_to("main", HISTORY_UPDATED_EVENT, ())
        .map_err(|error| format!("failed to notify main panel about history: {error}"))
}

fn emit_tag_update(app_handle: &AppHandle, tags: Vec<ClipboardTag>) -> CmdResult {
    app_handle
        .emit_to("main", TAGS_UPDATED_EVENT, tags)
        .map_err(|error| format!("failed to notify main panel about clipboard tags: {error}"))
}
