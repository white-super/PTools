use crate::{
    cmds,
    core::{
        sequential_paste::{
            SequentialPasteDirection, SequentialPasteMode, SequentialPasteSnapshot,
            SequentialPasteState,
        },
        sequential_paste_shortcut, sequential_paste_window, window_task,
    },
    platform,
    storage::{ClipboardFormat, ClipboardHistoryInput, SettingsState},
    system_permissions,
};
use std::path::Path;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_clipboard::Clipboard;

pub const STATE_CHANGED_EVENT: &str = "sequential-paste-state-changed";
type CommandResult<T = ()> = Result<T, String>;

#[tauri::command]
pub async fn show_sequential_paste(
    app_handle: AppHandle,
) -> CommandResult<SequentialPasteSnapshot> {
    window_task::run(app_handle, |handle| {
        cmds::hide_main_panel_now(handle)?;
        let state = handle.state::<SequentialPasteState>();
        state.enable();
        if let Err(error) = sequential_paste_window::show(handle) {
            state.disable_and_clear();
            return Err(error);
        }
        emit_state(handle)?;
        Ok(state.snapshot())
    })
    .await
}

#[tauri::command]
pub fn get_sequential_paste_state(app_handle: AppHandle) -> SequentialPasteSnapshot {
    app_handle.state::<SequentialPasteState>().snapshot()
}

#[tauri::command]
pub fn set_sequential_paste_mode(
    app_handle: AppHandle,
    mode: SequentialPasteMode,
) -> CommandResult<SequentialPasteSnapshot> {
    let state = app_handle.state::<SequentialPasteState>();
    let _operation = state.lock_operation();
    match mode {
        SequentialPasteMode::Capture => switch_to_capture(&app_handle, &state)?,
        SequentialPasteMode::Paste => switch_to_paste(&app_handle, &state)?,
    }
    emit_and_snapshot(&app_handle, &state)
}

#[tauri::command]
pub fn set_sequential_paste_direction(
    app_handle: AppHandle,
    direction: SequentialPasteDirection,
) -> CommandResult<SequentialPasteSnapshot> {
    let state = app_handle.state::<SequentialPasteState>();
    let _operation = state.lock_operation();
    state.set_direction(direction)?;
    emit_and_snapshot(&app_handle, &state)
}

#[tauri::command]
pub fn reorder_sequential_paste(
    app_handle: AppHandle,
    item_ids: Vec<u64>,
) -> CommandResult<SequentialPasteSnapshot> {
    let state = app_handle.state::<SequentialPasteState>();
    let _operation = state.lock_operation();
    state.reorder(&item_ids)?;
    emit_and_snapshot(&app_handle, &state)
}

#[tauri::command]
pub fn remove_sequential_paste_item(
    app_handle: AppHandle,
    item_id: u64,
) -> CommandResult<SequentialPasteSnapshot> {
    let state = app_handle.state::<SequentialPasteState>();
    let _operation = state.lock_operation();
    state.remove(item_id)?;
    emit_and_snapshot(&app_handle, &state)
}

#[tauri::command]
pub fn clear_sequential_paste(app_handle: AppHandle) -> CommandResult<SequentialPasteSnapshot> {
    let state = app_handle.state::<SequentialPasteState>();
    let _operation = state.lock_operation();
    state.clear()?;
    emit_and_snapshot(&app_handle, &state)
}

#[tauri::command]
pub async fn close_sequential_paste(app_handle: AppHandle) -> CommandResult {
    {
        let state = app_handle.state::<SequentialPasteState>();
        let _operation = state.lock_operation();
        sequential_paste_shortcut::unregister(&app_handle)?;
        state.disable_and_clear();
        emit_state(&app_handle)?;
    }
    window_task::run(app_handle, sequential_paste_window::hide).await
}

#[tauri::command]
pub fn write_clipboard_entry(app_handle: AppHandle, entry: ClipboardHistoryInput) -> CommandResult {
    entry.validate()?;
    write_internal_clipboard(&app_handle, entry)
}

pub fn paste_next(app_handle: &AppHandle) -> CommandResult {
    let state = app_handle.state::<SequentialPasteState>();
    let _operation = state.lock_operation();
    let item = state.next_item()?;
    let result = perform_paste(app_handle, item.clipboard_input());
    if let Err(error) = result {
        state.set_error(Some(item.id), error.clone());
        emit_state(app_handle)?;
        return Err(error);
    }
    let queue_empty = state.complete_item(item.id)?;
    if queue_empty {
        if let Err(error) = sequential_paste_shortcut::unregister(app_handle) {
            state.set_error(None, error.clone());
            emit_state(app_handle)?;
            return Err(error);
        }
    }
    emit_state(app_handle)
}

pub fn emit_state(app_handle: &AppHandle) -> CommandResult {
    let snapshot = app_handle.state::<SequentialPasteState>().snapshot();
    app_handle
        .emit(STATE_CHANGED_EVENT, snapshot)
        .map_err(|error| format!("failed to emit sequential paste state: {error}"))
}

fn switch_to_capture(app_handle: &AppHandle, state: &SequentialPasteState) -> CommandResult {
    sequential_paste_shortcut::unregister(app_handle)?;
    state.set_mode(SequentialPasteMode::Capture)
}

fn switch_to_paste(app_handle: &AppHandle, state: &SequentialPasteState) -> CommandResult {
    system_permissions::ensure_automatic_paste_permission()?;
    state.set_mode(SequentialPasteMode::Paste)?;
    let settings = app_handle.state::<SettingsState>().get()?;
    if let Err(error) =
        sequential_paste_shortcut::register(app_handle, &settings.sequential_paste_shortcut)
    {
        let _ = state.set_mode(SequentialPasteMode::Capture);
        return Err(error);
    }
    Ok(())
}

fn perform_paste(app_handle: &AppHandle, entry: ClipboardHistoryInput) -> CommandResult {
    system_permissions::ensure_automatic_paste_permission()?;
    platform::remember_frontmost_application(app_handle)?;
    let target = app_handle.state::<platform::PasteTargetState>().get()?;
    write_internal_clipboard(app_handle, entry)?;
    platform::post_paste_shortcut(target)
}

fn write_internal_clipboard(app_handle: &AppHandle, entry: ClipboardHistoryInput) -> CommandResult {
    let state = app_handle.state::<SequentialPasteState>();
    let token = state.expect_internal_write(entry.clone());
    if let Err(error) = write_clipboard(app_handle, entry) {
        state.cancel_internal_write(token);
        return Err(error);
    }
    Ok(())
}

fn write_clipboard(app_handle: &AppHandle, entry: ClipboardHistoryInput) -> CommandResult {
    let clipboard = app_handle.state::<Clipboard>();
    match entry.format {
        ClipboardFormat::Text => clipboard.write_text(entry.content),
        ClipboardFormat::Image => clipboard.write_image_base64(entry.content),
        ClipboardFormat::File => clipboard.write_files_uris(file_uris(&entry.file_paths)?),
    }
}

fn file_uris(file_paths: &[String]) -> CommandResult<Vec<String>> {
    for path in file_paths {
        if !Path::new(path).exists() {
            return Err(format!("文件不存在：{path}"));
        }
    }
    #[cfg(any(target_os = "macos", target_os = "linux"))]
    return Ok(file_paths
        .iter()
        .map(|path| format!("file://{path}"))
        .collect());
    #[cfg(target_os = "windows")]
    return Ok(file_paths.to_vec());
    #[allow(unreachable_code)]
    Ok(file_paths.to_vec())
}

fn emit_and_snapshot(
    app_handle: &AppHandle,
    state: &SequentialPasteState,
) -> CommandResult<SequentialPasteSnapshot> {
    emit_state(app_handle)?;
    Ok(state.snapshot())
}
