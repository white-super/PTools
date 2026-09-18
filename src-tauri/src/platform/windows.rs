pub(crate) use super::generic::{
    accessibility_permission_granted, configure_builder, configure_main_window_builder,
    configure_settings_window_builder, finish_setup, hide_help_window, hide_main_panel,
    initialize_help_window, initialize_main_panel, main_panel_is_visible,
    open_accessibility_settings, request_accessibility_permission, show_help_window,
    MAIN_PANEL_INITIAL_HEIGHT, MAIN_PANEL_INITIAL_WIDTH,
};
use super::{main_panel_layout, PasteTargetState};
use std::time::Duration;
use tauri::{
    image::Image,
    tray::{MouseButton, MouseButtonState, TrayIconEvent},
    AppHandle, Manager, PhysicalPosition, WebviewWindow,
};
use windows::Win32::{
    Foundation::HWND,
    UI::{
        Input::KeyboardAndMouse::{
            SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS,
            KEYEVENTF_KEYUP, VIRTUAL_KEY, VK_CONTROL, VK_V,
        },
        WindowsAndMessaging::{
            GetForegroundWindow, IsWindow, SetForegroundWindow, SetWindowPos, SWP_NOACTIVATE,
            SWP_NOOWNERZORDER, SWP_NOZORDER,
        },
    },
};

pub(crate) const DEFAULT_MAIN_SHORTCUT: &str = "Alt+V";
pub(crate) const TRAY_ICON: Image<'_> = tauri::include_image!("./icons/32x32.png");
pub(crate) const TRAY_ICON_IS_TEMPLATE: bool = false;
pub(crate) const TRAY_MENU_ON_LEFT_CLICK: bool = false;

const PASTE_FOCUS_DELAY: Duration = Duration::from_millis(40);
const MAIN_PANEL_HEIGHT_RATIO: f64 = 0.25;
const MAIN_PANEL_MIN_LOGICAL_HEIGHT: f64 = 320.0;
const MAIN_PANEL_MAX_LOGICAL_HEIGHT: f64 = 380.0;

pub(crate) fn tray_event_opens_settings(event: &TrayIconEvent) -> bool {
    match event {
        TrayIconEvent::Click {
            button,
            button_state,
            ..
        } => tray_click_opens_settings(*button, *button_state),
        _ => false,
    }
}

fn tray_click_opens_settings(button: MouseButton, button_state: MouseButtonState) -> bool {
    button == MouseButton::Left && button_state == MouseButtonState::Up
}

pub(super) fn calculate_main_panel_height(monitor_physical_height: u32, scale_factor: f64) -> u32 {
    let scale_factor = if scale_factor.is_finite() && scale_factor > 0.0 {
        scale_factor
    } else {
        1.0
    };
    let monitor_logical_height = monitor_physical_height as f64 / scale_factor;
    let panel_logical_height = (monitor_logical_height * MAIN_PANEL_HEIGHT_RATIO)
        .clamp(MAIN_PANEL_MIN_LOGICAL_HEIGHT, MAIN_PANEL_MAX_LOGICAL_HEIGHT);

    ((panel_logical_height * scale_factor).round() as u32).min(monitor_physical_height)
}

fn keyboard_input(key: VIRTUAL_KEY, flags: KEYBD_EVENT_FLAGS) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: key,
                dwFlags: flags,
                ..Default::default()
            },
        },
    }
}

pub(crate) fn remember_frontmost_application(app_handle: &AppHandle) -> Result<(), String> {
    let target_window = unsafe { GetForegroundWindow() };
    if target_window.is_invalid() {
        return Err("failed to identify the foreground Windows application".to_owned());
    }

    for window in app_handle.webview_windows().values() {
        let app_window = window
            .hwnd()
            .map_err(|error| format!("failed to read a PTools window HWND: {error}"))?;
        if target_window == app_window {
            return Err("PTools 窗口不能作为顺序粘贴的目标".to_owned());
        }
    }

    let identifier = target_window.0 as isize;
    app_handle.state::<PasteTargetState>().replace(identifier)?;
    println!("[paste] remembered Windows target HWND: {identifier:#x}");
    Ok(())
}

pub(crate) fn window_is_foreground(window: &WebviewWindow) -> Result<bool, String> {
    let hwnd = window
        .hwnd()
        .map_err(|error| format!("failed to get the Windows window HWND: {error}"))?;
    Ok(unsafe { GetForegroundWindow() } == hwnd)
}

pub(crate) fn post_paste_shortcut(target_identifier: isize) -> Result<(), String> {
    let target_window = HWND(target_identifier as *mut std::ffi::c_void);
    if !unsafe { IsWindow(target_window) }.as_bool() {
        return Err("the window selected for paste no longer exists".to_owned());
    }
    if !unsafe { SetForegroundWindow(target_window) }.as_bool() {
        return Err("Windows prevented PTools from restoring the paste target window".to_owned());
    }

    std::thread::sleep(PASTE_FOCUS_DELAY);
    if unsafe { GetForegroundWindow() } != target_window {
        return Err("the paste target window did not regain focus".to_owned());
    }

    let inputs = [
        keyboard_input(VK_CONTROL, KEYBD_EVENT_FLAGS::default()),
        keyboard_input(VK_V, KEYBD_EVENT_FLAGS::default()),
        keyboard_input(VK_V, KEYEVENTF_KEYUP),
        keyboard_input(VK_CONTROL, KEYEVENTF_KEYUP),
    ];
    let sent = unsafe { SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) };
    if sent != inputs.len() as u32 {
        return Err(format!(
            "Windows accepted {sent} of {} paste key events: {}",
            inputs.len(),
            windows::core::Error::from_win32()
        ));
    }

    println!("[paste] sent Ctrl+V to Windows target HWND: {target_identifier:#x}");
    Ok(())
}

fn apply_native_main_panel_layout(
    window: &WebviewWindow,
    cursor_position: PhysicalPosition<f64>,
) -> Result<(), String> {
    let (position, size) = main_panel_layout(window, cursor_position)?;
    let hwnd = window
        .hwnd()
        .map_err(|error| format!("failed to get the main panel HWND: {error}"))?;

    unsafe {
        SetWindowPos(
            hwnd,
            HWND::default(),
            position.x,
            position.y,
            size.width as i32,
            size.height as i32,
            SWP_NOACTIVATE | SWP_NOOWNERZORDER | SWP_NOZORDER,
        )
        .map_err(|error| format!("failed to position the Windows main panel: {error}"))?;
    }

    println!(
        "[main-panel] Windows bounds set to ({}, {}) with size {}x{}",
        position.x, position.y, size.width, size.height
    );
    Ok(())
}

pub(crate) fn show_main_panel(
    app_handle: &AppHandle,
    window: &WebviewWindow,
    cursor_position: PhysicalPosition<f64>,
) -> Result<(), String> {
    window
        .show()
        .map_err(|error| format!("failed to show main panel: {error}"))?;
    apply_native_main_panel_layout(window, cursor_position)?;
    window
        .set_focus()
        .map_err(|error| format!("failed to focus main panel: {error}"))?;

    // WebView2 may restore the builder bounds immediately after the first show.
    let delayed_window = window.clone();
    let delayed_app_handle = app_handle.clone();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(80));
        let _ = delayed_app_handle.run_on_main_thread(move || {
            if let Err(error) = apply_native_main_panel_layout(&delayed_window, cursor_position) {
                eprintln!("failed to reapply Windows main panel layout: {error}");
            }
        });
    });

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{calculate_main_panel_height, tray_click_opens_settings};
    use tauri::tray::{MouseButton, MouseButtonState};

    #[test]
    fn panel_height_uses_quarter_of_logical_screen_height() {
        assert_eq!(calculate_main_panel_height(2160, 1.5), 540);
    }

    #[test]
    fn panel_height_respects_logical_height_limits() {
        assert_eq!(calculate_main_panel_height(1080, 1.0), 320);
        assert_eq!(calculate_main_panel_height(2400, 1.0), 380);
        assert_eq!(calculate_main_panel_height(300, 1.0), 300);
    }

    #[test]
    fn only_released_left_click_opens_settings() {
        assert!(tray_click_opens_settings(
            MouseButton::Left,
            MouseButtonState::Up
        ));
        assert!(!tray_click_opens_settings(
            MouseButton::Left,
            MouseButtonState::Down
        ));
        assert!(!tray_click_opens_settings(
            MouseButton::Right,
            MouseButtonState::Up
        ));
    }
}
