mod blocked_drag;

pub(crate) use blocked_drag::{holdem_is_elevated, DRAG_ENTERED};

use crate::config::{AppConfig, MouseMonitorConfig};
use crate::DragState;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager, State};
use tracing::info;
use windows::Win32::Foundation::POINT;
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LBUTTON, VK_RBUTTON};
use windows::Win32::UI::WindowsAndMessaging::{
    GetCursorPos, GetForegroundWindow, GetWindowThreadProcessId,
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
        if hwnd.0.is_null() {
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

/// The button that drags: the physical right button when the user swapped
/// buttons (left-handed setups), as GetAsyncKeyState reports physical keys.
pub(crate) fn is_mouse_button_down() -> bool {
    use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_SWAPBUTTON};
    let swapped = unsafe { GetSystemMetrics(SM_SWAPBUTTON) } != 0;
    let button = if swapped { VK_RBUTTON } else { VK_LBUTTON };
    unsafe { GetAsyncKeyState(button.0 as i32) as u16 & 0x8000 != 0 }
}

/// True when something is really being dragged, not just a button held
/// while selecting text, moving a window or dragging a scrollbar. Shell
/// drags show a drag image (a "SysDragImage" window); drags without one
/// show OLE's own drag cursors instead of the standard arrow, I-beam, hand
/// or resize cursors that ordinary held-button moves use.
fn drag_in_progress() -> bool {
    use windows::core::w;
    use windows::Win32::UI::WindowsAndMessaging::{
        FindWindowW, GetCursorInfo, IsWindowVisible, LoadCursorW, CURSORINFO, IDC_APPSTARTING,
        IDC_ARROW, IDC_CROSS, IDC_HAND, IDC_HELP, IDC_IBEAM, IDC_PERSON, IDC_PIN, IDC_SIZEALL,
        IDC_SIZENESW, IDC_SIZENS, IDC_SIZENWSE, IDC_SIZEWE, IDC_UPARROW, IDC_WAIT,
    };
    unsafe {
        if let Ok(image) = FindWindowW(w!("SysDragImage"), None) {
            if IsWindowVisible(image).as_bool() {
                return true;
            }
        }
        let mut info = CURSORINFO {
            cbSize: std::mem::size_of::<CURSORINFO>() as u32,
            ..Default::default()
        };
        if GetCursorInfo(&mut info).is_err() || info.hCursor.is_invalid() {
            return false;
        }
        // Cursors of ordinary held-button work: selecting, resizing, a
        // screenshot or paint tool's crosshair, waiting.
        let ordinary = [
            IDC_ARROW, IDC_IBEAM, IDC_HAND, IDC_SIZEALL, IDC_SIZEWE, IDC_SIZENS, IDC_SIZENWSE,
            IDC_SIZENESW, IDC_CROSS, IDC_WAIT, IDC_APPSTARTING, IDC_UPARROW, IDC_HELP, IDC_PIN,
            IDC_PERSON,
        ];
        !ordinary.iter().any(|id| {
            LoadCursorW(None, *id).is_ok_and(|standard| standard.0 == info.hCursor.0)
        })
    }
}

/// True when the foreground window covers its whole monitor (games, videos,
/// presentations). Shaking there is ordinary input, never a drag.
fn foreground_is_fullscreen() -> bool {
    use windows::Win32::Foundation::RECT;
    use windows::Win32::Graphics::Gdi::{
        GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST,
    };
    use windows::Win32::UI::WindowsAndMessaging::{GetClassNameW, GetWindowRect};
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0.is_null() {
            return false;
        }
        // The desktop itself is "full screen" but is where drags start.
        let mut class = [0u16; 32];
        let len = GetClassNameW(hwnd, &mut class) as usize;
        let class = String::from_utf16_lossy(&class[..len]);
        if class == "Progman" || class == "WorkerW" {
            return false;
        }
        let mut rect = RECT::default();
        if GetWindowRect(hwnd, &mut rect).is_err() {
            return false;
        }
        let monitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
        let mut info = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        if !GetMonitorInfoW(monitor, &mut info).as_bool() {
            return false;
        }
        let m = info.rcMonitor;
        rect.left <= m.left && rect.top <= m.top && rect.right >= m.right && rect.bottom >= m.bottom
    }
}

fn hide_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
}

pub(crate) fn has_shelf_files(app: &AppHandle) -> bool {
    if let Some(file_list_state) = app.try_state::<crate::FileList>() {
        if let Ok(files) = file_list_state.lock() {
            return !files.is_empty();
        }
    }
    false
}

fn hide_main_window_after_delay(app_handle: AppHandle, drag_state: Arc<DragState>, delay_ms: u64) {
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(delay_ms));
        let successful_drop = drag_state.successful_drop.load(Ordering::Relaxed);
        let mouse_down = is_mouse_button_down();
        if !successful_drop && !mouse_down && !has_shelf_files(&app_handle) {
            hide_main_window(&app_handle);
        }
    });
}

fn show_main_window(app: &AppHandle, pos: POINT, via: &'static str) {
    if let Some(window) = app.get_webview_window("main") {
        crate::utils::move_window_to_cursor(&window, pos.x, pos.y);
        let _ = window.show();
        info!("Opened main window near cursor ({}, {}) by {}", pos.x, pos.y, via);
        crate::analytics::track(app, "shelf_shown", vec![("via", via.into())]);
    }
}

/// True when a drag gesture may open the shelf here: any app unless it is
/// excluded, full screen, or (older configs) missing from a non-empty
/// allowlist, and only while something is really being dragged.
fn gesture_allowed(config: &MouseMonitorConfig) -> bool {
    // Dragging items out of the shelf must not move the shelf.
    if crate::commands::drag_ops::OUTGOING_DRAG.load(Ordering::SeqCst) {
        return false;
    }
    // Cheapest and most selective check first: holding a button to select
    // text must not look up the app 30 times a second.
    if !drag_in_progress() {
        return false;
    }
    let active_app = get_active_process_name().unwrap_or_default();
    (config.whitelist.is_empty() || config.whitelist.iter().any(|app| same_app(app, &active_app)))
        && !config.blocklist.iter().any(|app| same_app(app, &active_app))
        && !foreground_is_fullscreen()
}

/// True when a list entry names the running process. Case-insensitive, and
/// ".exe" is optional on either side, so "brave" matches "brave.exe" but
/// "code" no longer matches "vscode.exe".
fn same_app(entry: &str, process: &str) -> bool {
    fn base(name: &str) -> String {
        let lower = name.trim().to_ascii_lowercase();
        lower.strip_suffix(".exe").unwrap_or(&lower).to_string()
    }
    !entry.trim().is_empty() && base(entry) == base(process)
}

/// Counts left-right direction changes of the cursor while a button is held.
///
/// A change counts once the cursor has travelled at least `threshold` pixels
/// back from the furthest point it reached, however slowly it got there, so
/// a relaxed wiggle works as well as a fast one. Jitter under the threshold
/// is ignored, and the count resets when the next change takes longer than
/// `time_limit`.
#[derive(Debug, Default)]
struct ShakeDetector {
    /// x where tracking started; `None` until the first sample.
    anchor: Option<i32>,
    /// Current direction of travel (1 right, -1 left, 0 not yet moving).
    direction: i32,
    /// Furthest x reached in `direction`.
    extreme: i32,
    count: u32,
    last_change: Option<Instant>,
}

impl ShakeDetector {
    fn update(&mut self, x: i32, now: Instant, threshold: i32, time_limit: Duration) -> u32 {
        let threshold = threshold.max(1);
        if self.last_change.is_some_and(|t| now.duration_since(t) > time_limit) {
            self.count = 0;
            self.last_change = None;
        }
        let anchor = *self.anchor.get_or_insert(x);
        if self.direction == 0 {
            if (x - anchor).abs() >= threshold {
                self.direction = (x - anchor).signum();
                self.extreme = x;
            }
        } else if (x - self.extreme) * self.direction > 0 {
            self.extreme = x;
        } else if (self.extreme - x) * self.direction >= threshold {
            self.direction = -self.direction;
            self.extreme = x;
            self.count += 1;
            self.last_change = Some(now);
        }
        self.count
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
        let mut shake = ShakeDetector::default();
        let mut blocked = blocked_drag::BlockedDragWatch::default();

        loop {
            let check_interval = Duration::from_millis(30);
            blocked.tick(is_mouse_button_down(), drag_in_progress, &app_handle);
            // Idle (no button down, nothing to tidy up): skip everything
            // else, including reading the config, and check less often.
            if !window_opened_by_shake && !is_mouse_button_down() {
                shake = ShakeDetector::default();
                thread::sleep(check_interval * 3);
                continue;
            }
            // Never unwrap here: a poisoned config lock would kill this
            // detached loop with no restart, silently disabling shake-to-show.
            let config = {
                let state: State<Arc<Mutex<AppConfig>>> = app_handle.state();
                let Ok(lock) = state.lock() else {
                    thread::sleep(check_interval);
                    continue;
                };
                lock.mouse_monitor.clone()
            };
            let movement_time_limit = Duration::from_millis(config.shake_time_limit);

            let current_pos = get_mouse_pos();
            let mouse_down = is_mouse_button_down();

            // --- CASE 1: USER RELEASES MOUSE ---
            if !mouse_down {
                // If window was opened by shake and drag didn't result in a drop, hide it
                if window_opened_by_shake {
                    let successful_drop = drag_state.successful_drop.load(Ordering::Relaxed);

                    if !successful_drop && !has_shelf_files(&app_handle) {
                        hide_main_window_after_delay(
                            app_handle.clone(),
                            Arc::clone(&drag_state),
                            config.window_close_delay,
                        );
                    }

                    // Reset drag state flags for next drag
                    drag_state.drag_started.store(false, Ordering::Relaxed);
                }

                // Reset state. Nothing to detect until a button goes down, so
                // check less often (saves battery while idle).
                window_opened_by_shake = false;
                shake = ShakeDetector::default();
                thread::sleep(check_interval * 3);
                continue;
            }

            // --- CASE 2: USER IS DRAGGING (Shake Detection) ---
            let shake_count = shake.update(
                current_pos.x,
                Instant::now(),
                config.shake_threshold,
                movement_time_limit,
            );

            // Trigger Window
            if shake_count >= config.required_shakes && !window_opened_by_shake {
                if gesture_allowed(&config) {
                    drag_state.successful_drop.store(false, Ordering::Relaxed);
                    show_main_window(&app_handle, current_pos, "shake");
                    window_opened_by_shake = true;
                }

                shake = ShakeDetector::default();
            }

            thread::sleep(check_interval);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Feeds x positions sampled every `step_ms` and returns the final count.
    fn run(xs: &[i32], step_ms: u64, threshold: i32, limit_ms: u64) -> u32 {
        let start = Instant::now();
        let mut detector = ShakeDetector::default();
        let mut count = 0;
        for (i, &x) in xs.iter().enumerate() {
            let now = start + Duration::from_millis(step_ms * i as u64);
            count = detector.update(x, now, threshold, Duration::from_millis(limit_ms));
        }
        count
    }

    /// A back-and-forth of `amplitude` px, `steps` samples per sweep.
    fn wiggle(sweeps: usize, amplitude: i32, steps: i32) -> Vec<i32> {
        let mut xs = vec![0];
        for sweep in 0..sweeps {
            let (from, to) = if sweep % 2 == 0 { (0, amplitude) } else { (amplitude, 0) };
            for s in 1..=steps {
                xs.push(from + (to - from) * s / steps);
            }
        }
        xs
    }

    #[test]
    fn app_names_match_exactly() {
        assert!(same_app("brave", "brave.exe"));
        assert!(same_app("Explorer.EXE", "explorer.exe"));
        assert!(!same_app("code", "vscode.exe"));
        assert!(!same_app("", "explorer.exe"));
    }

    #[test]
    fn slow_wide_wiggle_counts() {
        // 150 px sweeps over 8 samples (240 ms): never 100 px in one sample.
        assert_eq!(run(&wiggle(6, 150, 8), 30, 100, 1500), 5);
    }

    #[test]
    fn jitter_below_threshold_is_ignored() {
        assert_eq!(run(&wiggle(10, 60, 3), 30, 100, 1500), 0);
    }

    #[test]
    fn one_long_drag_is_not_a_shake() {
        let xs: Vec<i32> = (0..100).map(|i| i * 20).collect();
        assert_eq!(run(&xs, 30, 100, 1500), 0);
    }

    #[test]
    fn pause_longer_than_limit_resets() {
        // Two changes (ending at x=150), a 2.4 s hold in place, then one more
        // change: the earlier two no longer count.
        let mut xs = wiggle(3, 150, 4);
        xs.extend(std::iter::repeat_n(150, 80));
        xs.push(0);
        assert_eq!(run(&xs, 30, 100, 1500), 1);
    }
}
