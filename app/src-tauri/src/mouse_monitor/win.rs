use crate::config::MouseMonitorConfig;
use crate::DragState;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager, PhysicalPosition};
use windows::Win32::Foundation::POINT;
use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LBUTTON};
use windows::Win32::UI::WindowsAndMessaging::{
    GetCursorPos, GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN,
};

fn get_mouse_pos() -> POINT {
    let mut pos = POINT::default();
    unsafe {
        let _ = GetCursorPos(&mut pos);
    }
    pos
}

fn hide_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
}

fn show_main_window(app: &AppHandle, pos: POINT, config: &MouseMonitorConfig) {
    let screen_w = unsafe { GetSystemMetrics(SM_CXSCREEN) } as f64;
    let screen_h = unsafe { GetSystemMetrics(SM_CYSCREEN) } as f64;

    let margin = config.shake_threshold as f64;
    let mut x = pos.x as f64;
    let mut y = pos.y as f64;

    if x + margin > screen_w {
        x = pos.x as f64 - margin;
    }
    if y + margin > screen_h {
        y = pos.y as f64 - margin;
    }

    if let Some(window) = app.get_webview_window("main") {
        let _ = window.set_position(PhysicalPosition {
            x: x as i32,
            y: y as i32,
        });
        let _ = window.show();
        let _ = window.set_focus();
        println!("[WINDOW] Opened at x={}, y={}", x, y);
    }
}

pub fn start_mouse_monitor(
    config: MouseMonitorConfig,
    app_handle: AppHandle,
    drag_state: Arc<DragState>,
) {
    thread::spawn(move || {
        let mut window_opened_by_shake = false;
        let mut last_position = get_mouse_pos();
        let mut shake_count = 0;
        let mut last_shake_time = Instant::now();
        let mut last_direction: Option<i32> = None;

        let check_interval = Duration::from_millis(30);
        let movement_time_limit = Duration::from_millis(config.shake_time_limit);

        loop {
            let current_pos = get_mouse_pos();
            let mouse_down = unsafe { GetAsyncKeyState(VK_LBUTTON.0 as i32) as u16 & 0x8000 != 0 };

            // --- CASE 1: USER RELEASES MOUSE ---
            if !mouse_down {
                // If window was opened by shake and drag didn't result in a drop, hide it
                if window_opened_by_shake {
                    thread::sleep(Duration::from_millis(500));
                    let drag_started = drag_state.drag_started.load(Ordering::Relaxed);
                    let successful_drop = drag_state.successful_drop.load(Ordering::Relaxed);

                    if !drag_started && !successful_drop {
                        hide_main_window(&app_handle);
                        println!("[MOUSE_UP] Drag ended outside window, hiding");
                    }

                    // Reset drag state flags for next drag
                    drag_state.drag_started.store(false, Ordering::Relaxed);
                    drag_state.successful_drop.store(false, Ordering::Relaxed);
                }

                // Reset state
                window_opened_by_shake = false;
                shake_count = 0;
                last_direction = None;
                last_position = current_pos;
                thread::sleep(check_interval);
                continue;
            }

            // --- CASE 2: USER IS DRAGGING (Shake Detection) ---
            let distance_x = current_pos.x - last_position.x;
            let direction = if distance_x > config.shake_threshold {
                1
            } else if distance_x < -config.shake_threshold {
                -1
            } else {
                0
            };

            if direction != 0 {
                if let Some(last_dir) = last_direction {
                    if last_dir != direction {
                        shake_count += 1;
                        last_shake_time = Instant::now();
                        println!("[WIN_MONITOR] Shake count: {}", shake_count);
                    }
                }
                last_direction = Some(direction);
            }

            // Reset shake if too much time passes between wiggles
            if last_shake_time.elapsed() > movement_time_limit {
                shake_count = 0;
            }

            // Trigger Window
            if shake_count >= config.required_shakes && !window_opened_by_shake {
                show_main_window(&app_handle, current_pos, &config);
                window_opened_by_shake = true;

                // Spawn timeout thread to auto-hide if no drop
                let app_handle_clone = app_handle.clone();
                let drag_state_clone = Arc::clone(&drag_state);
                thread::spawn(move || {
                    let drag_started = drag_state_clone.drag_started.load(Ordering::Relaxed);
                    let successful_drop = drag_state_clone.successful_drop.load(Ordering::Relaxed);
                    if !drag_started && !successful_drop {
                        hide_main_window(&app_handle_clone);
                        println!("[TIMEOUT] Auto-hiding window after 500ms");
                    }
                });

                shake_count = 0;
                last_direction = None;
            }

            last_position = current_pos;
            thread::sleep(check_interval);
        }
    });
}
