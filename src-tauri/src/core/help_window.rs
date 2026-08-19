use crate::platform;
use tauri::{
    App, AppHandle, Emitter, EventTarget, Manager, PhysicalPosition, PhysicalSize, Position, Size,
    WebviewUrl, WebviewWindowBuilder,
};

pub const VISIBILITY_EVENT: &str = "shortcut-help-visibility-changed";
const HELP_WINDOW_LABEL: &str = "shortcut-help";
const HELP_WINDOW_WIDTH: f64 = 320.0;
const HELP_WINDOW_HEIGHT: f64 = 450.0;
const HELP_WINDOW_LEFT_OFFSET: f64 = 14.0;
const HELP_WINDOW_BOTTOM_GAP: f64 = 8.0;

pub fn create(app: &mut App) -> Result<(), String> {
    let window = WebviewWindowBuilder::new(
        app,
        HELP_WINDOW_LABEL,
        WebviewUrl::App("/shortcut-help".into()),
    )
    .title("快捷键帮助")
    .decorations(false)
    .transparent(true)
    .resizable(false)
    .always_on_top(true)
    .visible(false)
    .skip_taskbar(true)
    .closable(false)
    .minimizable(false)
    .visible_on_all_workspaces(true)
    .inner_size(HELP_WINDOW_WIDTH, HELP_WINDOW_HEIGHT)
    .build()
    .map_err(|error| format!("failed to create shortcut help window: {error}"))?;

    platform::initialize_help_window(window)
}

pub fn show(app_handle: &AppHandle) -> Result<(), String> {
    position_above_main_panel(app_handle)?;
    platform::show_help_window(app_handle, HELP_WINDOW_LABEL)?;
    emit_visibility(app_handle, true)
}

pub fn hide(app_handle: &AppHandle) -> Result<(), String> {
    platform::hide_help_window(app_handle, HELP_WINDOW_LABEL)?;
    emit_visibility(app_handle, false)
}

fn position_above_main_panel(app_handle: &AppHandle) -> Result<(), String> {
    let main_window = app_handle
        .get_webview_window("main")
        .ok_or_else(|| "failed to find main window".to_owned())?;
    let help_window = app_handle
        .get_webview_window(HELP_WINDOW_LABEL)
        .ok_or_else(|| "failed to find shortcut help window".to_owned())?;
    let scale_factor = main_window
        .scale_factor()
        .map_err(|error| format!("failed to read main window scale factor: {error}"))?;
    let main_position = main_window
        .outer_position()
        .map_err(|error| format!("failed to read main window position: {error}"))?;
    let help_size = PhysicalSize::new(
        (HELP_WINDOW_WIDTH * scale_factor).round() as u32,
        (HELP_WINDOW_HEIGHT * scale_factor).round() as u32,
    );
    let help_position = PhysicalPosition::new(
        main_position.x + (HELP_WINDOW_LEFT_OFFSET * scale_factor).round() as i32,
        main_position.y
            - help_size.height as i32
            - (HELP_WINDOW_BOTTOM_GAP * scale_factor).round() as i32,
    );
    help_window
        .set_size(Size::Physical(help_size))
        .map_err(|error| format!("failed to resize shortcut help window: {error}"))?;
    help_window
        .set_position(Position::Physical(help_position))
        .map_err(|error| format!("failed to position shortcut help window: {error}"))
}

fn emit_visibility(app_handle: &AppHandle, visible: bool) -> Result<(), String> {
    app_handle
        .emit_to(EventTarget::labeled("main"), VISIBILITY_EVENT, visible)
        .map_err(|error| format!("failed to emit shortcut help visibility: {error}"))
}
