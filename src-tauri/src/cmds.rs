use std::sync::RwLock;
use tauri::{
    AppHandle, Manager, Monitor, PhysicalPosition, PhysicalSize, Position, Size, WebviewWindow,
};
use tauri_nspanel::ManagerExt;

use crate::system_permissions;

type CmdResult<T = ()> = Result<T, String>;

#[derive(Default)]
pub struct PasteTargetProcessId {
    process_id: RwLock<Option<i32>>,
}

impl PasteTargetProcessId {
    fn replace(&self, process_id: i32) -> CmdResult {
        let mut stored_process_id = self
            .process_id
            .write()
            .map_err(|_| "failed to update the paste target application".to_owned())?;
        *stored_process_id = Some(process_id);
        Ok(())
    }

    fn get(&self) -> CmdResult<i32> {
        let stored_process_id = self
            .process_id
            .read()
            .map_err(|_| "failed to read the paste target application".to_owned())?;
        (*stored_process_id)
            .ok_or_else(|| "failed to identify the app that should receive the paste".to_owned())
    }
}

pub const MAIN_PANEL_INITIAL_WIDTH: f64 = 1.0;
pub const MAIN_PANEL_INITIAL_HEIGHT: f64 = 1.0;
const MAIN_PANEL_HEIGHT_RATIO: f64 = 0.30;
const MAIN_PANEL_BOTTOM_MARGIN: i32 = 0;

#[cfg(target_os = "macos")]
const PASTE_KEY_CODE: u16 = 0x09;

fn contains_cursor(monitor: &Monitor, cursor_position: PhysicalPosition<f64>) -> bool {
    let monitor_position = monitor.position();
    let monitor_size = monitor.size();
    let monitor_right = monitor_position.x as f64 + monitor_size.width as f64;
    let monitor_bottom = monitor_position.y as f64 + monitor_size.height as f64;

    cursor_position.x >= monitor_position.x as f64
        && cursor_position.x < monitor_right
        && cursor_position.y >= monitor_position.y as f64
        && cursor_position.y < monitor_bottom
}

fn monitor_for_cursor(
    window: &WebviewWindow,
    cursor_position: PhysicalPosition<f64>,
) -> Result<Monitor, String> {
    let monitors = window
        .available_monitors()
        .map_err(|error| format!("failed to enumerate monitors: {error}"))?;
    if let Some(monitor) = monitors
        .iter()
        .find(|monitor| contains_cursor(monitor, cursor_position))
        .cloned()
    {
        return Ok(monitor);
    }

    eprintln!(
        "cursor position ({}, {}) did not match monitor bounds; using the main panel's current monitor",
        cursor_position.x, cursor_position.y
    );
    if let Some(monitor) = window
        .current_monitor()
        .map_err(|error| format!("failed to read current monitor: {error}"))?
    {
        return Ok(monitor);
    }

    window
        .primary_monitor()
        .map_err(|error| format!("failed to read primary monitor: {error}"))?
        .ok_or_else(|| "failed to identify a monitor for the main panel".to_owned())
}

fn position_main_panel(
    window: &WebviewWindow,
    cursor_position: PhysicalPosition<f64>,
) -> Result<(), String> {
    let monitor = monitor_for_cursor(window, cursor_position)?;
    let monitor_position = monitor.position();
    let monitor_size = monitor.size();
    let panel_height = (monitor_size.height as f64 * MAIN_PANEL_HEIGHT_RATIO).round() as u32;
    let panel_position = PhysicalPosition::new(
        monitor_position.x,
        monitor_position.y + monitor_size.height as i32
            - panel_height as i32
            - MAIN_PANEL_BOTTOM_MARGIN,
    );
    let panel_size = PhysicalSize::new(monitor_size.width, panel_height);

    window
        .set_size(Size::Physical(panel_size))
        .map_err(|error| format!("failed to resize main panel: {error}"))?;
    window
        .set_position(Position::Physical(panel_position))
        .map_err(|error| format!("failed to position main panel: {error}"))?;

    println!(
        "[main-panel] positioned at ({}, {}) with size {}x{} on monitor ({}, {})",
        panel_position.x,
        panel_position.y,
        panel_size.width,
        panel_size.height,
        monitor_position.x,
        monitor_position.y,
    );
    Ok(())
}

pub(crate) fn hide_main_panel_now(app_handle: &AppHandle) -> CmdResult {
    crate::core::help_window::hide(app_handle)?;
    let window = app_handle
        .get_webview_window("main")
        .ok_or_else(|| "failed to find main window".to_owned())?;
    if window
        .is_focused()
        .map_err(|error| format!("failed to read main panel focus state: {error}"))?
    {
        app_handle
            .state::<crate::core::handle::MainPanelState>()
            .suppress_next_blur();
    }
    let panel = app_handle
        .get_webview_panel("main")
        .map_err(|error| format!("failed to find main panel: {error:?}"))?;
    panel.order_out(None);
    Ok(())
}

async fn run_main_panel_task(
    app_handle: AppHandle,
    task: impl FnOnce(&AppHandle) -> CmdResult + Send + 'static,
) -> CmdResult {
    let (sender, mut receiver) = tauri::async_runtime::channel(1);
    let main_thread_handle = app_handle.clone();
    app_handle
        .run_on_main_thread(move || {
            let result = task(&main_thread_handle);
            if sender.try_send(result).is_err() {
                eprintln!("failed to return main panel task result");
            }
        })
        .map_err(|error| format!("failed to schedule main panel task: {error}"))?;

    receiver
        .recv()
        .await
        .ok_or_else(|| "main panel task ended before returning a result".to_owned())?
}

#[cfg(target_os = "macos")]
fn frontmost_application_process_id() -> CmdResult<i32> {
    use cocoa::base::{id, nil};
    use objc::{class, msg_send, sel, sel_impl};

    unsafe {
        let workspace: id = msg_send![class!(NSWorkspace), sharedWorkspace];
        let application: id = msg_send![workspace, frontmostApplication];
        if workspace == nil || application == nil {
            return Err("failed to identify the frontmost application".to_owned());
        }

        let process_id: i32 = msg_send![application, processIdentifier];
        if process_id <= 0 || process_id == std::process::id() as i32 {
            return Err("failed to identify the app that should receive the paste".to_owned());
        }

        Ok(process_id)
    }
}

#[cfg(target_os = "macos")]
pub fn remember_frontmost_application(app_handle: &AppHandle) -> CmdResult {
    let process_id = frontmost_application_process_id()?;
    println!("[paste] remembered target process: {process_id}");
    app_handle
        .state::<PasteTargetProcessId>()
        .replace(process_id)
}

#[cfg(not(target_os = "macos"))]
pub fn remember_frontmost_application(_app_handle: &AppHandle) -> CmdResult {
    Ok(())
}

#[cfg(target_os = "macos")]
fn post_paste_shortcut(target_process_id: i32) -> CmdResult {
    use core_graphics::{
        event::{CGEvent, CGEventFlags, CGEventTapLocation},
        event_source::{CGEventSource, CGEventSourceStateID},
    };

    let source = CGEventSource::new(CGEventSourceStateID::CombinedSessionState)
        .map_err(|_| "failed to create the macOS keyboard event source".to_owned())?;
    let key_down = CGEvent::new_keyboard_event(source.clone(), PASTE_KEY_CODE, true)
        .map_err(|_| "failed to create the Command+V key-down event".to_owned())?;
    let key_up = CGEvent::new_keyboard_event(source, PASTE_KEY_CODE, false)
        .map_err(|_| "failed to create the Command+V key-up event".to_owned())?;

    key_down.set_flags(CGEventFlags::CGEventFlagCommand);
    key_up.set_flags(CGEventFlags::CGEventFlagCommand);
    match frontmost_application_process_id() {
        Ok(frontmost_process_id) if frontmost_process_id == target_process_id => {
            println!(
                "[paste] target process {target_process_id} is still frontmost; posting through HID event stream"
            );
            key_down.post(CGEventTapLocation::HID);
            key_up.post(CGEventTapLocation::HID);
        }
        Ok(frontmost_process_id) => {
            eprintln!(
                "[paste] frontmost process changed from {target_process_id} to {frontmost_process_id}; posting to remembered target"
            );
            key_down.post_to_pid(target_process_id);
            key_up.post_to_pid(target_process_id);
        }
        Err(error) => {
            eprintln!("[paste] failed to re-check frontmost process: {error}");
            key_down.post_to_pid(target_process_id);
            key_up.post_to_pid(target_process_id);
        }
    }
    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn frontmost_application_process_id() -> CmdResult<i32> {
    Err("automatic paste is currently supported only on macOS".to_owned())
}

#[cfg(not(target_os = "macos"))]
fn post_paste_shortcut(_target_process_id: i32) -> CmdResult {
    Err("automatic paste is currently supported only on macOS".to_owned())
}

#[tauri::command]
pub fn toggle_window(app_handle: tauri::AppHandle) {
    let main_thread_handle = app_handle.clone();
    if let Err(error) = app_handle.run_on_main_thread(move || {
        let Ok(panel) = main_thread_handle.get_webview_panel("main") else {
            eprintln!("failed to find main panel");
            return;
        };
        if panel.is_visible() {
            if let Err(error) = hide_main_panel_now(&main_thread_handle) {
                eprintln!("failed to hide main panel: {error}");
            }
            return;
        }

        let Some(window) = main_thread_handle.get_webview_window("main") else {
            eprintln!("failed to find main window");
            return;
        };
        main_thread_handle
            .state::<crate::core::formatter_window::TextFormatterState>()
            .set_main_panel_activation(true);
        let Ok(cursor_position) = main_thread_handle.cursor_position() else {
            eprintln!("failed to read cursor position");
            main_thread_handle
                .state::<crate::core::formatter_window::TextFormatterState>()
                .set_main_panel_activation(false);
            return;
        };
        if let Err(error) = remember_frontmost_application(&main_thread_handle) {
            eprintln!("failed to remember paste target application: {error}");
        }
        if let Err(error) = position_main_panel(&window, cursor_position) {
            eprintln!("failed to prepare main panel: {error}");
            main_thread_handle
                .state::<crate::core::formatter_window::TextFormatterState>()
                .set_main_panel_activation(false);
            return;
        }

        if let Err(error) = crate::core::formatter_window::destroy_unpinned(&main_thread_handle) {
            eprintln!("failed to close unpinned text formatter windows: {error}");
        }
        panel.show();
        main_thread_handle
            .state::<crate::core::formatter_window::TextFormatterState>()
            .set_main_panel_activation(false);
        println!(
            "[main-panel] show requested; visible={}",
            panel.is_visible()
        );
    }) {
        eprintln!("failed to run main panel toggle on the main thread: {error}");
    }
}

#[tauri::command]
pub async fn hide_main_panel(app_handle: AppHandle) -> CmdResult {
    run_main_panel_task(app_handle, hide_main_panel_now).await
}

#[tauri::command]
pub async fn paste_into_active_app(app_handle: AppHandle) -> CmdResult {
    run_main_panel_task(app_handle, |main_thread_handle| {
        system_permissions::ensure_accessibility_permission()?;
        let target_process_id = main_thread_handle.state::<PasteTargetProcessId>().get()?;
        hide_main_panel_now(main_thread_handle)?;
        post_paste_shortcut(target_process_id)
    })
    .await
}
