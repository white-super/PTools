use crate::platform;
use tauri::{
    App, AppHandle, Manager, PhysicalPosition, PhysicalSize, Position, Size, WebviewUrl,
    WebviewWindow, WebviewWindowBuilder,
};

pub const WINDOW_LABEL: &str = "sequential-paste";
const WINDOW_WIDTH: f64 = 320.0;
const WINDOW_MAX_HEIGHT: f64 = 640.0;
const WINDOW_MIN_HEIGHT: f64 = 360.0;
const WINDOW_MARGIN: f64 = 16.0;
const WINDOW_VERTICAL_INSET: f64 = 48.0;

pub fn create(app: &mut App) -> Result<(), String> {
    WebviewWindowBuilder::new(
        app,
        WINDOW_LABEL,
        WebviewUrl::App("/sequential-paste".into()),
    )
    .title("顺序粘贴")
    .decorations(false)
    .resizable(false)
    .fullscreen(false)
    .visible(false)
    .accept_first_mouse(true)
    .always_on_top(true)
    .skip_taskbar(true)
    .closable(false)
    .minimizable(false)
    .visible_on_all_workspaces(true)
    .inner_size(WINDOW_WIDTH, WINDOW_MAX_HEIGHT)
    .build()
    .map(|_| ())
    .map_err(|error| format!("failed to create sequential paste window: {error}"))
}

pub fn show(app_handle: &AppHandle) -> Result<(), String> {
    let window = get_window(app_handle)?;
    position_on_cursor_monitor(app_handle, &window)?;
    window
        .show()
        .and_then(|_| window.set_focus())
        .map_err(|error| format!("failed to show sequential paste window: {error}"))
}

pub fn hide(app_handle: &AppHandle) -> Result<(), String> {
    get_window(app_handle)?
        .hide()
        .map_err(|error| format!("failed to hide sequential paste window: {error}"))
}

fn get_window(app_handle: &AppHandle) -> Result<WebviewWindow, String> {
    app_handle
        .get_webview_window(WINDOW_LABEL)
        .ok_or_else(|| "failed to find sequential paste window".to_owned())
}

fn position_on_cursor_monitor(
    app_handle: &AppHandle,
    window: &WebviewWindow,
) -> Result<(), String> {
    let cursor = app_handle
        .cursor_position()
        .map_err(|error| format!("failed to read cursor position: {error}"))?;
    let monitor = platform::monitor_for_cursor(window, cursor)?;
    let scale = monitor.scale_factor();
    let monitor_position = monitor.position();
    let monitor_size = monitor.size();
    let width = (WINDOW_WIDTH * scale).round() as u32;
    let available_height = monitor_size.height as f64 / scale - WINDOW_VERTICAL_INSET * 2.0;
    let logical_height = available_height.clamp(WINDOW_MIN_HEIGHT, WINDOW_MAX_HEIGHT);
    let height = (logical_height * scale).round() as u32;
    let margin = (WINDOW_MARGIN * scale).round() as i32;
    let vertical_inset = (WINDOW_VERTICAL_INSET * scale).round() as i32;
    let position = PhysicalPosition::new(
        monitor_position.x + monitor_size.width as i32 - width as i32 - margin,
        monitor_position.y + vertical_inset,
    );
    window
        .set_size(Size::Physical(PhysicalSize::new(width, height)))
        .and_then(|_| window.set_position(Position::Physical(position)))
        .map_err(|error| format!("failed to position sequential paste window: {error}"))
}
