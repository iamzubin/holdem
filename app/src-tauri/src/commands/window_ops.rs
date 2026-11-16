use crate::analytics;
use crate::utils::{place_below_or_above, work_area_at, Rect};
use crate::DragState;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tauri::{
    AppHandle, Emitter, LogicalSize, Manager, Runtime, State, WebviewUrl, WebviewWindow, WebviewWindowBuilder,
    WindowEvent,
};
use tracing::{error, warn};
use windows::Win32::Foundation::{HWND, POINT, RECT};
use windows::Win32::UI::WindowsAndMessaging::{
    GetCursorPos, GetForegroundWindow, GetWindowRect, GetWindowThreadProcessId, IsWindowVisible,
    SetForegroundWindow, SetWindowPos, SWP_NOACTIVATE, SWP_NOZORDER,
};

fn sanitize_theme(theme: Option<String>) -> Option<String> {
    match theme.as_deref() {
        Some("dark") | Some("light") | Some("system") => theme,
        _ => None,
    }
}

fn url_with_theme(base: &str, theme: Option<String>) -> String {
    match sanitize_theme(theme) {
        Some(t) => format!("{}?theme={}", base, t),
        None => base.to_string(),
    }
}

/// Popup size in logical pixels.
const POPUP_SIZE: (f64, f64) = (450.0, 350.0);
/// Gap between the shelf and the popup, in logical pixels.
const POPUP_GAP: f64 = 5.0;

/// Bumped every time the popup is shown, so only the newest dismiss
/// watcher keeps running.
static POPUP_SHOWN: AtomicU64 = AtomicU64::new(0);
/// Set while something is dragged over the popup (see drop_target.rs).
pub(crate) static POPUP_DROP_HOVER: AtomicBool = AtomicBool::new(false);
/// Set while one of the popup's own dialogs (folder picker) is open.
pub(crate) static POPUP_KEEP_OPEN: AtomicBool = AtomicBool::new(false);
/// Set while the popup window is being created, so a second click doesn't
/// start another.
static POPUP_BUILDING: AtomicBool = AtomicBool::new(false);

/// Toggles the file popup. The popup is built once, on first use, and then
/// only shown and hidden: it opens instantly, keeps its drop target, and
/// keeps the chosen view. `focus` (opened from the keyboard) also puts focus
/// on the first item.
#[tauri::command]
pub fn open_popup_window(
    app: AppHandle,
    theme: Option<String>,
    focus: Option<bool>,
) -> Result<(), String> {
    let keyboard = focus.unwrap_or(false);
    if let Some(popup) = app.get_webview_window("popup") {
        if popup.is_visible().unwrap_or(false) {
            hide_popup(&app);
        } else {
            show_popup(&app, &popup, keyboard)?;
        }
        return Ok(());
    }
    if POPUP_BUILDING.swap(true, Ordering::SeqCst) {
        return Ok(());
    }
    // Built off the command thread: building a webview from a synchronous
    // command deadlocks on Windows. The first page load can't hear
    // `popup-shown` yet, so it reads "opened from the keyboard" from the URL.
    let mut popup_url = url_with_theme("popup", theme);
    if keyboard {
        popup_url.push_str(if popup_url.contains('?') { "&keyboard" } else { "?keyboard" });
    }
    tauri::async_runtime::spawn(async move {
        let built = WebviewWindowBuilder::new(&app, "popup", WebviewUrl::App(popup_url.into()))
            .title("File List")
            .decorations(false)
            .transparent(true) // lets Mica show through (window_style.rs)
            .shadow(false)
            .resizable(false)
            .inner_size(POPUP_SIZE.0, POPUP_SIZE.1)
            .visible(false)
            .always_on_top(true)
            .skip_taskbar(true)
            .accept_first_mouse(true)
            .visible_on_all_workspaces(true)
            // Holdem's own drop target takes drops here, as on the shelf.
            .disable_drag_drop_handler()
            .build();
        POPUP_BUILDING.store(false, Ordering::SeqCst);
        let popup = match built {
            Ok(popup) => popup,
            Err(e) => {
                error!("Failed to create the popup window: {}", e);
                return;
            }
        };
        accept_drops_on_popup(&app, &popup);
        if let Err(e) = show_popup(&app, &popup, keyboard) {
            error!("Failed to show the popup window: {}", e);
        }
    });
    Ok(())
}

/// Places the popup under (or above) the shelf, at the scale of the monitor
/// it lands on, then shows and focuses it.
fn show_popup(app: &AppHandle, popup: &WebviewWindow, keyboard: bool) -> Result<(), String> {
    let main = app.get_webview_window("main").ok_or("Main window not found")?;
    let position = main.outer_position().map_err(|e| e.to_string())?;
    let size = main.outer_size().map_err(|e| e.to_string())?;
    let shelf = Rect {
        x: position.x,
        y: position.y,
        width: size.width as i32,
        height: size.height as i32,
    };
    let centre_x = shelf.x + shelf.width / 2;
    // Not the shelf's scale, nor the popup's previous one: the scale of the
    // monitor the popup is about to land on.
    let scale = main
        .monitor_from_point(centre_x as f64, shelf.y as f64)
        .ok()
        .flatten()
        .or_else(|| main.primary_monitor().ok().flatten())
        .map_or(1.0, |m| m.scale_factor());
    let popup_size = (
        (POPUP_SIZE.0 * scale).round() as i32,
        (POPUP_SIZE.1 * scale).round() as i32,
    );
    let gap = (POPUP_GAP * scale).round() as i32;
    let (x, y) = match work_area_at(&main, centre_x, shelf.y) {
        Some(area) => place_below_or_above(shelf, popup_size, area, gap),
        None => (
            shelf.x + (shelf.width - popup_size.0) / 2,
            shelf.y + shelf.height + gap,
        ),
    };
    let hwnd = HWND(popup.hwnd().map_err(|e| e.to_string())?.0 as _);
    for _ in 0..2 {
        // Moved while still hidden, twice: when the move crosses into a
        // monitor with another scale, Windows resizes the window for it
        // (WM_DPICHANGED), and the second call sets the exact rectangle
        // again. Nothing is on screen yet, so neither step can be seen.
        unsafe {
            SetWindowPos(
                hwnd,
                None,
                x,
                y,
                popup_size.0,
                popup_size.1,
                SWP_NOZORDER | SWP_NOACTIVATE,
            )
        }
        .map_err(|e| e.to_string())?;
    }
    POPUP_DROP_HOVER.store(false, Ordering::SeqCst);
    let generation = POPUP_SHOWN.fetch_add(1, Ordering::SeqCst) + 1;
    let _ = popup.emit_to("popup", "popup-shown", keyboard);
    popup.show().map_err(|e| e.to_string())?;
    // Windows may refuse; then the popup simply opens unfocused and hides on
    // the next outside click. (Tauri's set_focus would fake an Alt key press
    // to force it, which can reach the app in front.)
    let _ = unsafe { SetForegroundWindow(hwnd) };
    // WebView2 can create or recreate its child windows after the first
    // registration; registering again is harmless (it revokes first). On the
    // main thread: OLE is initialised there only, and this can run on a
    // worker the first time.
    let handle = app.clone();
    let raw = hwnd.0 as isize;
    let _ = app.run_on_main_thread(move || {
        crate::drop_target::register_drop_target(&handle, "popup", HWND(raw as _));
    });
    let shelf_hwnd = main.hwnd().map_err(|e| e.to_string())?.0 as isize;
    watch_for_dismiss(app.clone(), hwnd.0 as isize, shelf_hwnd, generation);
    analytics::track(app, "popup_window_opened", vec![]);
    Ok(())
}

/// Hides the popup, if it is shown. The page then clears its selection, so
/// the list opens fresh next time.
pub(crate) fn hide_popup(app: &AppHandle) {
    let Some(popup) = app.get_webview_window("popup") else {
        return;
    };
    if !popup.is_visible().unwrap_or(false) {
        return;
    }
    if let Err(e) = popup.hide() {
        error!("Failed to hide the popup window: {}", e);
    }
    let _ = popup.emit_to("popup", "popup-hidden", ());
}

/// Esc in the popup.
#[tauri::command]
pub fn close_popup(app: AppHandle) {
    hide_popup(&app);
}

/// The popup gets out of the way once it is no longer in use, without a
/// timer:
/// - another app comes to the front with no button held (Alt+Tab, or a
///   click that already ended): it hides at once;
/// - a click or drag starts outside it while another app is in front: it
///   hides when the button is released, unless the pointer was over the
///   popup then (a drag dropped on it);
/// - after such a drop it stays until the next click outside it.
///
/// Holdem's own windows (the shelf, the folder picker) never count as
/// another app. Until the popup has been in front once (its focus request
/// can be refused), only clicks outside it count. It also goes when the
/// shelf it belongs to is hidden or moved. Polls, like the mouse monitor,
/// and only while it is shown.
fn watch_for_dismiss(app: AppHandle, hwnd: isize, shelf: isize, generation: u64) {
    std::thread::spawn(move || {
        let hwnd = HWND(hwnd as _);
        let shelf = HWND(shelf as _);
        let shelf_rect = window_rect(shelf);
        let current = move || POPUP_SHOWN.load(Ordering::SeqCst) == generation;
        // `show()` from a background thread is only queued: wait for the
        // window to appear before "not visible" means "closed".
        let mut seen_visible = false;
        let mut was_ours = false; // Holdem has been in front since showing
        let mut away = false; // another app is in front
        let mut pending = false; // a press that began outside is still held
        let mut was_down = false;
        let mut hovered = false; // a drag was over the popup at the last check
        loop {
            std::thread::sleep(Duration::from_millis(30));
            if !current() {
                return;
            }
            if !unsafe { IsWindowVisible(hwnd) }.as_bool() {
                if seen_visible {
                    return;
                }
                continue;
            }
            seen_visible = true;
            // Its folder picker is open: nothing closes it underneath.
            if POPUP_KEEP_OPEN.load(Ordering::SeqCst) {
                continue;
            }
            if !unsafe { IsWindowVisible(shelf) }.as_bool() || window_rect(shelf) != shelf_rect {
                break;
            }
            let down = crate::mouse_monitor::is_mouse_button_down();
            if foreground_is_ours() {
                was_ours = true;
                away = false;
                pending = false;
                was_down = down;
                continue;
            }
            if !away {
                away = true;
                if down {
                    pending = true;
                } else if was_ours {
                    break;
                }
            }
            if down && !was_down && !cursor_over(hwnd) {
                pending = true;
            }
            if pending && !down {
                // Read before the release: dropping clears the hover flag.
                if !hovered {
                    break;
                }
                pending = false;
            }
            hovered = down && POPUP_DROP_HOVER.load(Ordering::SeqCst);
            was_down = down;
        }
        let handle = app.clone();
        let _ = app.run_on_main_thread(move || {
            if current() {
                hide_popup(&handle);
            }
        });
    });
}

/// True when the window in front belongs to Holdem.
fn foreground_is_ours() -> bool {
    unsafe {
        let window = GetForegroundWindow();
        if window.is_invalid() {
            return false;
        }
        let mut pid = 0u32;
        GetWindowThreadProcessId(window, Some(&mut pid));
        pid == std::process::id()
    }
}

/// A window's rectangle on screen (empty if it can't be read).
fn window_rect(hwnd: HWND) -> (i32, i32, i32, i32) {
    let mut rect = RECT::default();
    let _ = unsafe { GetWindowRect(hwnd, &mut rect) };
    (rect.left, rect.top, rect.right, rect.bottom)
}

/// True when the pointer is over `hwnd`.
fn cursor_over(hwnd: HWND) -> bool {
    let mut point = POINT::default();
    let mut rect = RECT::default();
    let known = unsafe { GetCursorPos(&mut point).is_ok() && GetWindowRect(hwnd, &mut rect).is_ok() };
    known
        && (rect.left..rect.right).contains(&point.x)
        && (rect.top..rect.bottom).contains(&point.y)
}

/// Registers the shelf's drop target on the popup. WebView2 creates its
/// child windows a moment after the window itself, and an unregistered child
/// would swallow drops, so registration is repeated once they exist. The
/// popup is never destroyed, so this runs once.
fn accept_drops_on_popup(app: &AppHandle, popup: &WebviewWindow) {
    let Ok(hwnd) = popup.hwnd() else {
        warn!("Popup has no window handle; it won't accept drops");
        return;
    };
    let hwnd = hwnd.0 as isize;
    let app = app.clone();
    std::thread::spawn(move || {
        for wait_ms in [0u64, 300, 1200] {
            std::thread::sleep(Duration::from_millis(wait_ms));
            let handle = app.clone();
            let registered = app.run_on_main_thread(move || {
                let hwnd = HWND(hwnd as _);
                crate::drop_target::register_drop_target(&handle, "popup", hwnd);
            });
            if registered.is_err() {
                return;
            }
        }
    });
}

/// Settings fields other windows may point at (the id of the control).
const SETTINGS_FIELDS: &[&str] = &["mica"];

/// Sent to an open Settings window with the id of the field to show.
const SETTINGS_FOCUS_EVENT: &str = "settings-focus";

/// Opens Settings, or brings it back. `field`, when it names one of
/// [`SETTINGS_FIELDS`], is scrolled into view and focused.
#[tauri::command]
pub fn open_settings_window(
    app: AppHandle,
    theme: Option<String>,
    field: Option<String>,
) -> Result<(), String> {
    // Define settings window dimensions
    let settings_width = 500.0;
    let settings_height = 600.0;
    let field = field.filter(|f| SETTINGS_FIELDS.contains(&f.as_str()));

    if let Some(settings_window) = app.get_webview_window("settings") {
        // Bring it back rather than closing it: the shelf is always on top,
        // so Settings is often just hidden behind it, and closing would
        // throw away unsaved edits.
        settings_window.unminimize().map_err(|e| e.to_string())?;
        settings_window.set_focus().map_err(|e| e.to_string())?;
        if let Some(field) = field {
            let _ = settings_window.emit_to("settings", SETTINGS_FOCUS_EVENT, field);
        }
    } else {
        // Create the settings window
        let app_clone = app.clone();
        let mut settings_url = url_with_theme("settings", theme);
        if let Some(field) = field {
            let joiner = if settings_url.contains('?') { '&' } else { '?' };
            settings_url.push_str(&format!("{joiner}field={field}"));
        }
        tauri::async_runtime::spawn(async move {
            let built =
                WebviewWindowBuilder::new(&app, "settings", WebviewUrl::App(settings_url.into()))
                    .title("Settings")
                    .decorations(false)
                    .transparent(true)
                    .shadow(false)
                    .inner_size(settings_width, settings_height)
                    .focused(true)
                    .visible_on_all_workspaces(true)
                    .build();
            if let Err(e) = built {
                error!("Failed to open settings window: {}", e);
                return;
            }

            // Send analytics event (fire and forget)
            let _ = analytics::send_analytics_event(&app_clone, "settings_opened", None).await;
        });
    }
    Ok(())
}

/// Canonical "bring the main window back": show + unminimize + focus.
/// Used by the hotkey handler, tray click and second-launch callback so the
/// sequence can't drift again.
pub(crate) fn reveal_main_window<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.show().map_err(|e| e.to_string())?;
        window.unminimize().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Show the updater window, building it on first use.
pub(crate) fn open_updater_window<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    if let Some(existing) = app.get_webview_window("updater") {
        existing.show()?;
        existing.set_focus()?;
    } else {
        WebviewWindowBuilder::new(app, "updater", WebviewUrl::App("/updater".into()))
            .title("Software Updates")
            .inner_size(500.0, 440.0)
            .center()
            .decorations(false)
            .transparent(true)
            .build()?;
    }
    Ok(())
}

/// Opens the "what's new" window shown once after an update (see
/// [`crate::config::AppConfig::needs_whats_new`]). Closing it (its button,
/// Esc, Alt+F4) records the running version as seen.
///
/// It opens even when Holdem started with Windows: the updater restarts the
/// app with the arguments it had, so the first start of a new version often
/// carries `--autostart`.
pub(crate) fn open_whats_new_window(app: &AppHandle) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        let url = format!("/whats-new?release={}", crate::config::WHATS_NEW_RELEASE);
        let built = WebviewWindowBuilder::new(&app, "whats-new", WebviewUrl::App(url.into()))
            .title("What's new in Holdem")
            .decorations(false)
            .transparent(true)
            .shadow(true)
            .inner_size(480.0, 640.0)
            // On a small screen, shrink to the work area; the text scrolls
            // and the buttons stay at the bottom.
            .prevent_overflow_with_margin(LogicalSize::new(16.0, 16.0))
            .center()
            .focused(true)
            .resizable(false)
            .build();
        let window = match built {
            Ok(window) => window,
            Err(e) => {
                error!("Failed to open the what's new window: {}", e);
                return;
            }
        };
        let handle = app.clone();
        window.on_window_event(move |event| {
            if let WindowEvent::Destroyed = event {
                if let Err(e) = crate::commands::config_ops::mark_whats_new_seen(&handle) {
                    error!("Failed to record the what's new window as seen: {}", e);
                }
            }
        });
    });
}

/// Pages the "what's new" window links to. Only these open: the page names
/// one, never a URL.
#[derive(Debug, Clone, Copy, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ProjectLink {
    Donate,
    FeatureRequest,
    ReportProblem,
    Changelog,
}

impl ProjectLink {
    fn url(self) -> String {
        const ISSUES: &str = "https://github.com/iamzubin/holdem/issues/new";
        match self {
            Self::Donate => "https://www.buymeacoffee.com/iamzubin".to_string(),
            Self::FeatureRequest => format!("{ISSUES}?labels=enhancement"),
            // The version helps triage; the reporter writes the rest.
            Self::ReportProblem => format!(
                "{ISSUES}?labels=bug&body=Holdem%20version%3A%20{}%0A%0A",
                crate::config::APP_VERSION
            ),
            Self::Changelog => {
                "https://github.com/iamzubin/holdem/blob/master/CHANGELOG.md".to_string()
            }
        }
    }
}

/// Opens one of Holdem's project pages in the default browser.
#[tauri::command]
pub fn open_project_link(link: ProjectLink) -> Result<(), String> {
    crate::utils::shell_open(&windows::core::HSTRING::from(link.url()))
        .map_err(|code| format!("Windows couldn't open the link (code {})", code))
}

#[tauri::command]
pub fn open_consent_window(app: AppHandle) -> Result<(), String> {
    if let Some(consent_window) = app.get_webview_window("consent") {
        consent_window.show().map_err(|e| e.to_string())?;
        consent_window.set_focus().map_err(|e| e.to_string())?;
    } else {
        // Create the consent window
        let app_clone = app.clone();
        tauri::async_runtime::spawn(async move {
            let built =
                WebviewWindowBuilder::new(&app_clone, "consent", WebviewUrl::App("/consent".into()))
                    .title("Analytics Consent")
                    .decorations(false)
                    .transparent(true)
                    .shadow(true)
                    .inner_size(460.0, 680.0)
                    .center()
                    .focused(true)
                    .always_on_top(true)
                    .skip_taskbar(true)
                    .resizable(false)
                    .visible_on_all_workspaces(true)
                    .build();
            if let Err(e) = built {
                error!("Failed to open consent window: {}", e);
            }
        });
    }
    Ok(())
}

#[tauri::command]
pub fn mark_drop_received(drag_state: State<'_, Arc<DragState>>) -> Result<(), String> {
    drag_state.successful_drop.store(true, Ordering::Relaxed);
    drag_state.drag_started.store(false, Ordering::Relaxed);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::ProjectLink;

    /// The names the what's new page sends (WhatsNew.tsx `LINKS`).
    #[test]
    fn project_links_read_the_names_the_page_sends() {
        for name in ["donate", "featureRequest", "reportProblem", "changelog"] {
            let json = format!("\"{name}\"");
            assert!(serde_json::from_str::<ProjectLink>(&json).is_ok(), "{name}");
        }
        assert!(serde_json::from_str::<ProjectLink>("\"https://example.com\"").is_err());
    }
}
