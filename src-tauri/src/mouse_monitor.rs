use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use tauri::{Listener, Manager, PhysicalPosition};
use winapi::um::winuser::{GetForegroundWindow, GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};

pub fn start_mouse_monitor(required_shakes: u32, shake_time_limit: u64, app: tauri::AppHandle) {
    let files_dropped = Arc::new(AtomicBool::new(false));
    let files_dropped_clone = files_dropped.clone();

    // Set up a listener for the 'files_dropped' event
    let _unlistener = app.listen("files_dropped", move |_| {
        files_dropped_clone.store(true, Ordering::SeqCst);
    });

    thread::spawn(move || {
        let mut last_position = winapi::shared::windef::POINT { x: 0, y: 0 }; // Corrected POINT definition
        let check_interval = Duration::from_millis(50); // Check every 50 ms

        let shake_threshold_x = 50; // Set threshold for x-axis
        let mut shake_count = 0; // Counter for shakes
        let movement_time_limit = Duration::from_millis(shake_time_limit); // Time limit for direction change
        let mut last_shake_time = Instant::now(); // Track last shake time
        let mut last_direction: Option<i32> = None; // Track last direction (1 for right, -1 for left)
        let mut is_window_shown = false; // Track if window is shown

        loop {
            // Get screen dimensions
            let screen_width = unsafe { GetSystemMetrics(SM_CXSCREEN) };
            let screen_height = unsafe { GetSystemMetrics(SM_CYSCREEN) };

            // Check if a full-screen application is running
            let hwnd = unsafe { GetForegroundWindow() };
            let is_fullscreen = if hwnd.is_null() {
                false
            } else {
                let style = unsafe {
                    winapi::um::winuser::GetWindowLongW(hwnd, winapi::um::winuser::GWL_STYLE)
                };
                (style as i32 & winapi::um::winuser::WS_MAXIMIZE as i32) != 0 // Check if the window is maximized
            };

            if is_fullscreen {
                shake_count = 0; // Reset shake count if a full-screen app is running
                last_direction = None; // Reset last direction
                thread::sleep(check_interval); // Sleep and continue checking
                continue; // Skip the rest of the loop
            }

            // Check if the left mouse button is pressed
            let is_button_pressed = unsafe { winapi::um::winuser::GetAsyncKeyState(0x01) } < 0;
            if is_button_pressed {
                let mut current_position = winapi::shared::windef::POINT { x: 0, y: 0 }; // Changed LPPOINT to POINT
                unsafe {
                    winapi::um::winuser::GetCursorPos(&mut current_position);
                }

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
