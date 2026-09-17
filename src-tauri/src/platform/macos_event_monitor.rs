use super::MAIN_PANEL_LABEL;
use std::sync::{Mutex, OnceLock};
use tauri::AppHandle;
use tauri::WebviewWindow;
use tauri_nspanel::{
    block::ConcreteBlock,
    cocoa::{
        appkit::NSEventMask,
        base::id,
    },
    objc::{class, msg_send, sel, sel_impl},
    ManagerExt,
};

static MONITORS_INSTALLED: OnceLock<Mutex<bool>> = OnceLock::new();

pub(super) fn install_mouse_dismiss_monitors(
    app_handle: &AppHandle,
    main_window: &WebviewWindow,
) -> Result<(), String> {
    let installed = MONITORS_INSTALLED.get_or_init(|| Mutex::new(false));
    let mut is_installed = installed
        .lock()
        .map_err(|_| "failed to access macOS outside-click monitors".to_owned())?;
    if *is_installed {
        return Ok(());
    }

    let main_window = main_window
        .ns_window()
        .map_err(|error| format!("failed to get the main panel native window: {error}"))?
        as usize;
    if main_window == 0 {
        return Err("failed to get the main panel native window".to_owned());
    }

    let mask = NSEventMask::NSLeftMouseDownMask
        | NSEventMask::NSRightMouseDownMask
        | NSEventMask::NSOtherMouseDownMask;
    let global_handle = app_handle.clone();
    let global_handler = ConcreteBlock::new(move |_event: id| {
        schedule_main_panel_dismissal(&global_handle);
    })
    .copy();
    let local_handle = app_handle.clone();
    let local_handler = ConcreteBlock::new(move |event: id| {
        if !event_belongs_to_main_panel(event, main_window) {
            schedule_main_panel_dismissal(&local_handle);
        }
        event
    })
    .copy();

    unsafe {
        let global_monitor: id = msg_send![
            class!(NSEvent),
            addGlobalMonitorForEventsMatchingMask: mask.bits()
            handler: global_handler
        ];
        if global_monitor.is_null() {
            return Err("failed to install the macOS global outside-click monitor".to_owned());
        }

        let local_monitor: id = msg_send![
            class!(NSEvent),
            addLocalMonitorForEventsMatchingMask: mask.bits()
            handler: local_handler
        ];
        if local_monitor.is_null() {
            let _: () = msg_send![class!(NSEvent), removeMonitor: global_monitor];
            return Err("failed to install the macOS local outside-click monitor".to_owned());
        }

        let _ = (global_monitor, local_monitor);
    }

    *is_installed = true;
    Ok(())
}

fn event_belongs_to_main_panel(event: id, main_window: usize) -> bool {
    if event.is_null() {
        return false;
    }
    let event_window: id = unsafe { msg_send![event, window] };
    event_window as usize == main_window
}

fn schedule_main_panel_dismissal(app_handle: &AppHandle) {
    let main_thread_handle = app_handle.clone();
    if let Err(error) = app_handle.run_on_main_thread(move || {
        dismiss_visible_main_panel(&main_thread_handle);
    }) {
        eprintln!("failed to schedule main panel dismissal after an outside click: {error}");
    }
}

fn dismiss_visible_main_panel(app_handle: &AppHandle) {
    let Ok(panel) = app_handle.get_webview_panel(MAIN_PANEL_LABEL) else {
        return;
    };
    if !panel.is_visible() {
        return;
    }
    if let Err(error) = crate::cmds::hide_main_panel_now(app_handle) {
        eprintln!("failed to dismiss the main panel after an outside click: {error}");
    }
}
