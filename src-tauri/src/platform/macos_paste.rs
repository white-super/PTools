use super::PasteTargetState;
use cocoa::base::{id, nil};
use core_graphics::{
    event::{CGEvent, CGEventFlags, CGEventTapLocation},
    event_source::{CGEventSource, CGEventSourceStateID},
};
use objc::{class, msg_send, sel, sel_impl};
use std::time::Duration;
use tauri::{AppHandle, Manager};

const NS_APPLICATION_ACTIVATE_IGNORING_OTHER_APPS: usize = 1 << 1;
const PASTE_FOCUS_DELAY: Duration = Duration::from_millis(40);
const PASTE_KEY_CODE: u16 = 0x09;

fn frontmost_application_process_id() -> Result<i32, String> {
    unsafe {
        let workspace: id = msg_send![class!(NSWorkspace), sharedWorkspace];
        let application: id = msg_send![workspace, frontmostApplication];
        if workspace == nil || application == nil {
            return Err("failed to identify the frontmost application".to_owned());
        }
        let process_id: i32 = msg_send![application, processIdentifier];
        if process_id <= 0 {
            return Err("failed to identify the frontmost application process".to_owned());
        }
        Ok(process_id)
    }
}

pub(crate) fn remember_frontmost_application(app_handle: &AppHandle) -> Result<(), String> {
    let process_id = frontmost_application_process_id()?;
    if process_id == std::process::id() as i32 {
        return Err("PTools 不能作为粘贴目标".to_owned());
    }
    println!("[paste] remembered target process: {process_id}");
    app_handle
        .state::<PasteTargetState>()
        .replace(process_id as isize)
}

pub(crate) fn refresh_paste_target(app_handle: &AppHandle) -> Result<(), String> {
    let process_id = frontmost_application_process_id()?;
    if process_id == std::process::id() as i32 {
        return Ok(());
    }
    app_handle
        .state::<PasteTargetState>()
        .replace(process_id as isize)
}

fn activate_application(process_id: i32) -> Result<(), String> {
    unsafe {
        let application: id = msg_send![
            class!(NSRunningApplication),
            runningApplicationWithProcessIdentifier: process_id
        ];
        if application == nil {
            return Err("the remembered macOS paste target no longer exists".to_owned());
        }
        let activated: bool = msg_send![
            application,
            activateWithOptions: NS_APPLICATION_ACTIVATE_IGNORING_OTHER_APPS
        ];
        if !activated {
            return Err("macOS prevented PTools from restoring the paste target".to_owned());
        }
    }
    std::thread::sleep(PASTE_FOCUS_DELAY);
    if frontmost_application_process_id()? != process_id {
        return Err("the paste target application did not regain focus".to_owned());
    }
    Ok(())
}

pub(crate) fn post_paste_shortcut(target_identifier: isize) -> Result<(), String> {
    let target_process_id = i32::try_from(target_identifier)
        .map_err(|_| "the remembered macOS paste target is invalid".to_owned())?;
    activate_application(target_process_id)?;
    let source = CGEventSource::new(CGEventSourceStateID::CombinedSessionState)
        .map_err(|_| "failed to create the macOS keyboard event source".to_owned())?;
    let key_down = CGEvent::new_keyboard_event(source.clone(), PASTE_KEY_CODE, true)
        .map_err(|_| "failed to create the Command+V key-down event".to_owned())?;
    let key_up = CGEvent::new_keyboard_event(source, PASTE_KEY_CODE, false)
        .map_err(|_| "failed to create the Command+V key-up event".to_owned())?;
    key_down.set_flags(CGEventFlags::CGEventFlagCommand);
    key_up.set_flags(CGEventFlags::CGEventFlagCommand);
    key_down.post(CGEventTapLocation::HID);
    key_up.post(CGEventTapLocation::HID);
    println!("[paste] sent Command+V to macOS target process: {target_process_id}");
    Ok(())
}
