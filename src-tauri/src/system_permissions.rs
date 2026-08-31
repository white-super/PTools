use crate::platform::{self, SystemType};
use serde::Serialize;

type CmdResult<T = ()> = Result<T, String>;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemPermissionStatus {
    pub system_type: SystemType,
    pub system_name: &'static str,
    pub automatic_paste_supported: bool,
    pub accessibility_permission_granted: bool,
    pub accessibility_permission_supported: bool,
}

pub fn ensure_automatic_paste_permission() -> CmdResult {
    let system_type = SystemType::current();
    if !system_type.supports_automatic_paste() {
        return Err(format!(
            "automatic paste is not supported on {}",
            system_type.name()
        ));
    }
    if !system_type.supports_accessibility_permission() {
        return Ok(());
    }
    if platform::accessibility_permission_granted() {
        return Ok(());
    }

    platform::request_accessibility_permission();
    if platform::accessibility_permission_granted() {
        return Ok(());
    }
    Err(
        "自动粘贴需要系统权限。请在 macOS「系统设置 → 隐私与安全性 → 辅助功能」中允许本应用后重试"
            .to_owned(),
    )
}

#[tauri::command]
pub fn get_system_permission_status() -> SystemPermissionStatus {
    let system_type = SystemType::current();
    let accessibility_permission_supported = system_type.supports_accessibility_permission();
    SystemPermissionStatus {
        system_type,
        system_name: system_type.name(),
        automatic_paste_supported: system_type.supports_automatic_paste(),
        accessibility_permission_granted: accessibility_permission_supported
            && platform::accessibility_permission_granted(),
        accessibility_permission_supported,
    }
}

#[tauri::command]
pub fn request_system_permission() -> SystemPermissionStatus {
    platform::request_accessibility_permission();
    get_system_permission_status()
}

#[tauri::command]
pub fn open_system_permission_settings() -> CmdResult {
    platform::request_accessibility_permission();
    platform::open_accessibility_settings()
}
