use std::thread;
use std::time::{Duration, Instant};
use tauri::{Manager, WebviewWindowBuilder};
use winapi::um::winuser::GetForegroundWindow; // Import necessary traits

pub fn start_mouse_monitor(required_shakes: u32, shake_time_limit: u64, app: tauri::AppHandle) {
    thread::spawn(move || {
        let mut last_position = winapi::shared::windef::POINT { x: 0, y: 0 }; // Corrected POINT definition
        let check_interval = Duration::from_millis(50); // Check every 50 ms

        let shake_threshold_x = 50; // Set threshold for x-axis
        let mut shake_count = 0; // Counter for shakes
        let movement_time_limit = Duration::from_millis(shake_time_limit); // Time limit for direction change
        let mut last_shake_time = Instant::now(); // Track last shake time
        let mut last_direction: Option<i32> = None; // Track last direction (1 for right, -1 for left)

        loop {
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
            if unsafe { winapi::um::winuser::GetAsyncKeyState(0x01) } < 0 {
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

                    // Open a new window
                    let window_config = app.config().app.windows.get(0).unwrap(); // Get the first window config
                    match WebviewWindowBuilder::from_config(&app, window_config) {
                        Ok(window) => {
                            window.build().expect("Failed to build window");
                        }
                        Err(e) => {
                            eprintln!("Failed to create window: {}", e);
                        }
                    }

                    shake_count = 0; // Reset after detection
                }

                last_position = current_position; // No change needed here
            } else {
                shake_count = 0; // Reset if mouse button is not pressed
                last_direction = None; // Reset last direction
            }

            thread::sleep(check_interval);
        }
    });
}
