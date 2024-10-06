use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Listener, Manager, PhysicalPosition};
use windows::Win32::Foundation::{HWND, POINT};
use windows::Win32::UI::Input::KeyboardAndMouse::{DragDetect, GetAsyncKeyState, VK_LBUTTON};
use windows::Win32::UI::WindowsAndMessaging::{
    GetCursorPos, GetForegroundWindow, GetSystemMetrics, GetWindowLongW, GWL_STYLE, SM_CXSCREEN,
    SM_CYSCREEN, WS_MAXIMIZE,
};

#[derive(Clone)]
pub struct MouseMonitorConfig {
    pub(crate) required_shakes: u32,
    pub(crate) shake_time_limit: u64,
    pub(crate) shake_threshold: i32,
    pub(crate) window_close_delay: u64,
}

impl Default for MouseMonitorConfig {
    fn default() -> Self {
        MouseMonitorConfig {
            required_shakes: 5,
            shake_time_limit: 1500,
            shake_threshold: 100,
            window_close_delay: 3000,
        }
    }
}
pub fn start_mouse_monitor(config: MouseMonitorConfig, app: tauri::AppHandle) {
    let files_dropped = Arc::new(AtomicBool::new(false));
    let files_dropped_clone = files_dropped.clone();

    // Set up a listener for the 'files_dropped' event
    let _unlistener = app.listen("file_added", move |_| {
        println!("files_dropped event received");
        files_dropped_clone.store(true, Ordering::SeqCst);
    });

    thread::spawn(move || {
        let mut last_position = POINT { x: 0, y: 0 };
        let check_interval = Duration::from_millis(50);

        let shake_threshold_x = 50;
        let mut shake_count = 0;
        let movement_time_limit = Duration::from_millis(config.shake_time_limit);
        let mut last_shake_time = Instant::now();
        let mut last_direction: Option<i32> = None;

        loop {
            // Check if the left mouse button is pressed
            let is_mouse_down =
                unsafe { GetAsyncKeyState(VK_LBUTTON.0 as i32) & 0x8000u16 as i16 != 0 };

            if is_mouse_down {
                let mut current_position = POINT::default();
                unsafe { GetCursorPos(&mut current_position) };

                let drag_detect_result = unsafe {
                    DragDetect(
                        HWND(
                            app.app_handle()
                                .get_webview_window("main")
                                .unwrap()
                                .hwnd()
                                .unwrap()
                                .0 as isize,
                        ),
                        current_position,
                    )
                };

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
                        println!(
                            "last_dir: {:?}, direction: {:?}, shake_count: {:?}",
                            last_dir, direction, shake_count
                        );
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
                    let app = app.app_handle();
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.set_position(PhysicalPosition {
                            x: window_x,
                            y: window_y,
                        });
                        let _ = window.show();
                        let _ = window.unminimize();
                        let _ = window.set_focus();
                        println!("Window opened successfully");
                    }
                    shake_count = 0;
                }

                last_position = current_position;
            } else {
                // Reset shake count when mouse button is released
                shake_count = 0;
            }

            // Check if we need to hide the window
            if !files_dropped.load(Ordering::SeqCst) {
                let elapsed = last_shake_time.elapsed();
                if elapsed >= Duration::from_secs(config.window_close_delay) {
                    let app = app.app_handle();
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
