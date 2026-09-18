use parking_lot::Mutex;
use std::{
    str::FromStr,
    sync::atomic::{AtomicBool, Ordering},
};
use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

#[derive(Default)]
pub struct SequentialPasteShortcutState {
    registered: Mutex<Option<Shortcut>>,
    pressed: AtomicBool,
}

pub fn register(app_handle: &AppHandle, shortcut: &str) -> Result<(), String> {
    let parsed = parse_shortcut(shortcut)?;
    let state = app_handle.state::<SequentialPasteShortcutState>();
    if state.registered.lock().is_some() {
        return Ok(());
    }
    register_handler(app_handle, parsed)?;
    *state.registered.lock() = Some(parsed);
    Ok(())
}

pub fn unregister(app_handle: &AppHandle) -> Result<(), String> {
    let state = app_handle.state::<SequentialPasteShortcutState>();
    let registered = state.registered.lock().take();
    let Some(shortcut) = registered else {
        return Ok(());
    };
    if let Err(error) = app_handle.global_shortcut().unregister(shortcut) {
        *state.registered.lock() = Some(shortcut);
        return Err(format!(
            "failed to unregister sequential paste shortcut: {error}"
        ));
    }
    state.pressed.store(false, Ordering::Release);
    Ok(())
}

pub fn replace_if_registered(
    app_handle: &AppHandle,
    current: &str,
    next: &str,
) -> Result<(), String> {
    let state = app_handle.state::<SequentialPasteShortcutState>();
    if state.registered.lock().is_none() {
        return Ok(());
    }
    let current = parse_shortcut(current)?;
    let next = parse_shortcut(next)?;
    if current == next {
        return Ok(());
    }
    app_handle
        .global_shortcut()
        .unregister(current)
        .map_err(|error| format!("failed to unregister sequential paste shortcut: {error}"))?;
    *state.registered.lock() = None;
    if let Err(error) = register_handler(app_handle, next) {
        let rollback = register_handler(app_handle, current);
        if rollback.is_ok() {
            *state.registered.lock() = Some(current);
        }
        state.pressed.store(false, Ordering::Release);
        return Err(rollback_error(error, rollback));
    }
    *state.registered.lock() = Some(next);
    state.pressed.store(false, Ordering::Release);
    Ok(())
}

fn parse_shortcut(value: &str) -> Result<Shortcut, String> {
    let shortcut =
        Shortcut::from_str(value).map_err(|error| format!("顺序粘贴快捷键格式无效：{error}"))?;
    if shortcut.mods.is_empty() {
        return Err("顺序粘贴快捷键必须包含至少一个修饰键".to_owned());
    }
    Ok(shortcut)
}

fn register_handler(app_handle: &AppHandle, shortcut: Shortcut) -> Result<(), String> {
    app_handle
        .global_shortcut()
        .on_shortcut(shortcut, move |handle, _, event| {
            dispatch_shortcut(handle, event.state());
        })
        .map_err(|error| format!("failed to register sequential paste shortcut: {error}"))
}

fn dispatch_shortcut(app_handle: &AppHandle, state: ShortcutState) {
    let shortcut_state = app_handle.state::<SequentialPasteShortcutState>();
    if state == ShortcutState::Released {
        shortcut_state.pressed.store(false, Ordering::Release);
        return;
    }
    if shortcut_state.pressed.swap(true, Ordering::AcqRel) {
        return;
    }
    let handle = app_handle.clone();
    tauri::async_runtime::spawn_blocking(move || {
        if let Err(error) = crate::sequential_paste_commands::paste_next(&handle) {
            eprintln!("failed to perform sequential paste: {error}");
        }
    });
}

fn rollback_error(error: String, rollback: Result<(), String>) -> String {
    match rollback {
        Ok(()) => error,
        Err(rollback_error) => format!("{error}; failed to restore shortcut: {rollback_error}"),
    }
}
