use crate::analytics;
use crate::DragState;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use crate::utils::{place_below_or_above, work_area_at, Rect};
use tauri::{AppHandle, Manager, Runtime, State, WebviewUrl, WebviewWindowBuilder};
use tracing::{error, info};

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

    // Popup size in logical pixels; placement is computed in physical pixels.
    let popup_width = 450.0;
    let popup_height = 350.0;
    let scale_factor = main_window.scale_factor().map_err(|e| e.to_string())?;
    let position = main_window.outer_position().map_err(|e| e.to_string())?;
    let size = main_window.outer_size().map_err(|e| e.to_string())?;
    let shelf = Rect {
        x: position.x,
        y: position.y,
        width: size.width as i32,
        height: size.height as i32,
    };
    let popup_size = (
        (popup_width * scale_factor) as i32,
        (popup_height * scale_factor) as i32,
    );
    let gap = (5.0 * scale_factor) as i32;

    // Centred under the shelf, flipped above it near the bottom of the
    // screen, and always kept inside the work area.
    let (x, y) = match work_area_at(&main_window, shelf.x + shelf.width / 2, shelf.y) {
        Some(area) => place_below_or_above(shelf, popup_size, area, gap),
        None => (
            shelf.x + (shelf.width - popup_size.0) / 2,
            shelf.y + shelf.height + gap,
        ),
    };
    let popup_x = x as f64 / scale_factor;
    let popup_y = y as f64 / scale_factor;
    info!("Opening popup window at logical ({}, {})", popup_x, popup_y);

    if let Some(popup_window) = app.get_webview_window("popup") {
        popup_window.close().map_err(|e| e.to_string())?;
    } else {
        // Create the popup window
        let app_clone = app.clone();
        let popup_url = url_with_theme("popup", theme);
        tauri::async_runtime::spawn(async move {
            let built = WebviewWindowBuilder::new(
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
            .build();
            if let Err(e) = built {
                error!("Failed to open popup window: {}", e);
                return;
            }

            // Send analytics event (fire and forget)
            let _ =
                analytics::send_analytics_event(&app_clone, "popup_window_opened", None).await;
        });
    }
    Ok(())
}

pub fn close_popup_window(app: AppHandle) -> Result<(), String> {
    let popup_window = app
        .get_webview_window("popup")
        .ok_or("Popup window not found")?;
    popup_window.close().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn open_settings_window(app: AppHandle, theme: Option<String>) -> Result<(), String> {
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
            let built =
                WebviewWindowBuilder::new(&app, "settings", WebviewUrl::App(settings_url.into()))
                    .title("Settings")
                    .decorations(false)
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
            .inner_size(500.0, 400.0)
            .center()
            .decorations(false)
            .build()?;
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
            let built =
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
