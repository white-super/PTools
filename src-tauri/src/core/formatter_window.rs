use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    RwLock,
};
#[cfg(target_os = "windows")]
use std::time::{Duration, Instant};
use tauri::{
    AppHandle, Manager, PhysicalPosition, Position, WebviewUrl, WebviewWindowBuilder, WindowEvent,
};

pub const WINDOW_LABEL_PREFIX: &str = "text-formatter-";
const WINDOW_INITIAL_WIDTH: f64 = 1100.0;
const WINDOW_INITIAL_HEIGHT: f64 = 720.0;
const WINDOW_MIN_WIDTH: f64 = 760.0;
const WINDOW_MIN_HEIGHT: f64 = 520.0;
const WINDOW_CASCADE_OFFSET: i32 = 40;
#[cfg(target_os = "windows")]
const WINDOWS_BLUR_CONFIRM_DELAY: Duration = Duration::from_millis(180);
#[cfg(target_os = "windows")]
const WINDOWS_OPENING_GRACE_PERIOD: Duration = Duration::from_millis(500);
#[cfg(target_os = "windows")]
const WINDOWS_MOVE_GRACE_PERIOD: Duration = Duration::from_millis(500);

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TextFormat {
    Json,
    Xml,
    Html,
    Url,
    Base64,
    Date,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextFormatterInput {
    pub format: TextFormat,
    pub content: String,
    pub updated_at: i64,
}

#[derive(Default)]
pub struct TextFormatterState {
    inputs: RwLock<HashMap<String, TextFormatterInput>>,
    pinned_windows: RwLock<HashSet<String>>,
    #[cfg(target_os = "windows")]
    focused_windows: RwLock<HashSet<String>>,
    #[cfg(target_os = "windows")]
    opening_windows: RwLock<HashMap<String, Instant>>,
    #[cfg(target_os = "windows")]
    moving_windows: RwLock<HashMap<String, Instant>>,
    next_id: AtomicU64,
    main_panel_activation: AtomicBool,
}

impl TextFormatterState {
    pub fn allocate_label(&self) -> String {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        format!("{WINDOW_LABEL_PREFIX}{id}")
    }

    pub fn store_input(&self, label: &str, input: TextFormatterInput) -> Result<(), String> {
        self.inputs
            .write()
            .map_err(|_| "failed to store text formatter input".to_owned())?
            .insert(label.to_owned(), input);
        Ok(())
    }

    pub fn current_input(&self, label: &str) -> Result<Option<TextFormatterInput>, String> {
        self.inputs
            .read()
            .map(|inputs| inputs.get(label).cloned())
            .map_err(|_| "failed to read text formatter input".to_owned())
    }

    pub fn active_formats(&self) -> Result<Vec<TextFormat>, String> {
        let inputs = self
            .inputs
            .read()
            .map_err(|_| "failed to read active text formatter inputs".to_owned())?;
        let formats = inputs.values().map(|input| input.format).collect::<HashSet<_>>();
        let mut formats = formats.into_iter().collect::<Vec<_>>();
        formats.sort();
        Ok(formats)
    }

    pub fn is_pinned(&self, label: &str) -> Result<bool, String> {
        self.pinned_windows
            .read()
            .map(|windows| windows.contains(label))
            .map_err(|_| "failed to read text formatter pin state".to_owned())
    }

    pub fn set_pinned(&self, label: &str, pinned: bool) -> Result<(), String> {
        let mut windows = self
            .pinned_windows
            .write()
            .map_err(|_| "failed to update text formatter pin state".to_owned())?;
        if pinned {
            windows.insert(label.to_owned());
        } else {
            windows.remove(label);
        }
        Ok(())
    }

    pub fn remove(&self, label: &str) -> Result<(), String> {
        self.inputs
            .write()
            .map_err(|_| "failed to remove text formatter input".to_owned())?
            .remove(label);
        self.pinned_windows
            .write()
            .map_err(|_| "failed to remove text formatter pin state".to_owned())?
            .remove(label);
        #[cfg(target_os = "windows")]
        self.focused_windows
            .write()
            .map_err(|_| "failed to remove text formatter focus state".to_owned())?
            .remove(label);
        #[cfg(target_os = "windows")]
        self.opening_windows
            .write()
            .map_err(|_| "failed to remove text formatter opening state".to_owned())?
            .remove(label);
        #[cfg(target_os = "windows")]
        self.moving_windows
            .write()
            .map_err(|_| "failed to remove text formatter moving state".to_owned())?
            .remove(label);
        Ok(())
    }

    #[cfg(target_os = "windows")]
    pub fn mark_focused(&self, label: &str) -> Result<(), String> {
        self.focused_windows
            .write()
            .map_err(|_| "failed to update text formatter focus state".to_owned())?
            .insert(label.to_owned());
        Ok(())
    }

    #[cfg(target_os = "windows")]
    pub fn has_received_focus(&self, label: &str) -> Result<bool, String> {
        self.focused_windows
            .read()
            .map(|windows| windows.contains(label))
            .map_err(|_| "failed to read text formatter focus state".to_owned())
    }

    #[cfg(target_os = "windows")]
    pub fn mark_opening(&self, label: &str) -> Result<(), String> {
        self.opening_windows
            .write()
            .map_err(|_| "failed to update text formatter opening state".to_owned())?
            .insert(label.to_owned(), Instant::now());
        Ok(())
    }

    #[cfg(target_os = "windows")]
    pub fn is_opening(&self, label: &str) -> Result<bool, String> {
        let mut windows = self
            .opening_windows
            .write()
            .map_err(|_| "failed to read text formatter opening state".to_owned())?;
        let Some(opened_at) = windows.get(label).copied() else {
            return Ok(false);
        };
        if opened_at.elapsed() < WINDOWS_OPENING_GRACE_PERIOD {
            return Ok(true);
        }
        windows.remove(label);
        Ok(false)
    }

    #[cfg(target_os = "windows")]
    pub fn mark_moved(&self, label: &str) -> Result<(), String> {
        self.moving_windows
            .write()
            .map_err(|_| "failed to update text formatter moving state".to_owned())?
            .insert(label.to_owned(), Instant::now());
        Ok(())
    }

    #[cfg(target_os = "windows")]
    pub fn is_moving(&self, label: &str) -> Result<bool, String> {
        let mut windows = self
            .moving_windows
            .write()
            .map_err(|_| "failed to read text formatter moving state".to_owned())?;
        let Some(moved_at) = windows.get(label).copied() else {
            return Ok(false);
        };
        if moved_at.elapsed() < WINDOWS_MOVE_GRACE_PERIOD {
            return Ok(true);
        }
        windows.remove(label);
        Ok(false)
    }

    pub fn set_main_panel_activation(&self, active: bool) {
        self.main_panel_activation.store(active, Ordering::Relaxed);
    }

    pub fn is_main_panel_activation(&self) -> bool {
        self.main_panel_activation.load(Ordering::Relaxed)
    }
}

pub fn create_for_input(
    app_handle: &AppHandle,
    input: TextFormatterInput,
) -> Result<String, String> {
    let state = app_handle.state::<TextFormatterState>();
    let label = state.allocate_label();
    state.store_input(&label, input)?;
    #[cfg(target_os = "windows")]
    state.mark_opening(&label)?;
    if let Err(error) = build_window(app_handle, &label) {
        let _ = state.remove(&label);
        return Err(error);
    }
    Ok(label)
}

pub fn destroy(app_handle: &AppHandle, label: &str) -> Result<(), String> {
    let window = app_handle
        .get_webview_window(label)
        .ok_or_else(|| format!("failed to find text formatter window: {label}"))?;
    window
        .destroy()
        .map_err(|error| format!("failed to close text formatter window: {error}"))?;
    app_handle.state::<TextFormatterState>().remove(label)
}

pub fn destroy_unpinned(app_handle: &AppHandle) -> Result<(), String> {
    let labels = app_handle
        .webview_windows()
        .keys()
        .filter(|label| label.starts_with(WINDOW_LABEL_PREFIX))
        .cloned()
        .collect::<Vec<_>>();
    for label in labels {
        if !app_handle.state::<TextFormatterState>().is_pinned(&label)? {
            destroy(app_handle, &label)?;
        }
    }
    Ok(())
}

pub fn set_pinned(app_handle: &AppHandle, label: &str, pinned: bool) -> Result<(), String> {
    let window = app_handle
        .get_webview_window(label)
        .ok_or_else(|| format!("failed to find text formatter window: {label}"))?;
    window
        .set_always_on_top(pinned)
        .map_err(|error| format!("failed to update text formatter pin state: {error}"))?;
    app_handle
        .state::<TextFormatterState>()
        .set_pinned(label, pinned)
}

fn build_window(app_handle: &AppHandle, label: &str) -> Result<(), String> {
    let route = format!("/formatter?windowId={label}");
    let window = WebviewWindowBuilder::new(app_handle, label, WebviewUrl::App(route.into()))
        .title("文本格式化")
        .decorations(false)
        .resizable(true)
        .fullscreen(false)
        .visible(false)
        .accept_first_mouse(true)
        .always_on_top(true)
        .skip_taskbar(true)
        .inner_size(WINDOW_INITIAL_WIDTH, WINDOW_INITIAL_HEIGHT)
        .min_inner_size(WINDOW_MIN_WIDTH, WINDOW_MIN_HEIGHT)
        .center()
        .build()
        .map_err(|error| format!("failed to create text formatter window: {error}"))?;
    let event_app_handle = app_handle.clone();
    let event_label = label.to_owned();
    window.on_window_event(move |event| {
        handle_window_event(&event_app_handle, &event_label, event);
    });
    if let Some(position) = previous_window_position(app_handle, label)? {
        window
            .set_position(Position::Physical(PhysicalPosition::new(
                position.x + WINDOW_CASCADE_OFFSET,
                position.y + WINDOW_CASCADE_OFFSET,
            )))
            .map_err(|error| format!("failed to position text formatter window: {error}"))?;
    }
    window
        .show()
        .and_then(|_| window.set_focus())
        .map_err(|error| format!("failed to show text formatter window: {error}"))
}

fn previous_window_position(
    app_handle: &AppHandle,
    current_label: &str,
) -> Result<Option<PhysicalPosition<i32>>, String> {
    let previous_label = app_handle
        .webview_windows()
        .keys()
        .filter(|label| label.starts_with(WINDOW_LABEL_PREFIX) && label.as_str() != current_label)
        .max_by_key(|label| formatter_id(label))
        .cloned();
    let Some(previous_label) = previous_label else {
        return Ok(None);
    };
    app_handle
        .get_webview_window(&previous_label)
        .ok_or_else(|| format!("failed to find previous text formatter window: {previous_label}"))?
        .outer_position()
        .map(Some)
        .map_err(|error| format!("failed to read previous text formatter position: {error}"))
}

fn formatter_id(label: &str) -> u64 {
    label
        .strip_prefix(WINDOW_LABEL_PREFIX)
        .and_then(|value| value.parse().ok())
        .unwrap_or_default()
}

fn handle_window_event(app_handle: &AppHandle, label: &str, event: &WindowEvent) {
    match event {
        WindowEvent::CloseRequested { api, .. } => {
            api.prevent_close();
            destroy_with_log(app_handle, label);
        }
        WindowEvent::Focused(true) => {
            #[cfg(target_os = "windows")]
            if let Err(error) = app_handle.state::<TextFormatterState>().mark_focused(label) {
                eprintln!("failed to record text formatter focus: {error}");
            }
        }
        WindowEvent::Moved(_) => {
            #[cfg(target_os = "windows")]
            if let Err(error) = app_handle.state::<TextFormatterState>().mark_moved(label) {
                eprintln!("failed to record text formatter movement: {error}");
            }
        }
        WindowEvent::Focused(false) => {
            #[cfg(target_os = "windows")]
            schedule_windows_blur_confirmation(app_handle, label);
            #[cfg(not(target_os = "windows"))]
            match should_destroy_on_blur(app_handle, label) {
                Ok(true) => destroy_with_log(app_handle, label),
                Ok(false) => {}
                Err(error) => eprintln!("failed to handle text formatter blur: {error}"),
            }
        }
        _ => {}
    }
}

#[cfg(target_os = "windows")]
fn schedule_windows_blur_confirmation(app_handle: &AppHandle, label: &str) {
    let delayed_app_handle = app_handle.clone();
    let delayed_label = label.to_owned();
    std::thread::spawn(move || {
        std::thread::sleep(WINDOWS_BLUR_CONFIRM_DELAY);
        let callback_app_handle = delayed_app_handle.clone();
        let callback_label = delayed_label.clone();
        if let Err(error) = delayed_app_handle.run_on_main_thread(move || {
            if callback_app_handle
                .get_webview_window(&callback_label)
                .is_none()
            {
                return;
            }
            match should_destroy_on_blur(&callback_app_handle, &callback_label) {
                Ok(true) => destroy_with_log(&callback_app_handle, &callback_label),
                Ok(false) => {}
                Err(error) => eprintln!("failed to confirm text formatter blur: {error}"),
            }
        }) {
            eprintln!("failed to schedule text formatter blur confirmation: {error}");
        }
    });
}

fn should_destroy_on_blur(app_handle: &AppHandle, label: &str) -> Result<bool, String> {
    #[cfg(target_os = "windows")]
    if app_handle.state::<TextFormatterState>().is_opening(label)? {
        return Ok(false);
    }
    #[cfg(target_os = "windows")]
    let window = app_handle
        .get_webview_window(label)
        .ok_or_else(|| format!("failed to find text formatter window: {label}"))?;
    #[cfg(target_os = "windows")]
    if crate::platform::window_is_foreground(&window)? {
        return Ok(false);
    }
    #[cfg(target_os = "windows")]
    if app_handle.state::<TextFormatterState>().is_moving(label)? {
        return Ok(false);
    }
    #[cfg(target_os = "windows")]
    if !app_handle
        .state::<TextFormatterState>()
        .has_received_focus(label)?
    {
        // WebView2 can emit an initial blur while the hidden window is being shown.
        return Ok(false);
    }
    if app_handle
        .state::<TextFormatterState>()
        .is_main_panel_activation()
    {
        return Ok(false);
    }
    if app_handle.state::<TextFormatterState>().is_pinned(label)? {
        return Ok(false);
    }
    if has_focused_formatter_window(app_handle, label)? {
        return Ok(false);
    }
    #[cfg(not(target_os = "windows"))]
    let window = app_handle
        .get_webview_window(label)
        .ok_or_else(|| format!("failed to find text formatter window: {label}"))?;
    window
        .is_visible()
        .map_err(|error| format!("failed to read text formatter visibility: {error}"))
}

fn has_focused_formatter_window(
    app_handle: &AppHandle,
    current_label: &str,
) -> Result<bool, String> {
    for (label, window) in app_handle.webview_windows() {
        if label == current_label || !label.starts_with(WINDOW_LABEL_PREFIX) {
            continue;
        }
        if window
            .is_focused()
            .map_err(|error| format!("failed to read text formatter focus state: {error}"))?
        {
            return Ok(true);
        }
    }
    Ok(false)
}

fn destroy_with_log(app_handle: &AppHandle, label: &str) {
    if let Err(error) = destroy(app_handle, label) {
        eprintln!("failed to close text formatter window {label}: {error}");
    }
}

#[cfg(test)]
mod tests {
    use super::{TextFormat, TextFormatterInput, TextFormatterState};

    fn input(content: &str) -> TextFormatterInput {
        TextFormatterInput {
            format: TextFormat::Json,
            content: content.to_owned(),
            updated_at: 0,
        }
    }

    #[test]
    fn keeps_inputs_independent_by_window_label() {
        let state = TextFormatterState::default();
        let first = state.allocate_label();
        let second = state.allocate_label();
        state.store_input(&first, input("first")).unwrap();
        state.store_input(&second, input("second")).unwrap();

        assert_eq!(
            state.current_input(&first).unwrap().unwrap().content,
            "first"
        );
        assert_eq!(
            state.current_input(&second).unwrap().unwrap().content,
            "second"
        );
    }

    #[test]
    fn pin_state_is_independent_by_window_label() {
        let state = TextFormatterState::default();
        let first = state.allocate_label();
        let second = state.allocate_label();
        state.set_pinned(&first, true).unwrap();

        assert!(state.is_pinned(&first).unwrap());
        assert!(!state.is_pinned(&second).unwrap());
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn formatter_focus_state_ignores_initial_blur_until_first_focus() {
        let state = TextFormatterState::default();
        let label = state.allocate_label();

        assert!(!state.has_received_focus(&label).unwrap());
        state.mark_focused(&label).unwrap();
        assert!(state.has_received_focus(&label).unwrap());
        state.remove(&label).unwrap();
        assert!(!state.has_received_focus(&label).unwrap());
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn formatter_opening_state_is_cleared_when_window_is_removed() {
        let state = TextFormatterState::default();
        let label = state.allocate_label();

        state.mark_opening(&label).unwrap();
        assert!(state.is_opening(&label).unwrap());
        state.remove(&label).unwrap();
        assert!(!state.is_opening(&label).unwrap());
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn formatter_moving_state_is_cleared_when_window_is_removed() {
        let state = TextFormatterState::default();
        let label = state.allocate_label();

        state.mark_moved(&label).unwrap();
        assert!(state.is_moving(&label).unwrap());
        state.remove(&label).unwrap();
        assert!(!state.is_moving(&label).unwrap());
    }
}
