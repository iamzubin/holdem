use crate::config::MouseMonitorConfig;
use device_query::{DeviceQuery, DeviceState, MouseState};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use tauri::AppHandle;
use tauri::{Listener, Manager, PhysicalPosition};
use crate::analytics;

#[cfg(target_os = "windows")]
use windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState;
#[cfg(target_os = "windows")]
use windows::Win32::UI::Input::KeyboardAndMouse::VK_LBUTTON;

pub fn start_mouse_monitor(config: MouseMonitorConfig, app_handle: AppHandle) {
    let files_dropped = Arc::new(AtomicBool::new(false));
    let files_dropped_clone = files_dropped.clone();

    let _unlistener = app_handle.listen("file_added", move |_| {
        println!("files_dropped event received");
        files_dropped_clone.store(true, Ordering::SeqCst);
    });

    let config_clone = config.clone();
    thread::spawn(move || {
        // Initialize device state with panic handling
        let device_state = std::panic::catch_unwind(|| {
            DeviceState::new()
        });
        
        let device_state = match device_state {
            Ok(state) => state,
            Err(_) => {
                eprintln!("[MouseMonitor] Failed to initialize device query - likely missing permissions on macOS");
                println!("[MouseMonitor] Mouse shake detection disabled - requires Input Monitoring permission on macOS");
                return;
            }
        };
        
        println!("[MouseMonitor] Started successfully");

        let mut last_position = (0, 0);
        let check_interval = Duration::from_millis(100);
        let shake_threshold_x = 50;
        let mut shake_count = 0;
        let movement_time_limit = Duration::from_millis(config_clone.shake_time_limit);
        let mut last_shake_time = Instant::now();
        let mut last_direction: Option<i32> = None;
        let mut last_screen_check = Instant::now();
        let mut screen_size = (1920, 1080);

        loop {
            // Check for panic in thread safety
            if shake_count > 100 {
                shake_count = 0;
            }

            if last_screen_check.elapsed() >= Duration::from_secs(1) {
                if let Ok(monitors) = app_handle.available_monitors() {
                    if let Some(monitor) = monitors.first() {
                        let size = monitor.size();
                        screen_size = (size.width as i32, size.height as i32);
                    }
                }
                last_screen_check = Instant::now();
            }

            let mouse: MouseState = device_state.get_mouse();
            let current_position = mouse.coords;

            if current_position.0 == 0 && current_position.1 == 0 {
                thread::sleep(check_interval);
                continue;
            }

            let is_mouse_down = {
                let buttons = &mouse.button_pressed;
                buttons.get(0).copied().unwrap_or(false)
            };

            if !is_mouse_down {
                shake_count = 0;
                thread::sleep(check_interval);
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

            if shake_count >= config_clone.required_shakes {
                println!("Shake detected! Opening window...");

                let app_clone = app_handle.clone();
                let shake_count_clone = shake_count;
                tauri::async_runtime::spawn(async move {
                    if let Err(e) = analytics::send_mouse_shake_detected_event(&app_clone, shake_count_clone).await {
                        eprintln!("[Analytics] Failed to send mouse_shake_detected event: {}", e);
                    }
                });

                let mut window_x = current_position.0;
                let mut window_y = current_position.1;

                let margin = config_clone.shake_threshold;

                if current_position.0 + margin > screen_size.0 {
                    window_x = current_position.0 - config_clone.shake_threshold;
                }

                if current_position.1 + margin > screen_size.1 {
                    window_y = current_position.1 - config_clone.shake_threshold;
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
                    println!("Window opened successfully");
                    
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

            if !files_dropped.load(Ordering::SeqCst) {
                let elapsed = last_shake_time.elapsed();
                if elapsed >= Duration::from_secs(config_clone.window_close_delay) {
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
