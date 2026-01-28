use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Serialize, Deserialize)]
pub struct ScreenshotResult {
    pub width: usize,
    pub height: usize,
    pub data: String, // base64 encoded PNG
}

#[tauri::command]
fn take_screenshot_interactive() -> Result<ScreenshotResult, String> {
    // Use macOS screencapture command with interactive selection (-i)
    // This gives exactly the same UX as Cmd+Shift+4
    let temp_path = "/tmp/flashcap_screenshot.png";

    // Remove old file if exists
    let _ = std::fs::remove_file(temp_path);

    // Run screencapture -i (interactive selection mode)
    let status = Command::new("screencapture")
        .args(["-i", temp_path])
        .status()
        .map_err(|e| format!("Failed to run screencapture: {}", e))?;

    if !status.success() {
        return Err("screencapture was cancelled or failed".to_string());
    }

    // Check if file was created (user might have pressed Escape)
    if !std::path::Path::new(temp_path).exists() {
        return Err("Screenshot was cancelled".to_string());
    }

    // Read the PNG file
    let png_data = std::fs::read(temp_path)
        .map_err(|e| format!("Failed to read screenshot: {}", e))?;

    // Get image dimensions
    let img = image::load_from_memory(&png_data)
        .map_err(|e| format!("Failed to decode image: {}", e))?;

    let width = img.width() as usize;
    let height = img.height() as usize;

    // Clean up temp file
    let _ = std::fs::remove_file(temp_path);

    Ok(ScreenshotResult {
        width,
        height,
        data: STANDARD.encode(&png_data),
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![take_screenshot_interactive])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
