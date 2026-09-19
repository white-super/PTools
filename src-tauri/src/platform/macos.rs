use super::{
    apply_main_panel_layout, MainPanelState, MAIN_PANEL_LABEL, WINDOW_MOVED_EVENT,
    WINDOW_RESIZED_EVENT,
};
use core_foundation::{
    base::TCFType,
    boolean::CFBoolean,
    dictionary::{CFDictionary, CFDictionaryRef},
    string::{CFString, CFStringRef},
};
use std::process::Command;
use tauri::{
    image::Image,
    menu::{Menu, MenuBuilder, SubmenuBuilder},
    tray::TrayIconEvent,
    App, AppHandle, Builder, Emitter, EventTarget, Manager, PhysicalPosition, Runtime,
    TitleBarStyle, WebviewWindow, WebviewWindowBuilder, Wry,
};
use tauri_nspanel::{
    cocoa::appkit::NSWindowCollectionBehavior, panel_delegate, ManagerExt, WebviewWindowExt,
};

pub(crate) const DEFAULT_MAIN_SHORTCUT: &str = "Ctrl+V";
pub(crate) const MAIN_PANEL_INITIAL_WIDTH: f64 = 1.0;
pub(crate) const MAIN_PANEL_INITIAL_HEIGHT: f64 = 1.0;
pub(crate) const TRAY_ICON: Image<'_> = tauri::include_image!("./icons/menu-bar-template.png");
pub(crate) const TRAY_ICON_IS_TEMPLATE: bool = true;
pub(crate) const TRAY_MENU_ON_LEFT_CLICK: bool = true;
pub(crate) use super::macos_paste::{
    post_paste_shortcut, refresh_paste_target, remember_frontmost_application,
};

pub(crate) fn tray_event_opens_settings(_event: &TrayIconEvent) -> bool {
    false
}

const NS_POP_UP_MENU_WINDOW_LEVEL: i32 = 101;
const NS_WINDOW_STYLE_MASK_NON_ACTIVATING_PANEL: i32 = 1 << 7;
const ACCESSIBILITY_SETTINGS_URL: &str =
    "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility";

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXIsProcessTrustedWithOptions(options: CFDictionaryRef) -> bool;
    static kAXTrustedCheckOptionPrompt: CFStringRef;
    fn CGPreflightPostEventAccess() -> bool;
    fn CGRequestPostEventAccess() -> bool;
}

pub(crate) fn configure_builder(builder: Builder<Wry>) -> Builder<Wry> {
    builder.menu(create_edit_menu).plugin(tauri_nspanel::init())
}

fn create_edit_menu(app: &AppHandle) -> tauri::Result<Menu<Wry>> {
    let edit_menu = SubmenuBuilder::new(app, "Edit")
        .undo()
        .redo()
        .separator()
        .cut()
        .copy()
        .paste()
        .select_all()
        .build()?;
    MenuBuilder::new(app).item(&edit_menu).build()
}

pub(crate) fn configure_main_window_builder<'a, R, M>(
    builder: WebviewWindowBuilder<'a, R, M>,
) -> WebviewWindowBuilder<'a, R, M>
where
    R: Runtime,
    M: Manager<R>,
{
    builder
        .hidden_title(true)
        .title_bar_style(TitleBarStyle::Transparent)
}

pub(crate) fn initialize_main_panel(app: &mut App, window: WebviewWindow) -> Result<(), String> {
    let panel = window
        .to_panel()
        .map_err(|error| format!("failed to convert main window to panel: {error}"))?;
    panel.set_floating_panel(true);
    panel.set_level(NS_POP_UP_MENU_WINDOW_LEVEL);
    panel.set_style_mask(NS_WINDOW_STYLE_MASK_NON_ACTIVATING_PANEL);
    panel.set_hides_on_deactivate(false);
    panel.set_collection_behaviour(
        NSWindowCollectionBehavior::NSWindowCollectionBehaviorCanJoinAllSpaces
            | NSWindowCollectionBehavior::NSWindowCollectionBehaviorFullScreenAuxiliary,
    );
    let monitor_window = window.clone();
    let delegate = panel_delegate!(MyPanelDelegate {
        window_did_resign_key
    });
    let app_handle = app.handle().clone();
    delegate.set_listener(Box::new(move |delegate_name: String| {
        let target = EventTarget::labeled(MAIN_PANEL_LABEL);
        let window_move_event = || {
            if let Ok(position) = window.outer_position() {
                let _ = window.emit_to(target.clone(), WINDOW_MOVED_EVENT, position);
            }
        };

        match delegate_name.as_str() {
            "window_did_resign_key" => {
                let panel_state = app_handle.state::<MainPanelState>();
                if !panel_state.take_blur_suppression() {
                    if let Err(error) = crate::cmds::hide_main_panel_now(&app_handle) {
                        eprintln!("failed to dismiss the main panel after losing key status: {error}");
                    }
                }
            }
            "window_did_resize" => {
                window_move_event();
                if let Ok(size) = window.inner_size() {
                    let _ = window.emit_to(target, WINDOW_RESIZED_EVENT, size);
                }
            }
            "window_did_move" => window_move_event(),
            _ => (),
        }
    }));
    panel.set_delegate(delegate);
    if let Err(error) = super::macos_event_monitor::install_mouse_dismiss_monitors(
        app.handle(),
        &monitor_window,
    ) {
        eprintln!("failed to install macOS outside-click monitors: {error}");
    }
    Ok(())
}

pub(crate) fn main_panel_is_visible(
    app_handle: &AppHandle,
    _window: &WebviewWindow,
) -> Result<bool, String> {
    app_handle
        .get_webview_panel(MAIN_PANEL_LABEL)
        .map(|panel| panel.is_visible())
        .map_err(|error| format!("failed to find main panel: {error:?}"))
}

pub(crate) fn show_main_panel(
    app_handle: &AppHandle,
    window: &WebviewWindow,
    cursor_position: PhysicalPosition<f64>,
) -> Result<(), String> {
    apply_main_panel_layout(window, cursor_position)?;
    let panel = app_handle
        .get_webview_panel(MAIN_PANEL_LABEL)
        .map_err(|error| format!("failed to find main panel: {error:?}"))?;
    panel.show();
    Ok(())
}

pub(crate) fn hide_main_panel(
    app_handle: &AppHandle,
    _window: &WebviewWindow,
) -> Result<(), String> {
    let panel = app_handle
        .get_webview_panel(MAIN_PANEL_LABEL)
        .map_err(|error| format!("failed to find main panel: {error:?}"))?;
    panel.order_out(None);
    Ok(())
}

pub(crate) fn configure_settings_window_builder<'a, R, M>(
    builder: WebviewWindowBuilder<'a, R, M>,
) -> WebviewWindowBuilder<'a, R, M>
where
    R: Runtime,
    M: Manager<R>,
{
    builder.title_bar_style(TitleBarStyle::Transparent)
}

pub(crate) fn initialize_help_window(window: WebviewWindow) -> Result<(), String> {
    let panel = window
        .to_panel()
        .map_err(|error| format!("failed to convert shortcut help window to panel: {error:?}"))?;
    panel.set_floating_panel(true);
    panel.set_becomes_key_only_if_needed(true);
    panel.set_level(NS_POP_UP_MENU_WINDOW_LEVEL);
    panel.set_style_mask(NS_WINDOW_STYLE_MASK_NON_ACTIVATING_PANEL);
    panel.set_hides_on_deactivate(false);
    panel.set_collection_behaviour(
        NSWindowCollectionBehavior::NSWindowCollectionBehaviorCanJoinAllSpaces
            | NSWindowCollectionBehavior::NSWindowCollectionBehaviorFullScreenAuxiliary,
    );
    Ok(())
}

pub(crate) fn show_help_window(app_handle: &AppHandle, label: &str) -> Result<(), String> {
    let panel = app_handle
        .get_webview_panel(label)
        .map_err(|error| format!("failed to find shortcut help panel: {error:?}"))?;
    panel.order_front_regardless();
    Ok(())
}

pub(crate) fn hide_help_window(app_handle: &AppHandle, label: &str) -> Result<(), String> {
    let panel = app_handle
        .get_webview_panel(label)
        .map_err(|error| format!("failed to find shortcut help panel: {error:?}"))?;
    panel.order_out(None);
    Ok(())
}

pub(crate) fn finish_setup(app: &mut App) {
    app.set_activation_policy(tauri::ActivationPolicy::Accessory);
}

pub(crate) fn accessibility_permission_granted() -> bool {
    unsafe { AXIsProcessTrustedWithOptions(std::ptr::null()) && CGPreflightPostEventAccess() }
}

pub(crate) fn request_accessibility_permission() {
    if accessibility_permission_granted() {
        return;
    }

    let prompt_key = unsafe { CFString::wrap_under_get_rule(kAXTrustedCheckOptionPrompt) };
    let prompt_options = CFDictionary::from_CFType_pairs(&[(prompt_key, CFBoolean::true_value())]);
    unsafe {
        AXIsProcessTrustedWithOptions(prompt_options.as_concrete_TypeRef());
        CGRequestPostEventAccess();
    }
}

pub(crate) fn open_accessibility_settings() -> Result<(), String> {
    let status = Command::new("open")
        .arg(ACCESSIBILITY_SETTINGS_URL)
        .status()
        .map_err(|error| format!("failed to open macOS accessibility settings: {error}"))?;
    if status.success() {
        return Ok(());
    }
    Err("failed to open macOS accessibility settings".to_owned())
}
