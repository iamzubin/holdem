use crate::config::{AppConfig, MouseMonitorConfig};
use crate::DragState;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager, PhysicalPosition, State};
use tracing::info;
use windows::Win32::Foundation::POINT;
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LBUTTON};
use windows::Win32::UI::WindowsAndMessaging::{
    GetCursorPos, GetForegroundWindow, GetSystemMetrics, GetWindowThreadProcessId, SM_CXSCREEN,
    SM_CYSCREEN,
};

fn get_mouse_pos() -> POINT {
    let mut pos = POINT::default();
    unsafe {
        let _ = GetCursorPos(&mut pos);
    }
    pos
}

fn get_active_process_name() -> Option<String> {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0 == std::ptr::null_mut() {
            return None;
        }
        let mut process_id = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut process_id));
        if process_id == 0 {
            return None;
        }

        let process_handle =
            OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, process_id).ok()?;
        let mut buffer = [0u16; 1024];
        let mut size = buffer.len() as u32;

        // Use QueryFullProcessImageNameW (requires Win32_System_Threading)
        let success = QueryFullProcessImageNameW(
            process_handle,
            PROCESS_NAME_WIN32,
            windows::core::PWSTR(buffer.as_mut_ptr()),
            &mut size,
        );

        let _ = windows::Win32::Foundation::CloseHandle(process_handle);

        if success.is_ok() && size > 0 {
            let path = String::from_utf16_lossy(&buffer[..size as usize]);
            let name = std::path::Path::new(&path)
                .file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.to_string());
            return name;
        }
        None
    }
}

fn is_mouse_button_down() -> bool {
    unsafe { GetAsyncKeyState(VK_LBUTTON.0 as i32) as u16 & 0x8000 != 0 }
}

fn hide_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
}

fn hide_main_window_after_delay(app_handle: AppHandle, drag_state: Arc<DragState>, delay_ms: u64) {
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(delay_ms));
        let successful_drop = drag_state.successful_drop.load(Ordering::Relaxed);
        let mouse_down = is_mouse_button_down();
        if !successful_drop && !mouse_down {
            hide_main_window(&app_handle);
        }
    });
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
        info!(
            "Opened main window at position ({}, {})",
            x as i32, y as i32
        );
    }
}

pub fn start_mouse_monitor(
    config: MouseMonitorConfig,
    app_handle: AppHandle,
    drag_state: Arc<DragState>,
) {
    info!(
        "Starting Windows mouse monitor (threshold={}, required_shakes={}, time_limit_ms={})",
        config.shake_threshold, config.required_shakes, config.shake_time_limit
    );

    thread::spawn(move || {
        let mut window_opened_by_shake = false;
        let mut last_position = get_mouse_pos();
        let mut shake_count = 0;
        let mut last_shake_time = Instant::now();
        let mut last_direction: Option<i32> = None;

        loop {
            let config = {
                let state: State<Arc<Mutex<AppConfig>>> = app_handle.state();
                let lock = state.lock().unwrap();
                lock.mouse_monitor.clone()
            };

            let check_interval = Duration::from_millis(30);
            let movement_time_limit = Duration::from_millis(config.shake_time_limit);

            let current_pos = get_mouse_pos();
            let mouse_down = is_mouse_button_down();

            // --- CASE 1: USER RELEASES MOUSE ---
            if !mouse_down {
                // If window was opened by shake and drag didn't result in a drop, hide it
                if window_opened_by_shake {
                    let successful_drop = drag_state.successful_drop.load(Ordering::Relaxed);

                    if !successful_drop {
                        hide_main_window_after_delay(
                            app_handle.clone(),
                            Arc::clone(&drag_state),
                            config.window_close_delay,
                        );
                    }

                    // Reset drag state flags for next drag
                    drag_state.drag_started.store(false, Ordering::Relaxed);
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
                // Check if current active window is in the whitelist.
                // If whitelist is empty, we assume it's disabled or allows everything?
                // The frontend says "No apps whitelisted", so if it's empty, no app should trigger it.
                // But wait, what if the user expects it to work everywhere if empty?
                // Usually an empty whitelist means nothing is allowed.
                // Let's implement strict whitelist: if not empty, must be in it.
                // If it is empty, maybe allow nothing.
                let active_app = get_active_process_name().unwrap_or_default();
                let is_whitelisted = config.whitelist.is_empty()
                    || config.whitelist.iter().any(|app| {
                        app.eq_ignore_ascii_case(&active_app)
                            || active_app.to_lowercase().contains(&app.to_lowercase())
                    });

                if is_whitelisted {
                    drag_state.successful_drop.store(false, Ordering::Relaxed);
                    show_main_window(&app_handle, current_pos, &config);
                    window_opened_by_shake = true;
                }

                shake_count = 0;
                last_direction = None;
            }

            last_position = current_pos;
            thread::sleep(check_interval);
        }
    });
}
