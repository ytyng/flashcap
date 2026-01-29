use base64::{engine::general_purpose::STANDARD, Engine as _};
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::process::Command;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::{Emitter, Manager, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_store::StoreExt;

#[derive(Debug, Serialize, Deserialize)]
pub struct ScreenshotResult {
    pub width: usize,
    pub height: usize,
    pub data: String,      // base64 encoded PNG
    pub file_path: String, // saved file path
}

/// 設定から保存先ディレクトリを取得する
/// "tmp" -> /tmp/flashcap/
/// "macos_default" -> macOS のスクリーンショット保存先
/// "custom:<path>" -> カスタムパス
fn get_save_directory(app: &tauri::AppHandle) -> String {
    let setting = app
        .store("settings.json")
        .ok()
        .and_then(|store| store.get("save_directory"))
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or_else(|| "tmp".to_string());

    match setting.as_str() {
        "tmp" => "/tmp/flashcap".to_string(),
        "macos_default" => get_macos_screenshot_dir(),
        s if s.starts_with("custom:") => s.strip_prefix("custom:").unwrap().to_string(),
        _ => "/tmp/flashcap".to_string(),
    }
}

/// macOS の screencapture デフォルト保存先を取得
fn get_macos_screenshot_dir() -> String {
    Command::new("defaults")
        .args(["read", "com.apple.screencapture", "location"])
        .output()
        .ok()
        .and_then(|out| {
            if out.status.success() {
                String::from_utf8(out.stdout)
                    .ok()
                    .map(|s| s.trim().to_string())
            } else {
                None
            }
        })
        .unwrap_or_else(|| {
            dirs::desktop_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| "/tmp/flashcap".to_string())
        })
}

fn get_screenshot_path(app: &tauri::AppHandle) -> String {
    let dir = std::path::PathBuf::from(get_save_directory(app));
    let _ = std::fs::create_dir_all(&dir);
    let timestamp = Local::now().format("%Y%m%d-%H%M%S");
    let filename = format!("flashcap-{}.png", timestamp);
    dir.join(filename).to_string_lossy().to_string()
}

#[tauri::command]
fn take_screenshot_interactive(
    app: tauri::AppHandle,
    window: tauri::Window,
) -> Result<ScreenshotResult, String> {
    let file_path = get_screenshot_path(&app);

    // Hide the app window so it doesn't appear in the screenshot
    let _ = window.hide();
    // ウィンドウが完全に隠れるまで少し待つ
    std::thread::sleep(std::time::Duration::from_millis(300));

    let status = Command::new("screencapture")
        .args(["-i", &file_path])
        .status()
        .map_err(|e| {
            let _ = window.show();
            format!("Failed to run screencapture: {}", e)
        })?;

    // ウィンドウを再表示
    let _ = window.show();

    if !status.success() {
        return Err("Screenshot was cancelled".to_string());
    }

    if !std::path::Path::new(&file_path).exists() {
        return Err("Screenshot was cancelled".to_string());
    }

    // 絶対パスに解決
    let absolute_path = std::fs::canonicalize(&file_path)
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or(file_path);

    let png_data = std::fs::read(&absolute_path)
        .map_err(|e| format!("Failed to read screenshot: {}", e))?;

    let img = image::load_from_memory(&png_data)
        .map_err(|e| format!("Failed to decode image: {}", e))?;

    Ok(ScreenshotResult {
        width: img.width() as usize,
        height: img.height() as usize,
        data: STANDARD.encode(&png_data),
        file_path: absolute_path,
    })
}

/// base64 PNG データをファイルに書き出す
/// パスは保存先ディレクトリ内に制限する
#[tauri::command]
fn write_image_to_file(app: tauri::AppHandle, path: String, data_base64: String) -> Result<(), String> {
    let save_dir = std::fs::canonicalize(get_save_directory(&app))
        .map_err(|e| format!("Failed to resolve save directory: {}", e))?;
    let target = std::fs::canonicalize(&path)
        .or_else(|_| {
            // ファイルが未作成の場合、親ディレクトリで検証
            std::path::Path::new(&path)
                .parent()
                .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "no parent"))
                .and_then(std::fs::canonicalize)
                .map(|p| p.join(std::path::Path::new(&path).file_name().unwrap()))
        })
        .map_err(|e| format!("Failed to resolve path: {}", e))?;

    if !target.starts_with(&save_dir) {
        return Err(format!(
            "Path '{}' is outside the save directory '{}'",
            target.display(),
            save_dir.display()
        ));
    }

    let bytes = STANDARD
        .decode(&data_base64)
        .map_err(|e| format!("Failed to decode base64: {}", e))?;
    std::fs::write(&target, &bytes).map_err(|e| format!("Failed to write file: {}", e))?;
    Ok(())
}

/// プリファレンスウィンドウを開く (既に開いていればフォーカス)
fn open_preferences_window(app: &tauri::AppHandle) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window("preferences") {
        let _ = window.set_focus();
        return Ok(());
    }

    WebviewWindowBuilder::new(app, "preferences", WebviewUrl::App("/preferences".into()))
        .title("Preferences")
        .inner_size(500.0, 350.0)
        .resizable(true)
        .center()
        .build()?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_drag::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // 既に起動中のインスタンスに対して再度起動コマンドが来た場合
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.show();
                let _ = w.set_focus();
            }
            let _ = app.emit("reactivate", ());
        }))
        .setup(|app| {
            let handle = app.handle();

            // macOS ネイティブメニュー
            let preferences =
                MenuItem::with_id(handle, "preferences", "Preferences...", true, Some("CmdOrCtrl+,"))?;

            let app_submenu = Submenu::with_items(
                handle,
                app.package_info().name.clone(),
                true,
                &[
                    &PredefinedMenuItem::about(handle, None, None)?,
                    &PredefinedMenuItem::separator(handle)?,
                    &preferences,
                    &PredefinedMenuItem::separator(handle)?,
                    &PredefinedMenuItem::services(handle, None)?,
                    &PredefinedMenuItem::separator(handle)?,
                    &PredefinedMenuItem::hide(handle, None)?,
                    &PredefinedMenuItem::hide_others(handle, None)?,
                    &PredefinedMenuItem::separator(handle)?,
                    &PredefinedMenuItem::quit(handle, None)?,
                ],
            )?;

            let edit_submenu = Submenu::with_items(
                handle,
                "Edit",
                true,
                &[
                    &PredefinedMenuItem::undo(handle, None)?,
                    &PredefinedMenuItem::redo(handle, None)?,
                    &PredefinedMenuItem::separator(handle)?,
                    &PredefinedMenuItem::cut(handle, None)?,
                    &PredefinedMenuItem::copy(handle, None)?,
                    &PredefinedMenuItem::paste(handle, None)?,
                    &PredefinedMenuItem::select_all(handle, None)?,
                ],
            )?;

            let window_submenu = Submenu::with_items(
                handle,
                "Window",
                true,
                &[
                    &PredefinedMenuItem::minimize(handle, None)?,
                    &PredefinedMenuItem::maximize(handle, None)?,
                    &PredefinedMenuItem::close_window(handle, None)?,
                ],
            )?;

            let menu = Menu::with_items(handle, &[&app_submenu, &edit_submenu, &window_submenu])?;
            app.set_menu(menu)?;

            app.on_menu_event(move |app, event| {
                if event.id() == "preferences" {
                    let _ = open_preferences_window(app);
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![take_screenshot_interactive, write_image_to_file])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            if let tauri::RunEvent::Reopen { .. } = event {
                // Dock アイコンクリック時
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.show();
                    let _ = w.set_focus();
                }
                let _ = app.emit("reactivate", ());
            }
        });
}
