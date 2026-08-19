#[cfg(not(target_os = "windows"))]
use super::apply_main_panel_layout;
use super::{
    MainPanelState, MAIN_PANEL_BLUR_EVENT, MAIN_PANEL_FOCUS_EVENT, MAIN_PANEL_LABEL,
    WINDOW_MOVED_EVENT, WINDOW_RESIZED_EVENT,
};
#[cfg(not(target_os = "windows"))]
use tauri::image::Image;
#[cfg(not(target_os = "windows"))]
use tauri::tray::TrayIconEvent;
#[cfg(not(target_os = "windows"))]
use tauri::PhysicalPosition;
use tauri::{
    App, AppHandle, Builder, Emitter, EventTarget, Manager, Runtime, WebviewWindow,
    WebviewWindowBuilder, WindowEvent, Wry,
};

pub(crate) const MAIN_PANEL_INITIAL_WIDTH: f64 = 1100.0;
pub(crate) const MAIN_PANEL_INITIAL_HEIGHT: f64 = 320.0;
#[cfg(not(target_os = "windows"))]
pub(crate) const TRAY_ICON: Image<'_> = tauri::include_image!("./icons/menu-bar-template.png");
#[cfg(not(target_os = "windows"))]
pub(crate) const TRAY_ICON_IS_TEMPLATE: bool = true;
#[cfg(not(target_os = "windows"))]
pub(crate) const TRAY_MENU_ON_LEFT_CLICK: bool = true;

#[cfg(not(target_os = "windows"))]
pub(crate) fn tray_event_opens_settings(_event: &TrayIconEvent) -> bool {
    false
}

pub(crate) fn configure_builder(builder: Builder<Wry>) -> Builder<Wry> {
    builder
}

pub(crate) fn configure_main_window_builder<'a, R, M>(
    builder: WebviewWindowBuilder<'a, R, M>,
) -> WebviewWindowBuilder<'a, R, M>
where
    R: Runtime,
    M: Manager<R>,
{
    builder
}

pub(crate) fn initialize_main_panel(app: &mut App, window: WebviewWindow) -> Result<(), String> {
    let app_handle = app.handle().clone();
    window.on_window_event(move |event| {
        let target = EventTarget::labeled(MAIN_PANEL_LABEL);
        match event {
            WindowEvent::Focused(true) => {
                let _ = app_handle.emit_to(target, MAIN_PANEL_FOCUS_EVENT, true);
            }
            WindowEvent::Focused(false) => {
                let panel_state = app_handle.state::<MainPanelState>();
                if !panel_state.take_blur_suppression() {
                    let _ = app_handle.emit_to(target, MAIN_PANEL_BLUR_EVENT, true);
                }
            }
            WindowEvent::Moved(position) => {
                let _ = app_handle.emit_to(target, WINDOW_MOVED_EVENT, position);
            }
            WindowEvent::Resized(size) => {
                let _ = app_handle.emit_to(target, WINDOW_RESIZED_EVENT, size);
            }
            _ => (),
        }
    });
    Ok(())
}

pub(crate) fn main_panel_is_visible(
    _app_handle: &AppHandle,
    window: &WebviewWindow,
) -> Result<bool, String> {
    window
        .is_visible()
        .map_err(|error| format!("failed to read main panel visibility: {error}"))
}

#[cfg(not(target_os = "windows"))]
pub(crate) fn show_main_panel(
    _app_handle: &AppHandle,
    window: &WebviewWindow,
    cursor_position: PhysicalPosition<f64>,
) -> Result<(), String> {
    window
        .show()
        .map_err(|error| format!("failed to show main panel: {error}"))?;
    apply_main_panel_layout(window, cursor_position)?;
    window
        .set_focus()
        .map_err(|error| format!("failed to focus main panel: {error}"))
}

pub(crate) fn hide_main_panel(
    _app_handle: &AppHandle,
    window: &WebviewWindow,
) -> Result<(), String> {
    window
        .hide()
        .map_err(|error| format!("failed to hide main panel: {error}"))
}

pub(crate) fn configure_settings_window_builder<'a, R, M>(
    builder: WebviewWindowBuilder<'a, R, M>,
) -> WebviewWindowBuilder<'a, R, M>
where
    R: Runtime,
    M: Manager<R>,
{
    builder
}

pub(crate) fn initialize_help_window(_window: WebviewWindow) -> Result<(), String> {
    Ok(())
}

pub(crate) fn show_help_window(app_handle: &AppHandle, label: &str) -> Result<(), String> {
    let window = app_handle
        .get_webview_window(label)
        .ok_or_else(|| "failed to find shortcut help window".to_owned())?;
    window
        .show()
        .map_err(|error| format!("failed to show shortcut help window: {error}"))
}

pub(crate) fn hide_help_window(app_handle: &AppHandle, label: &str) -> Result<(), String> {
    app_handle
        .get_webview_window(label)
        .ok_or_else(|| "failed to find shortcut help window".to_owned())?
        .hide()
        .map_err(|error| format!("failed to hide shortcut help window: {error}"))
}

pub(crate) fn finish_setup(_app: &mut App) {}

#[cfg(not(target_os = "windows"))]
pub(crate) fn remember_frontmost_application(_app_handle: &AppHandle) -> Result<(), String> {
    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub(crate) fn post_paste_shortcut(_target_identifier: isize) -> Result<(), String> {
    Err("automatic paste is not supported on this platform".to_owned())
}

pub(crate) fn accessibility_permission_granted() -> bool {
    false
}

pub(crate) fn request_accessibility_permission() {}

pub(crate) fn open_accessibility_settings() -> Result<(), String> {
    Err("the current platform does not support opening accessibility settings".to_owned())
}
