use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
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

/// After a successful native drop the popup gets out of the way. The shelf
/// itself stays where it is: its files are still there (drag-out copies), and
/// hiding it would leave no visible way back to them.
fn on_drop_close_popup(app: &AppHandle, result: drag::DragResult) {
    if matches!(result, drag::DragResult::Cancel) {
        return;
    }
    super::window_ops::hide_popup(app);
}

/// Set while Holdem itself is dragging items out, so its own windows don't
/// take them back in (see drop_target.rs).
pub static OUTGOING_DRAG: AtomicBool = AtomicBool::new(false);

/// After a drop, check the shelf again: a Shift-drop moved the files away,
/// and those entries must leave the shelf. Explorer finishes a move on its
/// own thread after the drop, so look twice.
fn refresh_after_drop(app: &AppHandle) {
    super::file_ops::expect_moved_out(Duration::from_secs(5));
    let app = app.clone();
    std::thread::spawn(move || {
        for wait in [Duration::from_millis(500), Duration::from_secs(3)] {
            std::thread::sleep(wait);
            let check = super::file_ops::refresh_shelf(app.clone());
            if let Err(e) = tauri::async_runtime::block_on(check) {
                warn!("Shelf check after a drop failed: {}", e);
            }
        }
    });
}

fn launch_drag(
    app: &AppHandle,
    item: drag::DragItem,
    image: drag::Image,
    count: usize,
    mouse: bool,
) -> Result<(), String> {
    // Released while the drag was being prepared: starting now would drop
    // the items at once wherever the pointer is. (Only checkable for the
    // mouse: pen and touch contacts don't show as a held button.)
    if mouse && !crate::mouse_monitor::is_mouse_button_down() {
        return Err("The button was released before the drag started".to_string());
    }
    let window = app
        .get_webview_window("main")
        .ok_or("Main window not found")?;

    let app_clone = app.clone();
    let on_drop_callback = move |result: drag::DragResult, _: drag::CursorPosition| {
        OUTGOING_DRAG.store(false, Ordering::SeqCst);
        let dropped = !matches!(result, drag::DragResult::Cancel);
        if dropped {
            refresh_after_drop(&app_clone);
        }
        crate::analytics::track(
            &app_clone,
            "drag_out",
            vec![
                ("count", count.into()),
                ("dropped", dropped.into()),
            ],
        );
        on_drop_close_popup(&app_clone, result);
    };

    OUTGOING_DRAG.store(true, Ordering::SeqCst);
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
            OUTGOING_DRAG.store(false, Ordering::SeqCst);
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
    mouse: Option<bool>,
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

    let count = valid_paths.len();
    let image = resolve_drag_image(drag_image, count);
    info!("Prepared drag item with {} valid file(s)", count);
    let item = drag::DragItem::Files(valid_paths);

    launch_drag(&app, item, image, count, mouse.unwrap_or(true))
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

fn start_text_drag(
    app: AppHandle,
    text: String,
    drag_image: Option<String>,
    mouse: bool,
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

    launch_drag(&app, item, image, 1, mouse)
}

/// Drags a saved text snippet out as text. Reads the file here, by shelf id,
/// so the page needs no file-system access.
#[tauri::command]
pub fn start_note_drag(
    app: AppHandle,
    file_list: tauri::State<'_, crate::FileList>,
    id: u64,
    drag_image: Option<String>,
    mouse: Option<bool>,
) -> Result<(), String> {
    const MAX_NOTE_BYTES: u64 = 4 * 1024 * 1024;
    let path = {
        let list = file_list.lock().map_err(|_| "Failed to acquire lock".to_string())?;
        let file = list
            .iter()
            .find(|f| f.id == id && f.kind == "note")
            .ok_or_else(|| format!("No text snippet with ID {} on the shelf", id))?;
        file.path.clone()
    };
    let size = std::fs::metadata(&path).map_err(|e| e.to_string())?.len();
    if size > MAX_NOTE_BYTES {
        return Err("Text snippet is too large to drag as text".to_string());
    }
    let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    start_text_drag(app, text, drag_image, mouse.unwrap_or(true))
}

/// How far, in physical pixels, the pointer must move with the button held
/// before a press becomes a drag: the user's Windows setting
/// (SM_CXDRAG/SM_CYDRAG), at the DPI of the calling window.
#[tauri::command]
pub fn drag_threshold(window: tauri::WebviewWindow) -> (i32, i32) {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::HiDpi::{GetDpiForWindow, GetSystemMetricsForDpi};
    use windows::Win32::UI::WindowsAndMessaging::{SM_CXDRAG, SM_CYDRAG};
    let dpi = window
        .hwnd()
        .map(|h| unsafe { GetDpiForWindow(HWND(h.0 as _)) })
        .unwrap_or(96)
        .max(96);
    unsafe {
        (
            GetSystemMetricsForDpi(SM_CXDRAG, dpi).max(1),
            GetSystemMetricsForDpi(SM_CYDRAG, dpi).max(1),
        )
    }
}
