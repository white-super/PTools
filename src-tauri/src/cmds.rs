use tauri::{AppHandle, Emitter, EventTarget, Manager, PhysicalPosition, WebviewWindow};

use crate::{platform, system_permissions};

type CmdResult<T = ()> = Result<T, String>;

pub(crate) fn show_main_panel_now(
    app_handle: &AppHandle,
    window: &WebviewWindow,
    cursor_position: PhysicalPosition<f64>,
) -> CmdResult {
    platform::show_main_panel(app_handle, window, cursor_position)?;
    app_handle
        .emit_to(
            EventTarget::labeled(platform::MAIN_PANEL_LABEL),
            platform::MAIN_PANEL_FOCUS_EVENT,
            true,
        )
        .map_err(|error| format!("failed to notify the visible main panel: {error}"))
}

pub(crate) fn hide_main_panel_now(app_handle: &AppHandle) -> CmdResult {
    crate::core::help_window::hide(app_handle)?;
    if let Err(error) = app_handle.emit_to(
        EventTarget::labeled(platform::MAIN_PANEL_LABEL),
        platform::MAIN_PANEL_DISMISS_EVENT,
        true,
    ) {
        eprintln!("failed to notify the main panel before hiding: {error}");
    }
    let window = app_handle
        .get_webview_window("main")
        .ok_or_else(|| "failed to find main window".to_owned())?;
    if window
        .is_focused()
        .map_err(|error| format!("failed to read main panel focus state: {error}"))?
    {
        app_handle
            .state::<platform::MainPanelState>()
            .suppress_next_blur();
    }
    platform::hide_main_panel(app_handle, &window)
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

#[tauri::command]
pub fn toggle_window(app_handle: tauri::AppHandle) {
    let main_thread_handle = app_handle.clone();
    if let Err(error) = app_handle.run_on_main_thread(move || {
        let Some(window) = main_thread_handle.get_webview_window("main") else {
            eprintln!("failed to find main window");
            return;
        };
        if platform::main_panel_is_visible(&main_thread_handle, &window).unwrap_or(false) {
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
        if let Err(error) = platform::remember_frontmost_application(&main_thread_handle) {
            eprintln!("failed to remember paste target application: {error}");
        }
        if let Err(error) = crate::core::formatter_window::destroy_unpinned(&main_thread_handle) {
            eprintln!("failed to close unpinned text formatter windows: {error}");
        }
        if let Err(error) = show_main_panel_now(&main_thread_handle, &window, cursor_position) {
            eprintln!("failed to show main panel: {error}");
        }
        main_thread_handle
            .state::<crate::core::formatter_window::TextFormatterState>()
            .set_main_panel_activation(false);
        println!(
            "[main-panel] show requested; visible={}",
            platform::main_panel_is_visible(&main_thread_handle, &window).unwrap_or(false)
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
pub async fn show_settings_window(app_handle: AppHandle) -> CmdResult {
    run_main_panel_task(app_handle, |handle| {
        let window = handle
            .get_webview_window("setting")
            .ok_or_else(|| "failed to find settings window".to_owned())?;
        hide_main_panel_now(handle)?;
        window
            .unminimize()
            .and_then(|_| window.show())
            .and_then(|_| window.set_focus())
            .map_err(|error| format!("failed to show settings window: {error}"))
    })
    .await
}

#[tauri::command]
pub async fn paste_into_active_app(app_handle: AppHandle) -> CmdResult {
    run_main_panel_task(app_handle, |main_thread_handle| {
        system_permissions::ensure_automatic_paste_permission()?;
        let target_identifier = main_thread_handle
            .state::<platform::PasteTargetState>()
            .get()?;
        hide_main_panel_now(main_thread_handle)?;
        platform::post_paste_shortcut(target_identifier)
    })
    .await
}
