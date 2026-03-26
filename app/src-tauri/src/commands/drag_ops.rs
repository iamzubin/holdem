use crate::config::AppConfig;
use crate::file::FileMetadata;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager, State};

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
        "[Drag] Starting multi-file drag for {} files",
        file_paths.len()
    );
    println!("[Drag] File paths: {:?}", file_paths);

    let mut valid_paths = Vec::new();

    for file_path in &file_paths {
        println!("[Drag] Processing path: {}", file_path);
        match std::fs::canonicalize(file_path.clone()) {
            Ok(path) => {
                if path.exists() {
                    println!("[Drag] Valid path: {:?}", path);
                    valid_paths.push(path);
                } else {
                    println!("[Drag] Path does not exist: {:?}", path);
                }
            }
            Err(e) => {
                println!("[Drag ERROR] Error canonicalizing path: {}", e);
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
                println!("[Drag] Using frontend drag image ({} bytes)", bytes.len());
                drag::Image::Raw(bytes)
            }
            Err(e) => {
                println!("[Drag] Failed to decode drag image: {}, using file icon", e);
                generate_drag_image(valid_paths.len())
            }
        }
    } else {
        generate_drag_image(valid_paths.len())
    };

    let item = drag::DragItem::Files(valid_paths.clone());
    println!("[Drag] Created DragItem with {} files", valid_paths.len());

    let window = app
        .get_webview_window("main")
        .ok_or("Main window not found")?;
    println!("[Drag] Got main window handle");

    // Ensure window is shown and activated for drag to work on macOS
    #[cfg(target_os = "macos")]
    {
        if let Err(e) = window.show() {
            println!("[Drag WARNING] Failed to show window before drag: {}", e);
        }
        if let Err(e) = window.set_focus() {
            println!("[Drag WARNING] Failed to focus window before drag: {}", e);
        }
        // Small delay to ensure window is properly activated
        std::thread::sleep(std::time::Duration::from_millis(50));
        println!("[Drag] Window prepared for drag");
    }

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
        #[cfg(target_os = "macos")]
        {
            return;
        }
        #[cfg(not(target_os = "macos"))]
        if let Some(main_window) = app_clone.get_webview_window("main") {
            if let Err(e) = main_window.hide() {
                println!("Failed to hide main window: {}", e);
            }
        }
    };

    // On macOS, the drag crate only supports Copy or Move individually, not combined.
    // Using Copy as default (standard macOS behavior where Option key changes to Move).
    #[cfg(target_os = "macos")]
    let mode = drag::DragMode::Copy;
    #[cfg(not(target_os = "macos"))]
    let mode = if is_move_key_pressed() {
        drag::DragMode::Move
    } else {
        drag::DragMode::Copy
    };

    match drag::start_drag(
        &window,
        item,
        image,
        on_drop_callback,
        drag::Options {
            skip_animatation_on_cancel_or_failure: true,
            mode,
        },
    ) {
        Ok(_) => {
            println!("[Drag] Drag operation started successfully");
            Ok(())
        }
        Err(e) => {
            println!("[Drag ERROR] Failed to start drag: {:?}", e);
            Err(format!(
                "Failed to start multi-file drag operation: {:?}",
                e
            ))
        }
    }
}

/// Generate a simple drag image with file count badge using the `image` crate.
/// Returns a PNG-encoded drag::Image::Raw at 128x128 (good enough for Retina).
fn generate_drag_image(file_count: usize) -> drag::Image {
    use image::{Rgba, RgbaImage};

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
        let count_str = if file_count > 99 {
            "99+".to_string()
        } else {
            file_count.to_string()
        };
        // For simplicity, just draw a small dot pattern - the badge itself is informative
        let _ = count_str; // Count shown by badge presence; actual text rendering is complex with `image` crate

        // Draw a simple "+" or number shape in the badge center
        // For now, just the badge alone indicates multiple files
    }

    // Encode to PNG
    let mut png_bytes: Vec<u8> = Vec::new();
    if let Err(e) = img.write_to(
        &mut std::io::Cursor::new(&mut png_bytes),
        image::ImageFormat::Png,
    ) {
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
