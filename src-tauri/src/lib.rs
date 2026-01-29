use base64::{engine::general_purpose::STANDARD, Engine as _};
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Serialize, Deserialize)]
pub struct ScreenshotResult {
    pub width: usize,
    pub height: usize,
    pub data: String,      // base64 encoded PNG
    pub file_path: String, // saved file path
}

fn get_screenshot_path() -> String {
    let desktop = dirs::desktop_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("/tmp"));
    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let filename = format!("flashcap_{}.png", timestamp);
    desktop.join(filename).to_string_lossy().to_string()
}

#[tauri::command]
fn take_screenshot_interactive(window: tauri::Window) -> Result<ScreenshotResult, String> {
    let file_path = get_screenshot_path();

    // Hide the app window so it doesn't appear in the screenshot
    let _ = window.hide();
    // Small delay to ensure the window is fully hidden
    std::thread::sleep(std::time::Duration::from_millis(300));

    let status = Command::new("screencapture")
        .args(["-i", &file_path])
        .status()
        .map_err(|e| {
            let _ = window.show();
            format!("Failed to run screencapture: {}", e)
        })?;

    // Show the app window again
    let _ = window.show();

    if !status.success() {
        return Err("Screenshot was cancelled".to_string());
    }

    if !std::path::Path::new(&file_path).exists() {
        return Err("Screenshot was cancelled".to_string());
    }

    // Resolve to absolute path
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .invoke_handler(tauri::generate_handler![take_screenshot_interactive])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
