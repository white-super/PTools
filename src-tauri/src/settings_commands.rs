use crate::{
    core::handle::Handle,
    storage::{
        AppSettings, ClipboardHistoryInput, ClipboardHistoryPage, ClipboardHistoryQuery,
        ClipboardTag, ClipboardTagInput, HistoryStats, HistoryStore, SettingsState, SettingsStore,
        TagStore,
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
    Handle::set_app_theme(app_handle, settings.theme);
    app_handle
        .state::<HistoryStore>()
        .cleanup_history(&settings)?;
    emit_history_update(app_handle)?;
    app_handle
        .emit(SETTINGS_UPDATED_EVENT, &settings)
        .map_err(|error| format!("failed to notify application windows about settings: {error}"))?;
    Ok(settings)
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
