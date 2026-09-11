use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    RwLock,
};
use tauri::{
    AppHandle, Manager, PhysicalPosition, WebviewUrl, WebviewWindow, WebviewWindowBuilder,
    WindowEvent,
};

const PREFIX: &str = "text-diff-";
const WIDTH: f64 = 1200.0;
const HEIGHT: f64 = 760.0;
const MIN_WIDTH: f64 = 860.0;
const MIN_HEIGHT: f64 = 520.0;
const CASCADE_OFFSET: i32 = 40;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DiffSource {
    pub name: String,
    pub content: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DiffInput {
    pub left: DiffSource,
    pub right: DiffSource,
}

#[derive(Default)]
pub struct DiffState {
    inputs: RwLock<HashMap<String, DiffInput>>,
    pinned: RwLock<HashSet<String>>,
    next_id: AtomicU64,
}

impl DiffState {
    pub fn insert(&self, input: DiffInput, pinned: bool) -> Result<String, String> {
        let label = format!("{PREFIX}{}", self.next_id.fetch_add(1, Ordering::Relaxed));
        let mut inputs = self
            .inputs
            .write()
            .map_err(|e| e.to_string())?;
        let mut pinned_windows = self.pinned.write().map_err(|e| e.to_string())?;
        inputs.insert(label.clone(), input);
        if pinned {
            pinned_windows.insert(label.clone());
        }
        Ok(label)
    }

    pub fn get(&self, label: &str) -> Result<DiffInput, String> {
        self.inputs
            .read()
            .map_err(|e| e.to_string())?
            .get(label)
            .cloned()
            .ok_or_else(|| "对比窗口内容不存在".to_owned())
    }

    pub fn contains(&self, label: &str) -> Result<bool, String> {
        Ok(self
            .inputs
            .read()
            .map_err(|e| e.to_string())?
            .contains_key(label))
    }

    pub fn remove(&self, label: &str) -> Result<(), String> {
        self.inputs
            .write()
            .map_err(|e| e.to_string())?
            .remove(label);
        self.pinned
            .write()
            .map_err(|e| e.to_string())?
            .remove(label);
        Ok(())
    }

    pub fn is_pinned(&self, label: &str) -> Result<bool, String> {
        Ok(self
            .pinned
            .read()
            .map_err(|e| e.to_string())?
            .contains(label))
    }

    pub fn set_pinned(&self, label: &str, pinned: bool) -> Result<(), String> {
        let mut windows = self.pinned.write().map_err(|e| e.to_string())?;
        if pinned {
            windows.insert(label.to_owned());
        } else {
            windows.remove(label);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(content: &str) -> DiffInput {
        let source = DiffSource {
            name: "source".into(),
            content: content.into(),
        };
        DiffInput {
            left: source.clone(),
            right: source,
        }
    }

    #[test]
    fn windows_keep_independent_snapshots_and_release_on_close() {
        let state = DiffState::default();
        let first = state.insert(input("first"), true).unwrap();
        let second = state.insert(input("second"), false).unwrap();
        assert_ne!(first, second);
        assert!(state.is_pinned(&first).unwrap());
        assert!(!state.is_pinned(&second).unwrap());
        let mut returned = state.get(&first).unwrap();
        returned.left.content = "edited".into();
        assert_eq!(state.get(&first).unwrap().left.content, "first");
        state.remove(&first).unwrap();
        assert!(!state.contains(&first).unwrap());
        assert!(!state.is_pinned(&first).unwrap());
        assert!(state.get(&first).is_err());
        assert_eq!(state.get(&second).unwrap().right.content, "second");
    }
}

pub fn create(app: &AppHandle, input: DiffInput, pinned: bool) -> Result<String, String> {
    let state = app.state::<DiffState>();
    let label = state.insert(input, pinned)?;
    if let Err(error) = build(app, &label) {
        if let Some(window) = app.get_webview_window(&label) {
            if let Err(cleanup) = window.destroy() {
                eprintln!("failed to clean up diff window: {cleanup}");
            }
        }
        state.remove(&label)?;
        return Err(error);
    }
    Ok(label)
}

fn build(app: &AppHandle, label: &str) -> Result<(), String> {
    let window = WebviewWindowBuilder::new(app, label, WebviewUrl::App("/diff".into()))
        .title("文本对比")
        .decorations(false)
        .resizable(true)
        .visible(false)
        .always_on_top(true)
        .accept_first_mouse(true)
        .inner_size(WIDTH, HEIGHT)
        .min_inner_size(MIN_WIDTH, MIN_HEIGHT)
        .center()
        .build()
        .map_err(|e| e.to_string())?;
    let handle = app.clone();
    let owned_label = label.to_owned();
    window.on_window_event(move |event| {
        if let Err(error) = handle_event(&handle, &owned_label, event) {
            eprintln!("failed to update diff window: {error}");
        }
    });
    cascade_window(app, &window)?;
    window
        .show()
        .and_then(|_| window.set_focus())
        .map_err(|e| e.to_string())
}

fn handle_event(app: &AppHandle, label: &str, event: &WindowEvent) -> Result<(), String> {
    let state = app.state::<DiffState>();
    match event {
        WindowEvent::Destroyed => state.remove(label)?,
        WindowEvent::Focused(false) if !state.is_pinned(label)? => {
            // Raise a newly opened comparison above pinned siblings only until it loses focus.
            if let Some(window) = app.get_webview_window(label) {
                window.set_always_on_top(false).map_err(|e| e.to_string())?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn cascade_window(app: &AppHandle, window: &WebviewWindow) -> Result<(), String> {
    if let Some(previous) = app
        .webview_windows()
        .into_iter()
        .filter(|(key, _)| key.starts_with(PREFIX) && key != window.label())
        .max_by_key(|(key, _)| {
            key.strip_prefix(PREFIX)
                .and_then(|id| id.parse::<u64>().ok())
        })
        .map(|(_, window)| window)
    {
        let position = previous.outer_position().map_err(|e| e.to_string())?;
        let desired =
            PhysicalPosition::new(position.x + CASCADE_OFFSET, position.y + CASCADE_OFFSET);
        if let Some(monitor) = window.current_monitor().map_err(|e| e.to_string())? {
            let origin = monitor.position();
            let monitor_size = monitor.size();
            let size = window.outer_size().map_err(|e| e.to_string())?;
            let max_x = origin.x + monitor_size.width.saturating_sub(size.width) as i32;
            let max_y = origin.y + monitor_size.height.saturating_sub(size.height) as i32;
            window
                .set_position(PhysicalPosition::new(
                    desired.x.clamp(origin.x, max_x),
                    desired.y.clamp(origin.y, max_y),
                ))
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}
