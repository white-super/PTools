use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    RwLock,
};
use tauri::{
    AppHandle, Manager, PhysicalPosition, Position, WebviewUrl, WebviewWindowBuilder, WindowEvent,
};

pub const WINDOW_LABEL_PREFIX: &str = "text-formatter-";
const WINDOW_INITIAL_WIDTH: f64 = 1100.0;
const WINDOW_INITIAL_HEIGHT: f64 = 720.0;
const WINDOW_MIN_WIDTH: f64 = 760.0;
const WINDOW_MIN_HEIGHT: f64 = 520.0;
const WINDOW_CASCADE_OFFSET: i32 = 40;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TextFormat {
    Json,
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
        Ok(())
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
        WindowEvent::Focused(false) => match should_destroy_on_blur(app_handle, label) {
            Ok(true) => destroy_with_log(app_handle, label),
            Ok(false) => {}
            Err(error) => eprintln!("failed to handle text formatter blur: {error}"),
        },
        _ => {}
    }
}

fn should_destroy_on_blur(app_handle: &AppHandle, label: &str) -> Result<bool, String> {
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
}
