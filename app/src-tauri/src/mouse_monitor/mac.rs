use crate::analytics;
use crate::config::MouseMonitorConfig;
use crate::mouse_monitor::common::DRAG_PASTEBOARD_NAME;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Listener, Manager, PhysicalPosition};

use objc2::rc::Retained;
use objc2_foundation::{NSArray, NSString};
use objc2_app_kit::NSPasteboard;
use core_graphics_types::geometry::CGPoint;

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGEventGetLocation(event: *mut std::ffi::c_void) -> CGPoint;
    fn CGEventCreate(source: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
    fn CFRelease(cf: *mut std::ffi::c_void);
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

fn get_screen_size() -> (f64, f64) {
    (1920.0, 1080.0)
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

pub fn start_mouse_monitor(config: MouseMonitorConfig, app_handle: AppHandle) {
    println!("[MACOS_MONITOR] Starting with correct pasteboard name: {}", DRAG_PASTEBOARD_NAME);
    println!("[MACOS_MONITOR] Config: threshold={}, shakes={}, time_limit={}ms",
        config.shake_threshold, config.required_shakes, config.shake_time_limit);

    let files_dropped = Arc::new(AtomicBool::new(false));
    let files_dropped_clone = files_dropped.clone();

    let _unlistener = app_handle.listen("file_added", move |_| {
        println!("[MACOS_MONITOR] files_dropped event received");
        files_dropped_clone.store(true, Ordering::SeqCst);
    });

    thread::spawn(move || {
        let mut window_opened_by_shake = false;
        let mut window_position: Option<(f64, f64)> = None;
        let mut last_time_near_window = Instant::now();
        let window_proximity_threshold = 300.0; // pixels
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
        let mut debug_counter = 0u32;

        println!("[MACOS_MONITOR] Initial changeCount: {}", last_change_count);

        loop {
            let current_position = get_cursor_position();
            let current_change_count = get_pasteboard_change_count(&pasteboard);
            let has_files = pasteboard_has_files(&pasteboard);

            let change_count_changed = current_change_count != last_change_count && current_change_count > 0;

            debug_counter += 1;
            if debug_counter.is_multiple_of(20) {
                println!("[DEBUG] pos=({:.0},{:.0}) changeCount={} prev={} changed={} has_files={} dragging={}",
                    current_position.0, current_position.1,
                    current_change_count, last_change_count,
                    change_count_changed, has_files, is_drag_active);
            }

            if !is_drag_active {
                if change_count_changed && has_files {
                    is_drag_active = true;
                    drag_start_time = Some(Instant::now());
                    last_change_count = current_change_count;
                    println!("[DRAG_START] File drag detected! changeCount={}", current_change_count);
                }
            } else {
                let time_since_drag_start = drag_start_time.map(|t| t.elapsed()).unwrap_or(Duration::MAX);

                if (change_count_changed && time_since_drag_start > Duration::from_millis(100)) || !has_files {
                    let drag_ended_externally = !files_dropped.load(Ordering::SeqCst) && window_opened_by_shake;
                    
                    is_drag_active = false;
                    drag_start_time = None;
                    shake_count = 0;
                    last_direction = None;
                    last_change_count = current_change_count;
                    
                    if drag_ended_externally {
                        println!("[DRAG_END] Drag ended externally (files not dropped in app), hiding window");
                        let app = app_handle.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.hide();
                            println!("[WINDOW] Hidden due to external drop");
                        }
                        window_opened_by_shake = false;
                    } else {
                        println!("[DRAG_END] Drag ended. changeCount={}", current_change_count);
                    }
                }
            }

            if !is_drag_active {
                thread::sleep(check_interval);
                last_position = current_position;
                continue;
            }

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

            if shake_count > 0 && last_shake_time.elapsed() > movement_time_limit {
                println!("[SHAKE] Timeout, resetting");
                shake_count = 0;
                last_direction = None;
            }

            if shake_count >= config.required_shakes {
                println!("[SHAKE_DETECTED] Opening window at ({:.0}, {:.0})", current_position.0, current_position.1);

                let app_clone = app_handle.clone();
                let shake_count_clone = shake_count;
                tauri::async_runtime::spawn(async move {
                    let _ = analytics::send_mouse_shake_detected_event(&app_clone, shake_count_clone).await;
                });

                let mut window_x = current_position.0;
                let mut window_y = current_position.1;
                let margin = config.shake_threshold as f64;
                let (screen_width, screen_height) = get_screen_size();

                if current_position.0 + margin > screen_width {
                    window_x = current_position.0 - margin;
                }
                if current_position.1 + margin > screen_height {
                    window_y = current_position.1 - margin;
                }

                let app = app_handle.clone();
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.set_position(PhysicalPosition {
                        x: window_x as i32,
                        y: window_y as i32,
                    });
                    let _ = window.show();
                    let _ = window.unminimize();
                    let _ = window.set_focus();
                    window_opened_by_shake = true;
                    window_position = Some((window_x, window_y));
                    last_time_near_window = Instant::now();
                    println!("[WINDOW] Opened at x={}, y={}", window_x, window_y);

                    let app_clone2 = app.clone();
                    tauri::async_runtime::spawn(async move {
                        let _ = analytics::send_window_opened_event(&app_clone2, "main_shake").await;
                    });
                } else {
                    println!("[WINDOW] ERROR: Could not get main window!");
                }

                shake_count = 0;
                last_direction = None;
                is_drag_active = false;
                drag_start_time = None;
            }

            last_position = current_position;

            // Check if user moved away from the window after shake-opened it
            if window_opened_by_shake && !files_dropped.load(Ordering::SeqCst) {
                if let Some((win_x, win_y)) = window_position {
                    let dx = current_position.0 - win_x;
                    let dy = current_position.1 - win_y;
                    let distance = (dx * dx + dy * dy).sqrt();
                    
                    if distance < window_proximity_threshold {
                        // User is still near the window
                        last_time_near_window = Instant::now();
                    } else {
                        // User moved away from window
                        let time_away = last_time_near_window.elapsed();
                        if time_away >= Duration::from_secs(2) {
                            println!("[WINDOW] User moved away from window, hiding");
                            let app = app_handle.app_handle();
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.hide();
                                window_opened_by_shake = false;
                                window_position = None;
                                println!("[WINDOW] Hidden due to user moving away");
                            }
                        }
                    }
                }
            }

            if !files_dropped.load(Ordering::SeqCst) {
                let elapsed = last_shake_time.elapsed();
                if elapsed >= Duration::from_secs(config.window_close_delay) {
                    let app = app_handle.app_handle();
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.hide();
                        files_dropped.store(false, Ordering::SeqCst);
                        window_opened_by_shake = false;
                        println!("[WINDOW] Hidden");
                    }
                }
            } else {
                window_opened_by_shake = false;
                window_position = None;
            }

            thread::sleep(check_interval);
        }
    });
}
