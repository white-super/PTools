use serde::Serialize;

#[cfg(target_os = "macos")]
use core_foundation::{
    base::TCFType,
    boolean::CFBoolean,
    dictionary::{CFDictionary, CFDictionaryRef},
    string::{CFString, CFStringRef},
};
#[cfg(target_os = "macos")]
use std::process::Command;

type CmdResult<T = ()> = Result<T, String>;

#[cfg(target_os = "macos")]
const MACOS_ACCESSIBILITY_SETTINGS_URL: &str =
    "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility";

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SystemType {
    Macos,
    Windows,
    Linux,
    Other,
}

impl SystemType {
    fn current() -> Self {
        match std::env::consts::OS {
            "macos" => Self::Macos,
            "windows" => Self::Windows,
            "linux" => Self::Linux,
            _ => Self::Other,
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::Macos => "macOS",
            Self::Windows => "Windows",
            Self::Linux => "Linux",
            Self::Other => "未知系统",
        }
    }

    fn supports_accessibility_permission(self) -> bool {
        matches!(self, Self::Macos)
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemPermissionStatus {
    pub system_type: SystemType,
    pub system_name: &'static str,
    pub accessibility_permission_granted: bool,
    pub accessibility_permission_supported: bool,
}

#[cfg(target_os = "macos")]
#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXIsProcessTrustedWithOptions(options: CFDictionaryRef) -> bool;
    static kAXTrustedCheckOptionPrompt: CFStringRef;
    fn CGPreflightPostEventAccess() -> bool;
    fn CGRequestPostEventAccess() -> bool;
}

#[cfg(target_os = "macos")]
fn accessibility_permission_granted() -> bool {
    accessibility_client_is_trusted() && keyboard_events_are_allowed()
}

#[cfg(target_os = "macos")]
fn accessibility_client_is_trusted() -> bool {
    unsafe { AXIsProcessTrustedWithOptions(std::ptr::null()) }
}

#[cfg(target_os = "macos")]
fn keyboard_events_are_allowed() -> bool {
    unsafe { CGPreflightPostEventAccess() }
}

#[cfg(target_os = "macos")]
fn request_accessibility_permission() {
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

#[cfg(not(target_os = "macos"))]
fn accessibility_permission_granted() -> bool {
    false
}

#[cfg(target_os = "macos")]
pub fn ensure_accessibility_permission() -> CmdResult {
    if accessibility_permission_granted() {
        return Ok(());
    }

    request_accessibility_permission();
    if accessibility_permission_granted() {
        return Ok(());
    }
    Err(
        "自动粘贴需要系统权限。请在 macOS「系统设置 → 隐私与安全性 → 辅助功能」中允许本应用后重试"
            .to_owned(),
    )
}

#[cfg(not(target_os = "macos"))]
pub fn ensure_accessibility_permission() -> CmdResult {
    Err("automatic paste is currently supported only on macOS".to_owned())
}

#[cfg(not(target_os = "macos"))]
fn request_accessibility_permission() {}

#[tauri::command]
pub fn get_system_permission_status() -> SystemPermissionStatus {
    let system_type = SystemType::current();
    let accessibility_permission_supported = system_type.supports_accessibility_permission();
    SystemPermissionStatus {
        system_type,
        system_name: system_type.name(),
        accessibility_permission_granted: accessibility_permission_supported
            && accessibility_permission_granted(),
        accessibility_permission_supported,
    }
}

#[tauri::command]
pub fn request_system_permission() -> SystemPermissionStatus {
    request_accessibility_permission();
    get_system_permission_status()
}

#[cfg(target_os = "macos")]
fn open_accessibility_settings() -> CmdResult {
    let status = Command::new("open")
        .arg(MACOS_ACCESSIBILITY_SETTINGS_URL)
        .status()
        .map_err(|error| format!("failed to open macOS accessibility settings: {error}"))?;
    if status.success() {
        return Ok(());
    }
    Err("failed to open macOS accessibility settings".to_owned())
}

#[cfg(not(target_os = "macos"))]
fn open_accessibility_settings() -> CmdResult {
    Err("当前系统暂不支持打开辅助功能设置".to_owned())
}

#[tauri::command]
pub fn open_system_permission_settings() -> CmdResult {
    request_accessibility_permission();
    open_accessibility_settings()
}
