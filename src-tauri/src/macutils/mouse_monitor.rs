use cocoa::appkit::{NSApp, NSWindow};
use cocoa::foundation::NSPoint;
use core_graphics::event::{CGEvent, CGEventTapLocation, CGEventType};
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use tauri::{Listener, Manager, PhysicalPosition};

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
        let mut last_position = (0.0, 0.0);
        let check_interval = Duration::from_millis(50);

        let shake_threshold_x = 50;
        let mut shake_count = 0;
        let movement_time_limit = Duration::from_millis(config.shake_time_limit);
        let mut last_shake_time = Instant::now();
        let mut last_direction: Option<i32> = None;

        loop {
            // Check if the left mouse button is pressed
            let is_mouse_down = is_mouse_button_down();

            if is_mouse_down {
                let current_position = get_mouse_position();

                // Calculate the distance moved on the x-axis
                let distance_x = current_position.0 - last_position.0;

                // Determine direction: 1 for right, -1 for left
                let direction = if distance_x > shake_threshold_x as f64 {
                    1 // Right
                } else if distance_x < -(shake_threshold_x as f64) {
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
                    let mut window_x = current_position.0;
                    let mut window_y = current_position.1;

                    // Define some margin to consider as "corner"
                    let margin = config.shake_threshold as f64;

                    // Get screen size (macOS uses points, not pixels)
                    let screen_size = get_screen_size();

                    // Check if cursor is near the right or bottom edge
                    if current_position.0 + margin > screen_size.0 {
                        window_x = current_position.0 - config.shake_threshold as f64;
                        // Move window left
                    }

                    if current_position.1 + margin > screen_size.1 {
                        window_y = current_position.1 - config.shake_threshold as f64;
                        // Move window up
                    }

                    // Show the window at the adjusted position
                    let app = app.app_handle();
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.set_position(PhysicalPosition {
                            x: window_x as i32,
                            y: window_y as i32,
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

// Helper function to get the mouse position on macOS
fn get_mouse_position() -> (f64, f64) {
    let event = CGEvent::new(None).unwrap();
    let location = event.location();
    (location.x, location.y)
}

// Helper function to check if the left mouse button is pressed
fn is_mouse_button_down() -> bool {
    let event_source = CGEventSource::new(CGEventSourceStateID::HIDSystemState).unwrap();
    let state = event_source.button_state(CGEventType::LeftMouseDown);
    state
}

// Helper function to get the screen size on macOS
fn get_screen_size() -> (f64, f64) {
    let screen = NSScreen::main().unwrap();
    let frame = screen.frame();
    (frame.size.width, frame.size.height)
}
