use crate::cmds;
use std::str::FromStr;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{
    image::Image, App, AppHandle, Emitter, EventTarget, Manager, TitleBarStyle, WebviewUrl,
    WebviewWindowBuilder, WindowEvent,
};
use tauri_nspanel::{cocoa::appkit::NSWindowCollectionBehavior, panel_delegate, WebviewWindowExt};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

const NS_POP_UP_MENU_WINDOW_LEVEL: i32 = 101;
const MAIN_PANEL_WINDOW_LEVEL: i32 = NS_POP_UP_MENU_WINDOW_LEVEL;
const NS_WINDOW_STYLE_MASK_NON_ACTIVATING_PANEL: i32 = 1 << 7;
const WINDOW_FOCUS_EVENT: &str = "tauri://focus";
const WINDOW_BLUR_EVENT: &str = "tauri://blur";
const WINDOW_MOVED_EVENT: &str = "tauri://move";
const WINDOW_RESIZED_EVENT: &str = "tauri://resize";
const SETTINGS_WINDOW_INITIAL_WIDTH: f64 = 1100.0;
const SETTINGS_WINDOW_INITIAL_HEIGHT: f64 = 760.0;
const SETTINGS_WINDOW_MIN_WIDTH: f64 = 1024.0;
const SETTINGS_WINDOW_MIN_HEIGHT: f64 = 680.0;
const MENU_BAR_ICON: Image<'_> = tauri::include_image!("./icons/menu-bar-template.png");
#[derive(Debug, Default, Clone)]
pub struct Handle {}

impl Handle {
    pub fn create_main_window(app: &mut App) -> Result<(), String> {
        let win_builder = WebviewWindowBuilder::new(app, "main", WebviewUrl::default())
            .title("PTools")
            .position(0.0, 0.0)
            .decorations(false)
            .resizable(false)
            .fullscreen(false)
            .always_on_top(true)
            .visible(false)
            .skip_taskbar(true)
            .hidden_title(true)
            .closable(false)
            .minimizable(false)
            .visible_on_all_workspaces(true)
            .inner_size(
                cmds::MAIN_PANEL_INITIAL_WIDTH,
                cmds::MAIN_PANEL_INITIAL_HEIGHT,
            );

        #[cfg(target_os = "macos")]
        let win_builder = win_builder.title_bar_style(TitleBarStyle::Transparent);

        let window = win_builder
            .build()
            .map_err(|error| format!("failed to create main window: {error}"))?;

        #[cfg(target_os = "macos")]
        {
            use cocoa::appkit::{NSColor, NSWindow};
            use cocoa::base::{id, nil};
            let ns_window = window
                .ns_window()
                .map_err(|error| format!("failed to access native main window: {error}"))?
                as id;
            unsafe {
                let bg_color = NSColor::colorWithRed_green_blue_alpha_(nil, 1.0, 1.0, 1.0, 1.0);
                ns_window.setBackgroundColor_(bg_color);
            }
        }

        let panel = window
            .to_panel()
            .map_err(|error| format!("failed to convert main window to panel: {error}"))?;
        panel.set_floating_panel(true);
        panel.set_level(MAIN_PANEL_WINDOW_LEVEL);
        panel.set_style_mask(NS_WINDOW_STYLE_MASK_NON_ACTIVATING_PANEL);
        panel.set_hides_on_deactivate(false);
        panel.set_collection_behaviour(
            NSWindowCollectionBehavior::NSWindowCollectionBehaviorCanJoinAllSpaces
                | NSWindowCollectionBehavior::NSWindowCollectionBehaviorFullScreenAuxiliary,
        );
        let delegate = panel_delegate!(MyPanelDelegate {
            window_did_become_key,
            window_did_resign_key
        });
        delegate.set_listener(Box::new(move |delegate_name: String| {
            let target = EventTarget::labeled("main");
            let window_move_event = || {
                if let Ok(position) = window.outer_position() {
                    let _ = window.emit_to(target.clone(), WINDOW_MOVED_EVENT, position);
                }
            };

            match delegate_name.as_str() {
                "window_did_become_key" => {
                    let _ = window.emit_to(target, WINDOW_FOCUS_EVENT, true);
                }
                "window_did_resign_key" => {
                    let _ = window.emit_to(target, WINDOW_BLUR_EVENT, true);
                }
                "window_did_resize" => {
                    window_move_event();

                    if let Ok(size) = window.inner_size() {
                        let _ = window.emit_to(target, WINDOW_RESIZED_EVENT, size);
                    }
                }
                // 当窗口位置改变时调用
                "window_did_move" => window_move_event(),
                _ => (),
            }
        }));

        panel.set_delegate(delegate);
        Ok(())
    }

    pub fn create_setting_window(app: &mut App) -> Result<(), String> {
        let win_builder =
            WebviewWindowBuilder::new(app, "setting", WebviewUrl::App("/setting".into()))
                .title("设置")
                .always_on_top(true)
                .resizable(true)
                .fullscreen(false)
                .visible(false)
                .inner_size(
                    SETTINGS_WINDOW_INITIAL_WIDTH,
                    SETTINGS_WINDOW_INITIAL_HEIGHT,
                )
                .min_inner_size(SETTINGS_WINDOW_MIN_WIDTH, SETTINGS_WINDOW_MIN_HEIGHT)
                .title_bar_style(TitleBarStyle::Transparent)
                .center();
        let window = win_builder
            .build()
            .map_err(|error| format!("failed to create settings window: {error}"))?;
        let app_handle = app.handle().clone();
        window.on_window_event(move |event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                Self::hide_setting_window(&app_handle);
            }
        });
        Ok(())
    }

    pub fn register_shortcuts(app: &mut App, main_shortcut: &str) -> Result<(), String> {
        Self::register_main_shortcut(app.handle(), main_shortcut)
    }

    pub fn replace_main_shortcut(
        app_handle: &AppHandle,
        current_shortcut: &str,
        next_shortcut: &str,
    ) -> Result<(), String> {
        let current_shortcut = Self::parse_main_shortcut(current_shortcut)?;
        let next_shortcut = Self::parse_main_shortcut(next_shortcut)?;
        if current_shortcut == next_shortcut {
            return Ok(());
        }

        app_handle
            .global_shortcut()
            .unregister(current_shortcut)
            .map_err(|error| format!("failed to unregister current shortcut: {error}"))?;
        if let Err(error) = Self::register_shortcut(app_handle, next_shortcut) {
            let rollback_result = Self::register_shortcut(app_handle, current_shortcut);
            return Err(match rollback_result {
                Ok(()) => format!("failed to register new shortcut: {error}"),
                Err(rollback_error) => format!(
                    "failed to register new shortcut: {error}; failed to restore current shortcut: {rollback_error}"
                ),
            });
        }
        Ok(())
    }

    fn register_main_shortcut(app_handle: &AppHandle, shortcut: &str) -> Result<(), String> {
        Self::register_shortcut(app_handle, Self::parse_main_shortcut(shortcut)?)
    }

    fn parse_main_shortcut(shortcut: &str) -> Result<Shortcut, String> {
        let parsed_shortcut =
            Shortcut::from_str(shortcut).map_err(|error| format!("快捷键格式无效：{error}"))?;
        if parsed_shortcut.mods.is_empty() {
            return Err("唤醒快捷键必须包含至少一个修饰键".to_owned());
        }
        Ok(parsed_shortcut)
    }

    fn register_shortcut(app_handle: &AppHandle, shortcut: Shortcut) -> Result<(), String> {
        app_handle
            .global_shortcut()
            .on_shortcut(shortcut, move |app_handle, _shortcut, event| {
                if event.state().eq(&ShortcutState::Pressed) {
                    match Self::should_ignore_main_shortcut(app_handle) {
                        Ok(true) => (),
                        Ok(false) => cmds::toggle_window(app_handle.clone()),
                        Err(error) => {
                            eprintln!("failed to check settings window before shortcut: {error}")
                        }
                    }
                }
            })
            .map_err(|error| format!("failed to register shortcut: {error}"))
    }

    fn should_ignore_main_shortcut(app_handle: &AppHandle) -> Result<bool, String> {
        let Some(setting_window) = app_handle.get_webview_window("setting") else {
            return Ok(false);
        };
        setting_window
            .is_visible()
            .map_err(|error| format!("failed to read settings window visibility: {error}"))
    }

    fn toggle_setting_window(app_handle: &AppHandle) {
        let Some(window) = app_handle.get_webview_window("setting") else {
            eprintln!("failed to toggle settings window: window was not found");
            return;
        };
        match window.is_visible() {
            Ok(true) => Self::hide_setting_window(app_handle),
            Ok(false) => {
                if let Err(error) = window.show().and_then(|_| window.set_focus()) {
                    eprintln!("failed to show settings window: {error}");
                }
            }
            Err(error) => eprintln!("failed to read settings window visibility: {error}"),
        }
    }

    fn hide_setting_window(app_handle: &AppHandle) {
        let Some(window) = app_handle.get_webview_window("setting") else {
            eprintln!("failed to hide settings window: window was not found");
            return;
        };
        if let Err(error) = window.hide() {
            eprintln!("failed to hide settings window: {error}");
        }
    }

    pub fn create_tray_icon(app: &mut App) -> Result<(), String> {
        let setting_item = MenuItem::with_id(app, "setting", "设置", true, None::<&str>)
            .map_err(|error| format!("failed to create settings menu item: {error}"))?;
        let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)
            .map_err(|error| format!("failed to create quit menu item: {error}"))?;
        let menu = Menu::with_items(app, &[&setting_item, &quit_item])
            .map_err(|error| format!("failed to create tray menu: {error}"))?;
        TrayIconBuilder::new()
            .menu(&menu)
            .on_menu_event(|app, event| match event.id.as_ref() {
                "quit" => {
                    app.exit(0);
                }
                "setting" => {
                    Self::toggle_setting_window(app);
                }
                _ => {}
            })
            .icon(MENU_BAR_ICON)
            .icon_as_template(true)
            .build(app)
            .map_err(|error| format!("failed to create tray icon: {error}"))?;
        Ok(())
    }
}
