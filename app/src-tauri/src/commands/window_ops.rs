use crate::analytics;
use crate::DragState;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, State, WebviewUrl, WebviewWindowBuilder};
use tracing::{info, warn};
use windows::Win32::Foundation::POINT;
use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;

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

#[tauri::command]
pub fn open_popup_window(app: AppHandle, theme: Option<String>) -> Result<(), String> {
    // Get the main window
    let main_window = app
        .get_webview_window("main")
        .ok_or("Main window not found")?;

    // Get the scale factor to convert between physical and logical pixels
    let scale_factor = main_window.scale_factor().map_err(|e| e.to_string())?;

    // Get the position and size of the main window (physical pixels)
    let position = main_window.inner_position().map_err(|e| e.to_string())?;
    let size = main_window.inner_size().map_err(|e| e.to_string())?;

    // Convert to logical pixels for positioning
    let logical_x = (position.x as f64 / scale_factor) as i32;
    let logical_y = (position.y as f64 / scale_factor) as i32;
    let logical_width = (size.width as f64 / scale_factor) as i32;
    let logical_height = (size.height as f64 / scale_factor) as i32;

    // Define popup window dimensions (logical pixels)
    let popup_width = 450.0;
    let popup_height = 350.0;

    // Calculate centered position below the main window
    let popup_x = logical_x as f64 + (logical_width as f64 - popup_width) / 2.0;
    let popup_y = logical_y as f64 + logical_height as f64 + 5.0;

    info!(
        "Opening popup window - scale_factor={}, main_window: pos=({}, {}), size=({}, {}), logical=({}, {}, {}), popup_pos=({}, {}), popup_size=({}, {})",
        scale_factor, position.x, position.y, size.width, size.height, logical_x, logical_y, logical_height, popup_x, popup_y, popup_width, popup_height
    );

    if let Some(popup_window) = app.get_webview_window("popup") {
        popup_window.close().map_err(|e| e.to_string())?;
    } else {
        // Create the popup window
        let app_clone = app.clone();
        let popup_url = url_with_theme("popup", theme);
        tauri::async_runtime::spawn(async move {
            WebviewWindowBuilder::new(
                &app,
                "popup",                           // Window label
                WebviewUrl::App(popup_url.into()), // Same frontend build, themed via `?theme=`
            )
            .title("File List")
            .decorations(false) // Remove window decorations for a popup feel
            .shadow(false)
            .resizable(false)
            .inner_size(popup_width, popup_height)
            .position(popup_x, popup_y)
            .always_on_top(true)
            .focused(false)
            .accept_first_mouse(true)
            .visible_on_all_workspaces(true)
            .build()
            .map_err(|e: tauri::Error| e.to_string())?;

            // Send analytics event (fire and forget)
            std::mem::drop(analytics::send_popup_window_opened_event(&app_clone));

            Ok::<(), String>(())
        });
    }
    Ok(())
}

#[tauri::command]
pub fn close_popup_window(app: AppHandle) -> Result<(), String> {
    // A context menu belongs to the popup — never leave it dangling.
    if let Some(menu_window) = app.get_webview_window("contextmenu") {
        let _ = menu_window.close();
        let _ = app.emit("contextmenu-closed", ());
    }
    let popup_window = app
        .get_webview_window("popup")
        .ok_or("Popup window not found")?;
    popup_window.close().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn open_context_menu_window(
    app: AppHandle,
    selection: State<'_, crate::ContextMenuSelection>,
    theme: Option<String>,
    file_ids: Vec<u64>,
) -> Result<(), String> {
    // Remember what the menu should act on; the menu window reads it back
    // via `get_context_menu_selection` on mount.
    let selected_count = file_ids.len();
    *selection
        .lock()
        .map_err(|_| "Failed to lock context menu selection".to_string())? = file_ids;

    let main_window = app
        .get_webview_window("main")
        .ok_or("Main window not found")?;
    let scale_factor = main_window.scale_factor().map_err(|e| e.to_string())?;

    // Anchor the menu at the cursor (physical px) and keep it fully on-screen.
    let mut cursor = POINT { x: 0, y: 0 };
    if unsafe { GetCursorPos(&mut cursor) }.is_err() {
        warn!("GetCursorPos failed; anchoring context menu at monitor origin");
    }
    let menu_width = 230.0;
    let menu_height = 128.0;
    let (menu_x, menu_y) =
        match crate::utils::ScreenBounds::from_window(&main_window) {
            Ok(bounds) => {
                let w = menu_width * scale_factor;
                let h = menu_height * scale_factor;
                let x = (cursor.x as f64).min(bounds.x + bounds.width - w).max(bounds.x);
                let y = (cursor.y as f64).min(bounds.y + bounds.height - h).max(bounds.y);
                (x / scale_factor, y / scale_factor)
            }
            Err(_) => (
                cursor.x as f64 / scale_factor,
                cursor.y as f64 / scale_factor,
            ),
        };

    // Unlike popup/settings there is no toggle: a right-click elsewhere
    // repositions the menu instead of just dismissing it.
    // NOTE: destroy (not close) — close() is async and the label stays
    // reserved until teardown, so an immediate rebuild would fail with
    // "window label already exists" and the menu would never reopen.
    if let Some(existing) = app.get_webview_window("contextmenu") {
        let _ = existing.destroy();
    }

    let menu_url = url_with_theme("contextmenu", theme);
    info!(
        "Opening context menu window - cursor=({}, {}), menu_pos=({}, {}), menu_size=({}, {}), selected_files={}",
        cursor.x, cursor.y, menu_x, menu_y, menu_width, menu_height, selected_count
    );
    WebviewWindowBuilder::new(&app, "contextmenu", WebviewUrl::App(menu_url.into()))
        .title("Context Menu")
        .decorations(false)
        .shadow(true)
        .resizable(false)
        .skip_taskbar(true)
        .focused(false)
        // Non-activating: clicks land without stealing focus, so the popup
        // underneath never sees a blur-close while the menu is open.
        .focusable(false)
        .accept_first_mouse(true)
        .always_on_top(true)
        .visible_on_all_workspaces(true)
        .inner_size(menu_width, menu_height)
        .position(menu_x, menu_y)
        .build()
        .map_err(|e| e.to_string())?;
    // The popup tracks this to suppress its blur/inactivity auto-close
    // while the menu is open (see PopupWindow).
    let _ = app.emit("contextmenu-opened", ());
    Ok(())
}

#[tauri::command]
pub fn close_context_menu_window(app: AppHandle) -> Result<(), String> {
    // Idempotent: popup clicks dismiss the menu fire-and-forget.
    if let Some(menu_window) = app.get_webview_window("contextmenu") {
        menu_window.close().map_err(|e| e.to_string())?;
    }
    let _ = app.emit("contextmenu-closed", ());
    Ok(())
}

#[tauri::command]
pub fn get_context_menu_selection(
    selection: State<'_, crate::ContextMenuSelection>,
) -> Result<Vec<u64>, String> {
    selection
        .lock()
        .map(|ids| ids.clone())
        .map_err(|_| "Failed to lock context menu selection".to_string())
}

#[tauri::command]
pub fn open_settings_window(app: AppHandle, theme: Option<String>) -> Result<(), String> {
    // Get the main window
    let main_window = app
        .get_webview_window("main")
        .ok_or("Main window not found")?;

    // Get the position and size of the main window
    let _position = main_window.outer_position().map_err(|e| e.to_string())?;
    let _size = main_window.outer_size().map_err(|e| e.to_string())?;

    // Define settings window dimensions
    let settings_width = 500.0;
    let settings_height = 600.0;

    if let Some(settings_window) = app.get_webview_window("settings") {
        settings_window.close().map_err(|e| e.to_string())?;
    } else {
        // Create the settings window
        let app_clone = app.clone();
        let settings_url = url_with_theme("settings", theme);
        tauri::async_runtime::spawn(async move {
            WebviewWindowBuilder::new(&app, "settings", WebviewUrl::App(settings_url.into()))
                .title("Settings")
                .decorations(false)
                .shadow(false)
                .inner_size(settings_width, settings_height)
                .focused(true)
                .visible_on_all_workspaces(true)
                .build()
                .map_err(|e: tauri::Error| e.to_string())?;

            // Send analytics event (fire and forget)
            std::mem::drop(analytics::send_settings_opened_event(&app_clone));

            Ok::<(), String>(())
        });
    }
    Ok(())
}

#[tauri::command]
pub fn close_settings_window(app: AppHandle) -> Result<(), String> {
    if let Some(settings_window) = app.get_webview_window("settings") {
        settings_window.close().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn show_main_window(app_handle: AppHandle) -> Result<(), String> {
    if let Some(window) = app_handle.get_webview_window("main") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
    }
    Ok(())
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
            WebviewWindowBuilder::new(&app_clone, "consent", WebviewUrl::App("/consent".into()))
                .title("Analytics Consent")
                .decorations(false)
                .shadow(true)
                .inner_size(450.0, 500.0)
                .center()
                .focused(true)
                .always_on_top(true)
                .skip_taskbar(true)
                .resizable(false)
                .visible_on_all_workspaces(true)
                .build()
                .map_err(|e: tauri::Error| e.to_string())?;
            Ok::<(), String>(())
        });
    }
    Ok(())
}

#[tauri::command]
pub fn close_consent_window(app: AppHandle) -> Result<(), String> {
    if let Some(consent_window) = app.get_webview_window("consent") {
        consent_window.close().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn mark_drop_received(drag_state: State<'_, Arc<DragState>>) -> Result<(), String> {
    drag_state.successful_drop.store(true, Ordering::Relaxed);
    drag_state.drag_started.store(false, Ordering::Relaxed);
    Ok(())
}
