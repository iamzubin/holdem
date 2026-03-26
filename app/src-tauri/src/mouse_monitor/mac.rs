use crate::analytics;
use crate::config::MouseMonitorConfig;
use crate::DragState;
use crate::mouse_monitor::common::DRAG_PASTEBOARD_NAME;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager, PhysicalPosition};

use objc2::rc::Retained;
use objc2_foundation::{NSArray, NSString};
use objc2_app_kit::NSPasteboard;
use core_graphics_types::geometry::CGPoint;

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGEventGetLocation(event: *mut std::ffi::c_void) -> CGPoint;
    fn CGEventCreate(source: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
    fn CFRelease(cf: *mut std::ffi::c_void);
    fn CGEventSourceButtonState(stateID: u32, button: u32) -> bool;
}

fn get_cursor_position() -> (f64, f64) {
    unsafe {
        let event = CGEventCreate(std::ptr::null_mut());
        if !event.is_null() {
            let point = CGEventGetLocation(event);
            CFRelease(event);
            (point.x, point.y)
        } else {
            (0.0, 0.0)
        }
    }
}

fn is_mouse_button_down() -> bool {
    const K_CG_EVENT_SOURCE_STATE_HID_SYSTEM_STATE: u32 = 1;
    const K_CG_MOUSE_BUTTON_LEFT: u32 = 0;
    unsafe { CGEventSourceButtonState(K_CG_EVENT_SOURCE_STATE_HID_SYSTEM_STATE, K_CG_MOUSE_BUTTON_LEFT) }
}

fn get_drag_pasteboard() -> Option<Retained<NSPasteboard>> {
    let name = NSString::from_str(DRAG_PASTEBOARD_NAME);
    Some(NSPasteboard::pasteboardWithName(&name))
}

fn get_pasteboard_change_count(pasteboard: &NSPasteboard) -> i64 {
    pasteboard.changeCount() as i64
}

fn pasteboard_has_files(pasteboard: &NSPasteboard) -> bool {
    let types = match pasteboard.types() {
        Some(t) => t,
        None => return false,
    };

    if types.is_empty() {
        return false;
    }

    let file_url_type = NSString::from_str("public.file-url");
    let file_url_array = NSArray::from_slice(&[&*file_url_type]);
    let has_file_url = pasteboard.availableTypeFromArray(&file_url_array).is_some();

    let filenames_type = NSString::from_str("NSFilenamesPboardType");
    let filenames_array = NSArray::from_slice(&[&*filenames_type]);
    let has_filenames = pasteboard.availableTypeFromArray(&filenames_array).is_some();

    has_file_url || has_filenames
}

fn hide_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
}

fn show_main_window(app: &AppHandle, pos: (f64, f64), _config: &MouseMonitorConfig) {
    if let Some(window) = app.get_webview_window("main") {
        let scale_factor = window.scale_factor().unwrap_or(1.0);
        let logical_x = pos.0 / scale_factor;
        let logical_y = pos.1 / scale_factor;

        println!("[WINDOW] Physical: ({:.0}, {:.0}), Scale: {}, Logical: ({:.0}, {:.0})",
            pos.0, pos.1, scale_factor, logical_x, logical_y);

        let _ = window.set_position(PhysicalPosition {
            x: logical_x as i32,
            y: logical_y as i32,
        });
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

pub fn start_mouse_monitor(config: MouseMonitorConfig, app_handle: AppHandle, drag_state: Arc<DragState>) {
    println!("[MACOS_MONITOR] Starting with pasteboard name: {}", DRAG_PASTEBOARD_NAME);
    println!("[MACOS_MONITOR] Config: threshold={}, shakes={}, time_limit={}ms",
        config.shake_threshold, config.required_shakes, config.shake_time_limit);

    thread::spawn(move || {
        let mut window_opened_by_shake = false;
        let mut last_position = get_cursor_position();
        let check_interval = Duration::from_millis(50);
        let shake_threshold_x = config.shake_threshold as f64;
        let mut shake_count = 0u32;
        let movement_time_limit = Duration::from_millis(config.shake_time_limit);
        let mut last_shake_time = Instant::now();
        let mut last_direction: Option<i32> = None;

        let pasteboard = match get_drag_pasteboard() {
            Some(pb) => pb,
            None => {
                println!("[MACOS_MONITOR] ERROR: Could not get drag pasteboard!");
                return;
            }
        };

        let mut last_change_count = get_pasteboard_change_count(&pasteboard);
        let mut is_drag_active = false;
        let mut drag_start_time: Option<Instant> = None;

        loop {
            let current_position = get_cursor_position();
            let current_change_count = get_pasteboard_change_count(&pasteboard);
            let has_files = pasteboard_has_files(&pasteboard);
            let mouse_down = is_mouse_button_down();

            let change_count_changed = current_change_count != last_change_count && current_change_count > 0;

            // --- Detect drag start ---
            if !is_drag_active && change_count_changed && has_files {
                is_drag_active = true;
                drag_start_time = Some(Instant::now());
                last_change_count = current_change_count;
                last_shake_time = Instant::now();
                println!("[DRAG_START] File drag detected! changeCount={}", current_change_count);
            }

            // --- Detect drag end (mouse released) ---
            // Only check mouse button - pasteboard can flicker during drag
            let drag_ended = is_drag_active && !mouse_down;

            if drag_ended {
                let drag_started = drag_state.drag_started.load(Ordering::Relaxed);
                let successful_drop = drag_state.successful_drop.load(Ordering::Relaxed);

                println!("[DRAG_END] drag_started={}, successful_drop={}", drag_started, successful_drop);

                // Reset drag state flags for next drag
                drag_state.drag_started.store(false, Ordering::Relaxed);
                drag_state.successful_drop.store(false, Ordering::Relaxed);

                // If drag ended but files weren't dropped in our window, close the window
                if window_opened_by_shake && !drag_started && !successful_drop {
                    hide_main_window(&app_handle.app_handle());
                    println!("[DRAG_END] Drag ended outside window, hiding");
                }

                // Reset state
                is_drag_active = false;
                drag_start_time = None;
                shake_count = 0;
                last_direction = None;
                last_change_count = current_change_count;
                window_opened_by_shake = false;
            }

            // --- Shake detection while dragging ---
            if is_drag_active {
                let distance_x = current_position.0 - last_position.0;

                let direction = if distance_x > shake_threshold_x {
                    1
                } else if distance_x < -shake_threshold_x {
                    -1
                } else {
                    0
                };

                if direction != 0 {
                    if let Some(last_dir) = last_direction {
                        if last_dir != direction {
                            last_shake_time = Instant::now();
                            shake_count += 1;
                            println!("[SHAKE] {}/{} ({} -> {})", shake_count, config.required_shakes, last_dir, direction);
                        }
                    } else {
                        println!("[SHAKE] First movement: dir={}", direction);
                    }
                    last_direction = Some(direction);
                }

                // Reset shake if too much time passes between wiggles
                if last_shake_time.elapsed() > movement_time_limit {
                    shake_count = 0;
                    last_direction = None;
                }

                // Trigger window open on shake
                if shake_count >= config.required_shakes && !window_opened_by_shake {
                    println!("[SHAKE_DETECTED] Opening window at ({:.0}, {:.0})", current_position.0, current_position.1);

                    let app_clone = app_handle.clone();
                    let shake_count_clone = shake_count;
                    tauri::async_runtime::spawn(async move {
                        let _ = analytics::send_mouse_shake_detected_event(&app_clone, shake_count_clone).await;
                    });

                    show_main_window(&app_handle, current_position, &config);
                    window_opened_by_shake = true;

                    // Spawn timeout thread to auto-hide if no drop (and mouse is released)
                    let app_handle_clone = app_handle.clone();
                    let drag_state_clone = Arc::clone(&drag_state);
                    thread::spawn(move || {
                        thread::sleep(Duration::from_millis(2000));
                        let drag_started = drag_state_clone.drag_started.load(Ordering::Relaxed);
                        let successful_drop = drag_state_clone.successful_drop.load(Ordering::Relaxed);
                        let mouse_down = is_mouse_button_down();
                        if !drag_started && !successful_drop && !mouse_down {
                            hide_main_window(&app_handle_clone.app_handle());
                            println!("[TIMEOUT] Auto-hiding window after 2s (mouse released, no drop)");
                        }
                    });

                    let app_clone2 = app_handle.clone();
                    tauri::async_runtime::spawn(async move {
                        let _ = analytics::send_window_opened_event(&app_clone2, "main_shake").await;
                    });

                    shake_count = 0;
                    last_direction = None;
                }
            }

            last_position = current_position;
            thread::sleep(check_interval);
        }
    });
}
