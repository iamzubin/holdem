use tauri::{AppHandle, Manager};
use tracing::{error, info, warn};

/// Decode the frontend-provided drag image (`data:...;base64,...` or raw
/// base64), falling back to a generated badge image for `fallback_count`.
fn resolve_drag_image(drag_image: Option<String>, fallback_count: usize) -> drag::Image {
    if let Some(base64_data) = drag_image {
        // Remove data URL prefix if present (e.g., "data:image/png;base64,")
        let base64_str = if let Some(comma_pos) = base64_data.find(',') {
            &base64_data[comma_pos + 1..]
        } else {
            &base64_data
        };

        match base64::Engine::decode(&base64::engine::general_purpose::STANDARD, base64_str) {
            Ok(bytes) => {
                info!("Using frontend-provided drag image ({} bytes)", bytes.len());
                return drag::Image::Raw(bytes);
            }
            Err(e) => {
                warn!(
                    "Failed to decode drag image, falling back to generated image: {}",
                    e
                );
            }
        }
    }
    generate_drag_image(fallback_count)
}

/// After a successful native drop the shelf UI gets out of the way:
/// the popup (if open) closes and the main window hides.
fn on_drop_hide_windows(app: &AppHandle, result: drag::DragResult) {
    if matches!(result, drag::DragResult::Cancel) {
        return;
    }

    // check if the popup window is open
    if app.get_webview_window("popup").is_some() {
        if let Err(e) = super::window_ops::close_popup_window(app.clone()) {
            error!("Failed to close popup window after drag: {}", e);
        }
    }
    if let Some(main_window) = app.get_webview_window("main") {
        if let Err(e) = main_window.hide() {
            error!("Failed to hide main window after drag: {}", e);
        }
    }
}

fn launch_drag(app: &AppHandle, item: drag::DragItem, image: drag::Image) -> Result<(), String> {
    let window = app
        .get_webview_window("main")
        .ok_or("Main window not found")?;

    let app_clone = app.clone();
    let on_drop_callback = move |result: drag::DragResult, _: drag::CursorPosition| {
        on_drop_hide_windows(&app_clone, result);
    };

    match drag::start_drag(
        &window,
        item,
        image,
        on_drop_callback,
        drag::Options {
            skip_animatation_on_cancel_or_failure: true,
            mode: drag::DragMode::CopyOrMove,
        },
    ) {
        Ok(_) => {
            info!("Native drag started successfully");
            Ok(())
        }
        Err(e) => {
            error!("Failed to start native drag: {:?}", e);
            Err(format!("Failed to start drag operation: {:?}", e))
        }
    }
}

#[tauri::command]
pub fn start_multi_drag(
    app: AppHandle,
    file_paths: Vec<String>,
    drag_image: Option<String>,
) -> Result<(), String> {
    info!("Starting native drag for {} file(s)", file_paths.len());

    let mut valid_paths = Vec::new();

    for file_path in &file_paths {
        match std::fs::canonicalize(file_path.clone()) {
            Ok(path) => {
                if path.exists() {
                    valid_paths.push(path);
                } else {
                    warn!("Skipping drag path because it does not exist: {:?}", path);
                }
            }
            Err(e) => {
                warn!("Failed to canonicalize drag path '{}': {}", file_path, e);
            }
        }
    }

    if valid_paths.is_empty() {
        return Err("No valid files to drag".to_string());
    }

    let image = resolve_drag_image(drag_image, valid_paths.len());
    let item = drag::DragItem::Files(valid_paths.clone());
    info!(
        "Prepared drag item with {} valid file(s)",
        valid_paths.len()
    );

    launch_drag(&app, item, image)
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

    // If multiple files, draw a badge circle. The badge alone signals
    // "multiple" — the `image` crate has no text rasterization, so no count
    // is drawn inside it.
    if file_count > 1 {
        let badge_radius = 18i32;
        let badge_cx = (size - margin) as i32;
        let badge_cy = (size - margin) as i32;
        let badge_color = Rgba([59, 130, 246, 255]); // Blue

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
    }

    // Encode to PNG
    let mut png_bytes: Vec<u8> = Vec::new();
    if let Err(e) = img.write_to(
        &mut std::io::Cursor::new(&mut png_bytes),
        image::ImageFormat::Png,
    ) {
        error!("Failed to encode generated drag image: {}", e);
        // Fallback: return a 1x1 transparent PNG
        return drag::Image::Raw(vec![]);
    }

    drag::Image::Raw(png_bytes)
}

#[tauri::command]
pub fn start_text_drag(
    app: AppHandle,
    text: String,
    drag_image: Option<String>,
) -> Result<(), String> {
    info!("Starting text drag");

    let image = resolve_drag_image(drag_image, 1);

    let text_clone = text.clone();
    let provider: drag::DataProvider = Box::new(move |format: &str| -> Option<Vec<u8>> {
        if format == "text/plain" {
            Some(text_clone.as_bytes().to_vec())
        } else {
            None
        }
    });

    let item = drag::DragItem::Data {
        provider,
        types: vec!["text/plain".to_string()],
    };

    launch_drag(&app, item, image)
}
