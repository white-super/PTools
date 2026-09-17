use crate::{cmds, platform, storage::AppTheme};
use std::str::FromStr;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{App, AppHandle, Manager, Theme, WebviewUrl, WebviewWindowBuilder, WindowEvent};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

const SETTINGS_WINDOW_INITIAL_WIDTH: f64 = 1100.0;
const SETTINGS_WINDOW_INITIAL_HEIGHT: f64 = 760.0;
const SETTINGS_WINDOW_MIN_WIDTH: f64 = 1024.0;
const SETTINGS_WINDOW_MIN_HEIGHT: f64 = 680.0;
#[derive(Debug, Default, Clone)]
pub struct Handle {}

impl Handle {
    pub fn set_app_theme(app_handle: &AppHandle, app_theme: AppTheme) {
        let native_theme = match app_theme {
            AppTheme::Dark => Theme::Dark,
            AppTheme::SoftGlow | AppTheme::Classic => Theme::Light,
        };
        app_handle.set_theme(Some(native_theme));
    }

    pub fn create_main_window(app: &mut App) -> Result<(), String> {
        let win_builder =
            WebviewWindowBuilder::new(app, platform::MAIN_PANEL_LABEL, WebviewUrl::default())
                .title("PTools")
                .position(0.0, 0.0)
                .decorations(false)
                .resizable(false)
                .fullscreen(false)
                .always_on_top(true)
                .visible(false)
                .skip_taskbar(true)
                .closable(false)
                .minimizable(false)
                .visible_on_all_workspaces(true)
                .inner_size(
                    platform::MAIN_PANEL_INITIAL_WIDTH,
                    platform::MAIN_PANEL_INITIAL_HEIGHT,
                );

        let window = platform::configure_main_window_builder(win_builder)
            .build()
            .map_err(|error| format!("failed to create main window: {error}"))?;

        platform::initialize_main_panel(app, window)
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
                .center();
        let window = platform::configure_settings_window_builder(win_builder)
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
                        Ok(false) => {
                            let app_handle = app_handle.clone();
                            tauri::async_runtime::spawn(async move {
                                cmds::toggle_window(app_handle);
                            });
                        }
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

    fn show_setting_window(app_handle: &AppHandle) {
        let Some(window) = app_handle.get_webview_window("setting") else {
            eprintln!("failed to show settings window: window was not found");
            return;
        };
        if let Err(error) = window
            .unminimize()
            .and_then(|_| window.show())
            .and_then(|_| window.set_focus())
        {
            eprintln!("failed to show settings window: {error}");
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
            .menu_on_left_click(platform::TRAY_MENU_ON_LEFT_CLICK)
            .on_menu_event(|app, event| match event.id.as_ref() {
                "quit" => {
                    app.exit(0);
                }
                "setting" => {
                    Self::toggle_setting_window(app);
                }
                _ => {}
            })
            .on_tray_icon_event(|tray, event| {
                if platform::tray_event_opens_settings(&event) {
                    Self::show_setting_window(tray.app_handle());
                }
            })
            .icon(platform::TRAY_ICON)
            .icon_as_template(platform::TRAY_ICON_IS_TEMPLATE)
            .build(app)
            .map_err(|error| format!("failed to create tray icon: {error}"))?;
        Ok(())
    }
}
