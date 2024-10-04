use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use tauri::{Listener, Manager, PhysicalPosition};
use windows::Win32::Foundation::POINT;
use windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState;
use windows::Win32::UI::WindowsAndMessaging::{
    GetCursorPos, GetForegroundWindow, GetSystemMetrics, GetWindowLongW, GWL_STYLE, SM_CXSCREEN,
    SM_CYSCREEN, WS_MAXIMIZE,
};

pub fn start_mouse_monitor(required_shakes: u32, shake_time_limit: u64, app: tauri::AppHandle) {
    let files_dropped = Arc::new(AtomicBool::new(false));
    let files_dropped_clone = files_dropped.clone();

    // Set up a listener for the 'files_dropped' event
    let _unlistener = app.listen("files_dropped", move |_| {
        files_dropped_clone.store(true, Ordering::SeqCst);
    });

    thread::spawn(move || {
        let mut last_position = POINT { x: 0, y: 0 };
        let check_interval = Duration::from_millis(50);

        let shake_threshold_x = 50;
        let mut shake_count = 0;
        let movement_time_limit = Duration::from_millis(shake_time_limit);
        let mut last_shake_time = Instant::now();
        let mut last_direction: Option<i32> = None;
        let mut is_window_shown = false;

        loop {
            // Get screen dimensions
            let screen_width = unsafe { GetSystemMetrics(SM_CXSCREEN) };
            let screen_height = unsafe { GetSystemMetrics(SM_CYSCREEN) };

            // Check if a full-screen application is running
            let hwnd = unsafe { GetForegroundWindow() };
            let is_fullscreen = if hwnd.0 == 0 {
                false
            } else {
                let style = unsafe { GetWindowLongW(hwnd, GWL_STYLE) };
                (style & WS_MAXIMIZE.0 as i32) != 0
            };

            if is_fullscreen {
                shake_count = 0;
                last_direction = None;
                thread::sleep(check_interval);
                continue;
            }

            // Check if the left mouse button is pressed
            let is_button_pressed = unsafe { GetAsyncKeyState(0x01) } < 0;
            if is_button_pressed {
                let mut current_position = POINT::default();
                unsafe { GetCursorPos(&mut current_position) };

                // Calculate the distance moved on the x-axis
                let distance_x = current_position.x as f64 - last_position.x as f64;

                // Determine direction: 1 for right, -1 for left
                let direction = if distance_x > shake_threshold_x as f64 {
                    1 // Right
                } else if distance_x < -shake_threshold_x as f64 {
                    -1 // Left
                } else {
                    0 // No significant movement
                };

                // Check for direction change within the time limit
                if direction != 0 {
                    if let Some(last_dir) = last_direction {
                        if last_dir != direction && last_shake_time.elapsed() <= movement_time_limit
                        {
                            shake_count += 1; // Increment shake count
                        }
                    }
                    last_direction = Some(direction); // Update last direction
                    last_shake_time = Instant::now(); // Update last shake time
                }

                // Reset shake count if time limit exceeded
                if last_shake_time.elapsed() > movement_time_limit {
                    shake_count = 0; // Reset shake count if time limit exceeded
                    last_direction = None; // Reset last direction
                }

                if shake_count >= required_shakes {
                    // Check if shake count meets the required shakes
                    println!("Shake detected!");

                    // Ensure the window does not open off-screen or in the corner
                    let mut window_x = current_position.x;
                    let mut window_y = current_position.y;

                    // Define some margin to consider as "corner"
                    let margin = 300;

                    // Check if cursor is near the right or bottom edge
                    if current_position.x + margin > screen_width {
                        window_x = current_position.x - 300; // Move window left
                    }

                    if current_position.y + margin > screen_height {
                        window_y = current_position.y - 300; // Move window up
                    }

                    // Show the window at the adjusted position
                    let app = app.app_handle();
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.set_position(PhysicalPosition {
                            x: window_x,
                            y: window_y,
                        });
                        let _ = window.show();
                        let _ = window.set_focus();
                        is_window_shown = true; // Track that the window is shown
                    }
                    shake_count = 0; // Reset after detection
                }

                last_position = current_position; // No change needed here
            } else if is_window_shown {
                // Only hide the window when the left mouse button is released if no files were dropped
                if !files_dropped.load(Ordering::SeqCst) {
                    let app = app.app_handle();
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.hide(); // Hide the window
                        is_window_shown = false; // Reset window state
                    }
                } else {
                    // Reset the files_dropped flag after a short delay
                    let files_dropped_clone = files_dropped.clone();
                    let app_clone = app.clone();
                    thread::spawn(move || {
                        thread::sleep(Duration::from_secs(1));
                        files_dropped_clone.store(false, Ordering::SeqCst);
                        if let Some(window) = app_clone.get_webview_window("main") {
                            let _ = window.show();
                        }
                    });
                }
                shake_count = 0; // Reset shake count when the button is released
                last_direction = None; // Reset last direction
            }

            thread::sleep(check_interval);
        }
    });
}
