use crate::{
    platform,
    storage::{AppSettings, SettingsState},
};
use std::{collections::HashMap, str::FromStr, sync::Mutex};
use tauri::{AppHandle, Emitter, EventTarget, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

#[derive(Default)]
pub(crate) struct MainPanelShortcutState {
    registered: Mutex<Vec<Shortcut>>,
}

impl MainPanelShortcutState {
    fn replace(&self, shortcuts: Vec<Shortcut>) -> Result<(), String> {
        let mut registered = self
            .registered
            .lock()
            .map_err(|_| "failed to update main panel shortcuts".to_owned())?;
        *registered = shortcuts;
        Ok(())
    }

    fn take(&self) -> Result<Vec<Shortcut>, String> {
        let mut registered = self
            .registered
            .lock()
            .map_err(|_| "failed to access main panel shortcuts".to_owned())?;
        Ok(std::mem::take(&mut *registered))
    }
}

pub(crate) fn register(app_handle: &AppHandle) -> Result<(), String> {
    let settings = app_handle.state::<SettingsState>().get()?;
    let shortcuts = configured_shortcuts(&settings)?;
    unregister(app_handle)?;
    if shortcuts.is_empty() {
        return Ok(());
    }

    let shortcut_indices = shortcut_index_map(&shortcuts);
    let registration = app_handle.global_shortcut().on_shortcuts(
        shortcuts.iter().copied(),
        move |handle, shortcut, event| {
            dispatch_shortcut(handle, shortcut, event, &shortcut_indices);
        },
    );
    if let Err(error) = registration {
        return Err(clean_up_failed_registration(app_handle, &shortcuts, error));
    }
    app_handle
        .state::<MainPanelShortcutState>()
        .replace(shortcuts)
}

pub(crate) fn unregister(app_handle: &AppHandle) -> Result<(), String> {
    let state = app_handle.state::<MainPanelShortcutState>();
    let shortcuts = state.take()?;
    if shortcuts.is_empty() {
        return Ok(());
    }
    if let Err(error) = app_handle
        .global_shortcut()
        .unregister_multiple(shortcuts.iter().copied())
    {
        state.replace(shortcuts)?;
        return Err(format!("failed to unregister main panel shortcuts: {error}"));
    }
    Ok(())
}

pub(crate) fn refresh_if_visible(app_handle: &AppHandle) -> Result<(), String> {
    let window = app_handle
        .get_webview_window(platform::MAIN_PANEL_LABEL)
        .ok_or_else(|| "failed to find main window".to_owned())?;
    if platform::main_panel_is_visible(app_handle, &window)? {
        register(app_handle)?;
    }
    Ok(())
}

fn configured_shortcuts(settings: &AppSettings) -> Result<Vec<Shortcut>, String> {
    settings
        .quick_tool_shortcuts
        .iter()
        .take(settings.quick_tool_ids.len())
        .map(|shortcut| {
            Shortcut::from_str(shortcut)
                .map_err(|error| format!("invalid quick tool shortcut {shortcut}: {error}"))
        })
        .collect()
}

fn shortcut_index_map(shortcuts: &[Shortcut]) -> HashMap<u32, usize> {
    shortcuts
        .iter()
        .enumerate()
        .map(|(index, shortcut)| (shortcut.id(), index))
        .collect()
}

fn dispatch_shortcut(
    app_handle: &AppHandle,
    shortcut: &Shortcut,
    event: tauri_plugin_global_shortcut::ShortcutEvent,
    shortcut_indices: &HashMap<u32, usize>,
) {
    if event.state() != ShortcutState::Pressed {
        return;
    }
    let Some(index) = shortcut_indices.get(&shortcut.id()) else {
        return;
    };
    if let Err(error) = app_handle.emit_to(
        EventTarget::labeled(platform::MAIN_PANEL_LABEL),
        platform::MAIN_PANEL_QUICK_TOOL_SHORTCUT_EVENT,
        *index,
    ) {
        eprintln!("failed to dispatch main panel shortcut: {error}");
    }
}

fn clean_up_failed_registration(
    app_handle: &AppHandle,
    shortcuts: &[Shortcut],
    registration_error: tauri_plugin_global_shortcut::Error,
) -> String {
    match app_handle
        .global_shortcut()
        .unregister_multiple(shortcuts.iter().copied())
    {
        Ok(()) => format!("failed to register main panel shortcuts: {registration_error}"),
        Err(cleanup_error) => format!(
            "failed to register main panel shortcuts: {registration_error}; cleanup failed: {cleanup_error}"
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::{configured_shortcuts, shortcut_index_map};
    use crate::storage::AppSettings;

    #[test]
    fn default_quick_tool_shortcuts_map_to_their_positions() {
        let shortcuts = configured_shortcuts(&AppSettings::default()).unwrap();
        let indices = shortcut_index_map(&shortcuts);

        assert_eq!(shortcuts.len(), 5);
        assert_eq!(indices.get(&shortcuts[0].id()), Some(&0));
        assert_eq!(indices.get(&shortcuts[4].id()), Some(&4));
    }
}
