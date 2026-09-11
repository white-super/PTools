use crate::core::{
    diff_window::{self, DiffInput, DiffSource, DiffState},
    window_task,
};
use std::path::Path;
use tauri::{AppHandle, Manager, WebviewWindow};

#[tauri::command]
pub async fn show_text_diff(
    app_handle: AppHandle,
    window: WebviewWindow,
    input: DiffInput,
    pinned: bool,
) -> Result<String, String> {
    require_main_window(&window)?;
    window_task::run(app_handle, move |app| {
        crate::cmds::hide_main_panel_now(app)?;
        let result = diff_window::create(app, input, pinned);
        if result.is_err() {
            let panel = app.get_webview_window("main").ok_or("剪贴板面板不存在")?;
            let cursor = app.cursor_position().map_err(|e| e.to_string())?;
            crate::platform::show_main_panel(app, &panel, cursor)?;
        }
        result
    })
    .await
}

#[tauri::command]
pub fn get_text_diff_input(window: WebviewWindow) -> Result<DiffInput, String> {
    window.state::<DiffState>().get(window.label())
}

#[tauri::command]
pub fn get_text_diff_pinned(window: WebviewWindow) -> Result<bool, String> {
    require_diff_window(&window)?;
    window.state::<DiffState>().is_pinned(window.label())
}

#[tauri::command]
pub async fn set_text_diff_pinned(window: WebviewWindow, pinned: bool) -> Result<(), String> {
    require_diff_window(&window)?;
    let app = window.app_handle().clone();
    window_task::run(app, move |_| {
        window
            .set_always_on_top(pinned)
            .map_err(|e| e.to_string())?;
        window
            .state::<DiffState>()
            .set_pinned(window.label(), pinned)
    })
    .await
}

#[tauri::command]
pub async fn close_text_diff(window: WebviewWindow) -> Result<(), String> {
    require_diff_window(&window)?;
    let app = window.app_handle().clone();
    window_task::run(app, move |_| window.destroy().map_err(|e| e.to_string())).await
}

#[tauri::command]
pub async fn read_diff_file(window: WebviewWindow, path: String) -> Result<DiffSource, String> {
    require_main_window(&window)?;
    tauri::async_runtime::spawn_blocking(move || read_text_file(Path::new(&path)))
        .await
        .map_err(|e| e.to_string())?
}

fn require_main_window(window: &WebviewWindow) -> Result<(), String> {
    if window.label() == "main" {
        Ok(())
    } else {
        Err("此操作只能由剪贴板面板发起".into())
    }
}

fn require_diff_window(window: &WebviewWindow) -> Result<(), String> {
    if window.state::<DiffState>().contains(window.label())? {
        Ok(())
    } else {
        Err("此操作只能用于当前对比窗口".into())
    }
}

fn read_text_file(path: &Path) -> Result<DiffSource, String> {
    if !path.is_file() {
        return Err("请选择存在的文本文件，目录无法对比".to_owned());
    }
    let bytes = std::fs::read(path).map_err(|e| format!("无法读取文件：{e}"))?;
    let content = decode_text_bytes(&bytes)?;
    Ok(DiffSource {
        name: path
            .file_name()
            .ok_or("文件名无效")?
            .to_string_lossy()
            .into_owned(),
        content: content.to_owned(),
    })
}

fn decode_text_bytes(bytes: &[u8]) -> Result<&str, String> {
    let bytes = bytes.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(&bytes);
    let content =
        std::str::from_utf8(bytes).map_err(|_| "文件不是 UTF-8 文本，请转换编码后重试")?;
    if content
        .chars()
        .any(|c| c.is_control() && !matches!(c, '\t' | '\n' | '\r'))
    {
        return Err("文件包含二进制控制字符，无法进行文本对比".to_owned());
    }
    Ok(content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_utf8_bom_and_preserves_whitespace() {
        assert_eq!(decode_text_bytes(&[0xef, 0xbb, 0xbf, 65]).unwrap(), "A");
        assert_eq!(
            decode_text_bytes("中文\r\n\t".as_bytes()).unwrap(),
            "中文\r\n\t"
        );
        assert_eq!(decode_text_bytes(&[]).unwrap(), "");
    }

    #[test]
    fn rejects_binary_and_non_utf8() {
        assert!(decode_text_bytes(&[65, 0, 66])
            .unwrap_err()
            .contains("二进制"));
        assert!(decode_text_bytes(&[0xff, 0xfe, 65, 0])
            .unwrap_err()
            .contains("UTF-8"));
    }

    #[test]
    fn rejects_directories_and_missing_files() {
        assert!(read_text_file(Path::new(env!("CARGO_MANIFEST_DIR"))).is_err());
        assert!(read_text_file(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("missing-diff-fixture")
                .as_path()
        )
        .is_err());
    }
}
