use serde::Serialize;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    RwLock,
};
use tauri::{Monitor, PhysicalPosition, PhysicalSize, WebviewWindow};
#[cfg(not(target_os = "windows"))]
use tauri::{Position, Size};

#[cfg(not(target_os = "macos"))]
mod generic;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
mod other;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "macos")]
pub(crate) use macos::*;
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub(crate) use other::*;
#[cfg(target_os = "windows")]
pub(crate) use windows::*;

pub(crate) const MAIN_PANEL_LABEL: &str = "main";
pub(crate) const MAIN_PANEL_FOCUS_EVENT: &str = "ptools://main-panel-focus";
pub(crate) const MAIN_PANEL_BLUR_EVENT: &str = "ptools://main-panel-blur";
pub(crate) const MAIN_PANEL_DISMISS_EVENT: &str = "ptools://main-panel-dismiss";
pub(crate) const WINDOW_MOVED_EVENT: &str = "tauri://move";
pub(crate) const WINDOW_RESIZED_EVENT: &str = "tauri://resize";

#[cfg(not(target_os = "windows"))]
const MAIN_PANEL_HEIGHT_RATIO: f64 = 0.30;

#[cfg(target_os = "windows")]
fn main_panel_height(monitor: &Monitor) -> u32 {
    windows::calculate_main_panel_height(monitor.size().height, monitor.scale_factor())
}

#[cfg(not(target_os = "windows"))]
fn main_panel_height(monitor: &Monitor) -> u32 {
    (monitor.size().height as f64 * MAIN_PANEL_HEIGHT_RATIO).round() as u32
}

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

pub(super) fn main_panel_layout(
    window: &WebviewWindow,
    cursor_position: PhysicalPosition<f64>,
) -> Result<(PhysicalPosition<i32>, PhysicalSize<u32>), String> {
    let monitor = monitor_for_cursor(window, cursor_position)?;
    let monitor_position = monitor.position();
    let monitor_size = monitor.size();
    let panel_height = main_panel_height(&monitor);
    let panel_position = PhysicalPosition::new(
        monitor_position.x,
        monitor_position.y + monitor_size.height as i32 - panel_height as i32,
    );
    let panel_size = PhysicalSize::new(monitor_size.width, panel_height);

    Ok((panel_position, panel_size))
}

#[cfg(not(target_os = "windows"))]
pub(super) fn apply_main_panel_layout(
    window: &WebviewWindow,
    cursor_position: PhysicalPosition<f64>,
) -> Result<(), String> {
    let (panel_position, panel_size) = main_panel_layout(window, cursor_position)?;

    window
        .set_size(Size::Physical(panel_size))
        .map_err(|error| format!("failed to resize main panel: {error}"))?;
    window
        .set_position(Position::Physical(panel_position))
        .map_err(|error| format!("failed to position main panel: {error}"))?;

    println!(
        "[main-panel] positioned at ({}, {}) with size {}x{} and bottom edge {}",
        panel_position.x,
        panel_position.y,
        panel_size.width,
        panel_size.height,
        panel_position.y + panel_size.height as i32,
    );
    Ok(())
}

#[derive(Default)]
pub(crate) struct MainPanelState {
    suppress_next_blur: AtomicBool,
}

#[derive(Default)]
pub(crate) struct PasteTargetState {
    identifier: RwLock<Option<isize>>,
}

impl PasteTargetState {
    pub(crate) fn replace(&self, identifier: isize) -> Result<(), String> {
        let mut stored_identifier = self
            .identifier
            .write()
            .map_err(|_| "failed to update the paste target".to_owned())?;
        *stored_identifier = Some(identifier);
        Ok(())
    }

    pub(crate) fn get(&self) -> Result<isize, String> {
        let stored_identifier = self
            .identifier
            .read()
            .map_err(|_| "failed to read the paste target".to_owned())?;
        (*stored_identifier)
            .ok_or_else(|| "failed to identify the window that should receive the paste".to_owned())
    }
}

impl MainPanelState {
    pub(crate) fn suppress_next_blur(&self) {
        self.suppress_next_blur.store(true, Ordering::Release);
    }

    pub(crate) fn take_blur_suppression(&self) -> bool {
        self.suppress_next_blur.swap(false, Ordering::AcqRel)
    }
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum SystemType {
    Macos,
    Windows,
    Linux,
    Other,
}

impl SystemType {
    pub(crate) fn current() -> Self {
        match std::env::consts::OS {
            "macos" => Self::Macos,
            "windows" => Self::Windows,
            "linux" => Self::Linux,
            _ => Self::Other,
        }
    }

    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Macos => "macOS",
            Self::Windows => "Windows",
            Self::Linux => "Linux",
            Self::Other => "未知系统",
        }
    }

    pub(crate) fn supports_accessibility_permission(self) -> bool {
        matches!(self, Self::Macos)
    }

    pub(crate) fn supports_automatic_paste(self) -> bool {
        matches!(self, Self::Macos | Self::Windows)
    }
}

#[cfg(test)]
mod tests {
    use super::{MainPanelState, SystemType};

    #[test]
    fn programmatic_hide_suppresses_only_the_next_blur() {
        let state = MainPanelState::default();
        state.suppress_next_blur();

        assert!(state.take_blur_suppression());
        assert!(!state.take_blur_suppression());
    }

    #[test]
    fn automatic_paste_support_is_independent_from_accessibility_permissions() {
        assert!(SystemType::Macos.supports_automatic_paste());
        assert!(SystemType::Macos.supports_accessibility_permission());
        assert!(SystemType::Windows.supports_automatic_paste());
        assert!(!SystemType::Windows.supports_accessibility_permission());
        assert!(!SystemType::Linux.supports_automatic_paste());
        assert!(!SystemType::Other.supports_automatic_paste());
    }
}
