use tauri::AppHandle;

pub async fn run<T: Send + 'static>(
    app: AppHandle,
    task: impl FnOnce(&AppHandle) -> Result<T, String> + Send + 'static,
) -> Result<T, String> {
    let (sender, mut receiver) = tauri::async_runtime::channel(1);
    let handle = app.clone();
    app.run_on_main_thread(move || {
        if sender.try_send(task(&handle)).is_err() {
            eprintln!("failed to return window task result");
        }
    })
    .map_err(|error| error.to_string())?;
    receiver
        .recv()
        .await
        .ok_or("window task ended without a result")?
}
