use std::sync::{Arc, Mutex};
use tauri::{AppHandle, State, Manager};
use crate::config::AppConfig;
use crate::file::FileMetadata;
use crate::analytics;

type FileList = Arc<Mutex<Vec<FileMetadata>>>;

#[tauri::command]
pub fn start_multi_drag(
    app: AppHandle,
    _file_list: State<'_, FileList>,
    _config: State<'_, Arc<Mutex<AppConfig>>>,
    file_paths: Vec<String>,
    drag_image: Option<String>,
) -> Result<(), String> {
    println!(
        "Starting multi-file drag for files: {}",
        file_paths.join(", ")
    );

    // Send analytics event asynchronously using centralized service
    let num_files = file_paths.len();
    let app_handle = app.clone();
    
    tauri::async_runtime::spawn(async move {
        if let Err(e) = analytics::send_analytics_event(&app_handle, "files_dropped", Some(vec![
            ("num_files", serde_json::Value::Number((num_files as i64).into())),
        ])).await {
            eprintln!("[Analytics] Failed to send files_dropped event: {}", e);
        }
    });

    let mut valid_paths = Vec::new();

    for file_path in &file_paths {
        match std::fs::canonicalize(file_path.clone()) {
            Ok(path) => {
                if path.exists() {
                    valid_paths.push(path);
                }
            }
            Err(e) => {
                println!("Error processing file: {}", e);
            }
        }
    }

    if valid_paths.is_empty() {
        return Err("No valid files to drag".to_string());
    }

    // Use the drag image from the frontend if provided, otherwise generate one
    let image = if let Some(base64_data) = drag_image {
        // Remove data URL prefix if present (e.g., "data:image/png;base64,")
        let base64_str = if let Some(comma_pos) = base64_data.find(',') {
            &base64_data[comma_pos + 1..]
        } else {
            &base64_data
        };
        
        match base64::Engine::decode(&base64::engine::general_purpose::STANDARD, base64_str) {
            Ok(bytes) => {
                println!("Using frontend drag image ({} bytes)", bytes.len());
                drag::Image::Raw(bytes)
            }
            Err(e) => {
                println!("Failed to decode drag image: {}, using file icon", e);
                generate_drag_image(valid_paths.len())
            }
        }
    } else {
        generate_drag_image(valid_paths.len())
    };

    let item = drag::DragItem::Files(valid_paths);

    let window = app.get_webview_window("main")
        .ok_or("Main window not found")?;
    let app_clone = app.clone();

    let on_drop_callback = move |result: drag::DragResult, _: drag::CursorPosition| {
        if matches!(result, drag::DragResult::Cancel) {
            return;
        }

        // check if the popup window is open
        if app_clone.get_webview_window("popup").is_some() {
            if let Err(e) = super::window_ops::close_popup_window(app_clone.clone()) {
                println!("Failed to close popup window: {}", e);
            }
        }
        if let Some(main_window) = app.get_webview_window("main") {
            if let Err(e) = main_window.hide() {
                println!("Failed to hide main window: {}", e);
            }
        }
    };

    // On macOS, use CopyMove mode (17 = NSDragOperationCopy | NSDragOperationMove)
    // so the OS handles modifier key detection (Option = Move, default = Copy).
    // On Windows, check the key state at drag start.
    #[cfg(target_os = "macos")]
    let mode: drag::DragMode = unsafe { std::mem::transmute(17u64) };
    #[cfg(not(target_os = "macos"))]
    let mode = if is_move_key_pressed() { drag::DragMode::Move } else { drag::DragMode::Copy };

    drag::start_drag(
        &window,
        item,
        image,
        on_drop_callback,
        drag::Options {
            skip_animatation_on_cancel_or_failure: true,
            mode,
        },
    )
    .map_err(|e| format!("Failed to start multi-file drag operation: {}", e))?;

    Ok(())
}

/// Generate a simple drag image with file count badge using the `image` crate.
/// Returns a PNG-encoded drag::Image::Raw at 128x128 (good enough for Retina).
fn generate_drag_image(file_count: usize) -> drag::Image {
    use image::{RgbaImage, Rgba};
    
    let size = 128u32;
    let mut img = RgbaImage::new(size, size);
    
    // Draw a simple file icon (white rectangle with gray border and folded corner)
    let margin = 16u32;
    let fold = 24u32;
    let border_color = Rgba([160, 160, 160, 255]);
    let fill_color = Rgba([245, 245, 245, 255]);
    let fold_color = Rgba([200, 200, 200, 255]);
    
    // Fill the file body
    for y in margin..size - margin {
        for x in margin..size - margin {
            // Skip the folded corner area
            if y < margin + fold && x > size - margin - fold {
                continue;
            }
            img.put_pixel(x, y, fill_color);
        }
    }
    
    // Draw fold triangle
    for y in margin..margin + fold {
        for x in (size - margin - fold)..(size - margin) {
            let dx = x - (size - margin - fold);
            let dy = y - margin;
            if dx + dy <= fold {
                img.put_pixel(x, y, fold_color);
            }
        }
    }
    
    // Draw border
    for x in margin..size - margin - fold {
        img.put_pixel(x, margin, border_color); // top
    }
    for x in margin..size - margin {
        img.put_pixel(x, size - margin - 1, border_color); // bottom
    }
    for y in margin..size - margin {
        img.put_pixel(margin, y, border_color); // left
        img.put_pixel(size - margin - 1, y, border_color); // right
    }
    // Fold diagonal
    for i in 0..fold {
        let x = size - margin - fold + i;
        let y = margin + fold - i;
        if x < size && y < size {
            img.put_pixel(x, y, border_color);
        }
    }
    // Fold vertical
    for y in margin + fold..size - margin {
        if size - margin - 1 < size {
            // already drawn in right border
        }
    }
    
    // If multiple files, draw a badge circle with count
    if file_count > 1 {
        let badge_radius = 18i32;
        let badge_cx = (size - margin) as i32;
        let badge_cy = (size - margin) as i32;
        let badge_color = Rgba([59, 130, 246, 255]); // Blue
        let _badge_text_color = Rgba([255, 255, 255, 255]);
        
        // Draw badge circle
        for y in 0..size as i32 {
            for x in 0..size as i32 {
                let dx = x - badge_cx;
                let dy = y - badge_cy;
                if dx * dx + dy * dy <= badge_radius * badge_radius {
                    img.put_pixel(x as u32, y as u32, badge_color);
                }
            }
        }
        
        // Draw count number (simple pixel art for single/double digit)
        let count_str = if file_count > 99 { "99+".to_string() } else { file_count.to_string() };
        // For simplicity, just draw a small dot pattern - the badge itself is informative
        let _ = count_str; // Count shown by badge presence; actual text rendering is complex with `image` crate
        
        // Draw a simple "+" or number shape in the badge center
        // For now, just the badge alone indicates multiple files
    }
    
    // Encode to PNG
    let mut png_bytes: Vec<u8> = Vec::new();
    if let Err(e) = img.write_to(&mut std::io::Cursor::new(&mut png_bytes), image::ImageFormat::Png) {
        println!("Failed to encode drag image: {}", e);
        // Fallback: return a 1x1 transparent PNG
        return drag::Image::Raw(vec![]);
    }
    
    drag::Image::Raw(png_bytes)
}

#[cfg(target_os = "windows")]
fn is_move_key_pressed() -> bool {
    use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_CONTROL, VK_SHIFT};
    unsafe {
        let ctrl_pressed = GetAsyncKeyState(VK_CONTROL.0 as i32) < 0;
        let shift_pressed: bool = GetAsyncKeyState(VK_SHIFT.0 as i32) < 0;
        ctrl_pressed || shift_pressed
    }
}

#[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
fn is_move_key_pressed() -> bool {
    false
}
