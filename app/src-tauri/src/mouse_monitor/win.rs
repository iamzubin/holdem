use crate::analytics;
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

pub fn start_mouse_monitor(config: MouseMonitorConfig, app_handle: AppHandle) {
    let files_dropped = Arc::new(AtomicBool::new(false));
    let files_dropped_clone = files_dropped.clone();

    let _unlistener = app_handle.listen("file_added", move |_| {
        println!("files_dropped event received");
        files_dropped_clone.store(true, Ordering::SeqCst);
    });

    thread::spawn(move || {
        let mut window_opened_by_shake = false;
        let mut window_position: Option<(i32, i32)> = None;
        let mut last_time_near_window = Instant::now();
        let window_proximity_threshold: f64 = 300.0;
        let mut last_position = POINT { x: 0, y: 0 };
        let check_interval = Duration::from_millis(100);
        let shake_threshold_x = 50;
        let mut shake_count = 0;
        let movement_time_limit = Duration::from_millis(config.shake_time_limit);
        let mut last_shake_time = Instant::now();
        let mut last_direction: Option<i32> = None;
        let mut last_active_window_check = Instant::now();
        let mut is_whitelisted_app_active = false;

        loop {
            if last_active_window_check.elapsed() >= Duration::from_millis(500) {
                is_whitelisted_app_active = match get_active_window() {
                    Ok(active_window) => {
                        let process_path = active_window
                            .process_path
                            .to_str()
                            .unwrap_or("")
                            .to_lowercase();

                        config.whitelist.iter().any(|app_name| {
                            process_path.contains(&app_name.to_lowercase())
                        })
                    }
                    Err(_) => {
                        println!("error occurred while getting the active window");
                        false
                    }
                };
                last_active_window_check = Instant::now();
            }

            if !is_whitelisted_app_active {
                thread::sleep(check_interval);
                continue;
            }

            let is_mouse_down =
                unsafe { GetAsyncKeyState(VK_LBUTTON.0 as i32) & 0x8000u16 as i16 != 0 };

            if !is_mouse_down {
                if window_opened_by_shake && !files_dropped.load(Ordering::SeqCst) {
                    println!("Mouse released without dropping files, hiding window");
                    let app = app_handle.app_handle();
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.hide();
                        println!("Window hidden due to external drop");
                    }
                    window_opened_by_shake = false;
                }
                shake_count = 0;
                thread::sleep(check_interval);
                continue;
            }

            let mut current_position = POINT::default();
            let _ = unsafe { GetCursorPos(&mut current_position) };

            if let Some(window) = app_handle.get_webview_window("main") {
                if let Ok(hwnd) = window.hwnd() {
                    unsafe {
                        let _ = DragDetect(
                            HWND(hwnd.0 as *mut std::ffi::c_void),
                            current_position,
                        );
                    };
                }
            }

            let distance_x = current_position.x - last_position.x;

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
                        if last_shake_time.elapsed() <= movement_time_limit {
                            shake_count += 1;
                            println!("Shake count: {}", shake_count);
                            last_shake_time = Instant::now();
                        }
                    }
                }
                last_direction = Some(direction);
            }

            if last_shake_time.elapsed() > movement_time_limit {
                shake_count = 0;
            }

            if shake_count >= config.required_shakes {
                println!("Shake detected! Opening window...");

                let app_clone = app_handle.clone();
                let shake_count_clone = shake_count;
                tauri::async_runtime::spawn(async move {
                    let _ = analytics::send_mouse_shake_detected_event(&app_clone, shake_count_clone).await;
                });

                let mut window_x = current_position.x;
                let mut window_y = current_position.y;
                let margin = config.shake_threshold;

                if current_position.x + margin > unsafe { GetSystemMetrics(SM_CXSCREEN) } {
                    window_x = current_position.x - config.shake_threshold;
                }

                if current_position.y + margin > unsafe { GetSystemMetrics(SM_CYSCREEN) } {
                    window_y = current_position.y - config.shake_threshold;
                }

                let app = app_handle.clone();
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.set_position(PhysicalPosition {
                        x: window_x,
                        y: window_y,
                    });
                    let _ = window.show();
                    let _ = window.unminimize();
                    let _ = window.set_focus();
                    window_opened_by_shake = true;
                    window_position = Some((window_x, window_y));
                    last_time_near_window = Instant::now();
                    println!("Window opened successfully");

                    let app_clone = app.clone();
                    tauri::async_runtime::spawn(async move {
                        let _ = analytics::send_window_opened_event(&app_clone, "main_shake").await;
                    });
                }
                shake_count = 0;
            }

            last_position = current_position;

            // Check if user moved away from the window after shake-opened it
            if window_opened_by_shake && !files_dropped.load(Ordering::SeqCst) {
                if let Some((win_x, win_y)) = window_position {
                    let dx = (current_position.x - win_x) as f64;
                    let dy = (current_position.y - win_y) as f64;
                    let distance = (dx * dx + dy * dy).sqrt();
                    
                    if distance < window_proximity_threshold {
                        // User is still near the window
                        last_time_near_window = Instant::now();
                    } else {
                        // User moved away from window
                        let time_away = last_time_near_window.elapsed();
                        if time_away >= Duration::from_secs(2) {
                            println!("User moved away from window, hiding");
                            let app = app_handle.app_handle();
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.hide();
                                window_opened_by_shake = false;
                                window_position = None;
                                println!("Window hidden due to user moving away");
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
                        println!("Window hidden");
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
