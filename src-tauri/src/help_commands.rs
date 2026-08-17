use tauri::AppHandle;

use crate::core::help_window;

type CommandResult = Result<(), String>;

async fn run_help_window_task(
    app_handle: AppHandle,
    task: impl FnOnce(&AppHandle) -> CommandResult + Send + 'static,
) -> CommandResult {
    let (sender, mut receiver) = tauri::async_runtime::channel(1);
    let main_thread_handle = app_handle.clone();
    app_handle
        .run_on_main_thread(move || {
            if sender.try_send(task(&main_thread_handle)).is_err() {
                eprintln!("failed to return shortcut help window task result");
            }
        })
        .map_err(|error| format!("failed to schedule shortcut help window task: {error}"))?;
    receiver
        .recv()
        .await
        .ok_or_else(|| "shortcut help window task ended before returning a result".to_owned())?
}

#[tauri::command]
pub async fn show_shortcut_help(app_handle: AppHandle) -> CommandResult {
    run_help_window_task(app_handle, help_window::show).await
}

#[tauri::command]
pub async fn hide_shortcut_help(app_handle: AppHandle) -> CommandResult {
    run_help_window_task(app_handle, help_window::hide).await
}
