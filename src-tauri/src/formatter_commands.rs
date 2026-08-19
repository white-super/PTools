use tauri::{AppHandle, Manager, State};

use crate::core::formatter_window::{self, TextFormatterInput, TextFormatterState};

type CommandResult<T = ()> = Result<T, String>;

async fn run_formatter_window_task<T>(
    app_handle: AppHandle,
    task: impl FnOnce(&AppHandle) -> CommandResult<T> + Send + 'static,
) -> CommandResult<T>
where
    T: Send + 'static,
{
    let (sender, mut receiver) = tauri::async_runtime::channel(1);
    let main_thread_handle = app_handle.clone();
    app_handle
        .run_on_main_thread(move || {
            if sender.try_send(task(&main_thread_handle)).is_err() {
                eprintln!("failed to return text formatter window task result");
            }
        })
        .map_err(|error| format!("failed to schedule text formatter window task: {error}"))?;
    receiver
        .recv()
        .await
        .ok_or_else(|| "text formatter window task ended before returning a result".to_owned())?
}

#[tauri::command]
pub async fn show_text_formatter(
    app_handle: AppHandle,
    input: TextFormatterInput,
) -> CommandResult<String> {
    run_formatter_window_task(app_handle, move |main_thread_handle| {
        let formatter_state = main_thread_handle.state::<TextFormatterState>();
        formatter_state.set_main_panel_activation(true);
        let result = (|| {
            crate::cmds::hide_main_panel_now(main_thread_handle)?;
            formatter_window::create_for_input(main_thread_handle, input)
        })();
        formatter_state.set_main_panel_activation(false);
        result
    })
    .await
}

#[tauri::command]
pub fn get_text_formatter_input(
    window_id: String,
    state: State<'_, TextFormatterState>,
) -> CommandResult<TextFormatterInput> {
    state
        .current_input(&window_id)?
        .ok_or_else(|| format!("text formatter input not found: {window_id}"))
}

#[tauri::command]
pub fn get_text_formatter_pinned(
    window_id: String,
    state: State<'_, TextFormatterState>,
) -> CommandResult<bool> {
    state.is_pinned(&window_id)
}

#[tauri::command]
pub async fn set_text_formatter_pinned(
    app_handle: AppHandle,
    window_id: String,
    pinned: bool,
) -> CommandResult<bool> {
    run_formatter_window_task(app_handle, move |main_thread_handle| {
        formatter_window::set_pinned(main_thread_handle, &window_id, pinned)
    })
    .await?;
    Ok(pinned)
}

#[tauri::command]
pub async fn close_text_formatter(app_handle: AppHandle, window_id: String) -> CommandResult {
    run_formatter_window_task(app_handle, move |main_thread_handle| {
        formatter_window::destroy(main_thread_handle, &window_id)
    })
    .await
}
