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

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[cfg(desktop)]
mod tray;

#[tauri::command]
fn start_drag(app: tauri::AppHandle, file_path: String) -> Result<(), String> {
    println!("Starting drag for file: {}", file_path);
    let item = match std::fs::canonicalize(file_path.clone()) {
        Ok(path) => {
            if !path.exists() {
                return Err(format!("File not found: {}", file_path));
            }
            drag::DragItem::Files(vec![path])
        }
        Err(e) => {
            return Err(format!("Error finding file: {} ({})", file_path, e));
        }
    };

    let window = app.get_webview_window("main").unwrap();
    // Define the on_drop_callback function
    let on_drop_callback = |result: drag::DragResult, _: drag::CursorPosition| {
        println!("Drag result: {:?}", result);
    };

    // Start the drag operation
    drag::start_drag(
        &(window.hwnd().unwrap().0 as isize),
        item,
        on_drop_callback,
        drag::Options::default(),
    )
    .expect("Failed to start drag operation");

    println!("Starting drag for file: {}", file_path);
    Ok(())
}

#[tauri::command]
fn start_multi_drag(app: tauri::AppHandle, file_paths: Vec<String>) -> Result<(), String> {
    println!(
        "Starting multi-file drag for files: {}",
        file_paths.join(", ")
    );

    let mut valid_paths = Vec::new();

    for file_path in file_paths {
        match std::fs::canonicalize(file_path.clone()) {
            Ok(path) => {
                if path.exists() {
                    valid_paths.push(path);
                } else {
                    println!("File not found: {}", file_path);
                }
            }
            Err(e) => {
                println!("Error finding file: {} ({})", file_path, e);
            }
        }
    }

    if valid_paths.is_empty() {
        return Err("No valid files to drag".to_string());
    }

    let item = drag::DragItem::Files(valid_paths);

    let window = app.get_webview_window("main").unwrap();
    let on_drop_callback = |result: drag::DragResult, _: drag::CursorPosition| {
        println!("Multi-file drag result: {:?}", result);
    };

    drag::start_drag(
        &(window.hwnd().unwrap().0 as isize),
        item,
        on_drop_callback,
        drag::Options::default(),
    )
    .map_err(|e| format!("Failed to start multi-file drag operation: {}", e))?;

    Ok(())
}

// Define a struct for mouse monitor configuration
#[derive(Clone)]
pub struct MouseMonitorConfig {
    required_shakes: u32,
    shake_time_limit: u64,
    shake_threshold: i32,
    window_close_delay: u64,
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
    let _unlistener = app.listen("files_dropped", move |_| {
        files_dropped_clone.store(true, Ordering::SeqCst);
    });

    thread::spawn(move || {
        let mut last_position = POINT { x: 0, y: 0 };
        let check_interval = Duration::from_millis(50);

        let mut shake_count = 0;
        let movement_time_limit = Duration::from_millis(config.shake_time_limit);
        let mut last_shake_time = Instant::now();
        let mut last_direction: Option<i32> = None;
        let mut window_open_time: Option<Instant> = None;
        let window_close_delay = Duration::from_millis(config.window_close_delay);

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

            let mut current_position = POINT::default();
            unsafe { GetCursorPos(&mut current_position) };

            // Calculate the distance moved on the x-axis
            let distance_x = current_position.x - last_position.x;

            // Determine direction: 1 for right, -1 for left, 0 for no significant movement
            let direction = if distance_x.abs() > config.shake_threshold {
                distance_x.signum()
            } else {
                0
            };

            // Check for direction change within the time limit
            if direction != 0 {
                if let Some(last_dir) = last_direction {
                    if last_dir != direction && last_shake_time.elapsed() <= movement_time_limit {
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

            if shake_count >= config.required_shakes {
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
                        x: window_x as f64,
                        y: window_y as f64,
                    });
                    let _ = window.show();
                    window_open_time = Some(Instant::now()); // Track when the window was opened
                }
                shake_count = 0; // Reset after detection
            }

            // Check if files were dropped
            if files_dropped.load(Ordering::SeqCst) {
                // Reset the files_dropped flag after a short delay
                let files_dropped_clone = files_dropped.clone();
                thread::spawn(move || {
                    thread::sleep(Duration::from_secs(1));
                    files_dropped_clone.store(false, Ordering::SeqCst);
                });
                window_open_time = None; // Reset the window open time to keep it open
            }

            // Check if the window should be closed
            if let Some(open_time) = window_open_time {
                if open_time.elapsed() > window_close_delay && !files_dropped.load(Ordering::SeqCst)
                {
                    let app = app.app_handle();
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.hide();
                        window_open_time = None; // Reset the window open time
                    }
                }
            }

            last_position = current_position; // Update last position
            thread::sleep(check_interval);
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![start_drag, start_multi_drag])
        .setup(|app| {
            #[cfg(all(desktop))]
            {
                let handle = app.handle();
                tray::create_tray(handle)?;
            }

            // Start the mouse monitor with custom configuration
            let app_handle = app.handle().clone();
            let config = MouseMonitorConfig {
                required_shakes: 5,
                shake_time_limit: 1500,
                shake_threshold: 100,
                window_close_delay: 3000,
            };
            start_mouse_monitor(config, app_handle.clone());

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
