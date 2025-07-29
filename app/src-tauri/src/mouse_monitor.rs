use crate::config::MouseMonitorConfig;
use active_win_pos_rs::get_active_window;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use tauri::AppHandle;
use tauri::{Listener, Manager, PhysicalPosition};
use windows::Win32::Foundation::{HWND, POINT};
use windows::Win32::UI::Input::KeyboardAndMouse::{DragDetect, GetAsyncKeyState, VK_LBUTTON};
use windows::Win32::UI::WindowsAndMessaging::{
    GetCursorPos, GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN,
};
use crate::analytics;

pub fn start_mouse_monitor(config: MouseMonitorConfig, app_handle: AppHandle) {
    let files_dropped = Arc::new(AtomicBool::new(false));
    let files_dropped_clone = files_dropped.clone();

    // Set up a listener for the 'files_dropped' event
    let _unlistener = app_handle.listen("file_added", move |_| {
        println!("files_dropped event received");
        files_dropped_clone.store(true, Ordering::SeqCst);
    });

    thread::spawn(move || {
        let mut last_position = POINT { x: 0, y: 0 };
        // Increase check interval to reduce CPU usage
        let check_interval = Duration::from_millis(100);
        let shake_threshold_x = 50;
        let mut shake_count = 0;
        let movement_time_limit = Duration::from_millis(config.shake_time_limit);
        let mut last_shake_time = Instant::now();
        let mut last_direction: Option<i32> = None;
        let mut last_active_window_check = Instant::now();
        let mut is_explorer_active = false;

        loop {
            // Only check active window every 500ms instead of every iteration
            if last_active_window_check.elapsed() >= Duration::from_millis(500) {
                is_explorer_active = match get_active_window() {
                    Ok(active_window) => active_window
                        .process_path
                        .to_str()
                        .unwrap()
                        .contains("explorer.exe"),
                    Err(_) => {
                        println!("error occurred while getting the active window");
                        false
                    }
                };
                last_active_window_check = Instant::now();
            }

            if !is_explorer_active {
                thread::sleep(check_interval);
                continue;
            }

            // Check if the left mouse button is pressed
            let is_mouse_down =
                unsafe { GetAsyncKeyState(VK_LBUTTON.0 as i32) & 0x8000u16 as i16 != 0 };

            if !is_mouse_down {
                // Reset shake count when mouse button is released
                shake_count = 0;
                thread::sleep(check_interval);
                continue;
            }

            let mut current_position = POINT::default();
            let _ = unsafe { GetCursorPos(&mut current_position) };

            // Safely get the window handle
            if let Some(window) = app_handle.get_webview_window("main") {
                if let Ok(hwnd) = window.hwnd() {
                    unsafe {
                        DragDetect(
                            HWND(hwnd.0 as isize),
                            current_position,
                        )
                    };
                }
            }

            // Calculate the distance moved on the x-axis
            let distance_x = current_position.x - last_position.x;

            // Determine direction: 1 for right, -1 for left
            let direction = if distance_x > shake_threshold_x {
                1 // Right
            } else if distance_x < -shake_threshold_x {
                -1 // Left
            } else {
                0 // No significant movement
            };

            // Check for direction change within the time limit
            if direction != 0 {
                if let Some(last_dir) = last_direction {
                    if last_dir != direction {
                        last_shake_time = Instant::now();
                        if last_shake_time.elapsed() <= movement_time_limit {
                            shake_count += 1;
                            println!("Shake count: {}", shake_count);
                            last_shake_time = Instant::now();
                        }
                    }
                }
                last_direction = Some(direction);
            }

            // Reset shake count if time limit exceeded
            if last_shake_time.elapsed() > movement_time_limit {
                shake_count = 0;
            }

            if shake_count >= config.required_shakes {
                println!("Shake detected! Opening window...");

                // Send analytics event for mouse shake detection
                let app_clone = app_handle.clone();
                let shake_count_clone = shake_count;
                tauri::async_runtime::spawn(async move {
                    if let Err(e) = analytics::send_mouse_shake_detected_event(&app_clone, shake_count_clone).await {
                        eprintln!("[Analytics] Failed to send mouse_shake_detected event: {}", e);
                    }
                });

                // Ensure the window does not open off-screen or in the corner
                let mut window_x = current_position.x;
                let mut window_y = current_position.y;

                // Define some margin to consider as "corner"
                let margin = config.shake_threshold;

                // Check if cursor is near the right or bottom edge
                if current_position.x + margin > unsafe { GetSystemMetrics(SM_CXSCREEN) } {
                    window_x = current_position.x - config.shake_threshold; // Move window left
                }

                if current_position.y + margin > unsafe { GetSystemMetrics(SM_CYSCREEN) } {
                    window_y = current_position.y - config.shake_threshold; // Move window up
                }

                // Show the window at the adjusted position
                let app = app_handle.clone();
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.set_position(PhysicalPosition {
                        x: window_x,
                        y: window_y,
                    });
                    let _ = window.show();
                    let _ = window.unminimize();
                    let _ = window.set_focus();
                    println!("Window opened successfully");
                    
                    // Send analytics event for window opened via mouse shake
                    let app_clone = app.clone();
                    tauri::async_runtime::spawn(async move {
                        if let Err(e) = analytics::send_window_opened_event(&app_clone, "main_shake").await {
                            eprintln!("[Analytics] Failed to send window_opened event: {}", e);
                        }
                    });
                }
                shake_count = 0;
            }

            last_position = current_position;

            // Check if we need to hide the window
            if !files_dropped.load(Ordering::SeqCst) {
                let elapsed = last_shake_time.elapsed();
                if elapsed >= Duration::from_secs(config.window_close_delay) {
                    let app = app_handle.app_handle();
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.hide();
                        files_dropped.store(false, Ordering::SeqCst);
                        println!("Window hidden");
                    }
                }
            }

            thread::sleep(check_interval);
        }
    });
}
