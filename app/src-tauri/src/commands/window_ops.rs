use tauri::{AppHandle, WebviewUrl, WebviewWindowBuilder, Manager};

#[tauri::command]
pub fn open_popup_window(app: AppHandle) -> Result<(), String> {
    // Get the main window
    let main_window = app
        .get_webview_window("main")
        .ok_or("Main window not found")?;

    // Get the position and size of the main window
    let _position = main_window.outer_position().map_err(|e| e.to_string())?;
    let _size = main_window.outer_size().map_err(|e| e.to_string())?;

    // Define popup window dimensions
    let popup_width = 450.0;
    let popup_height = 350.0;

    // Calculate the position for the popup window (centered below the main window)
    let popup_x = _position.x as f64 + (_size.width as f64 - popup_width) / 2.0;
    let popup_y = _position.y as f64 + _size.height as f64 + 5.0;

    if let Some(popup_window) = app.get_webview_window("popup") {
        popup_window.close().map_err(|e| e.to_string())?;
    } else {
        // Create the popup window
        tauri::async_runtime::spawn(async move {
            WebviewWindowBuilder::new(
                &app,
                "popup",                         // Window label
                WebviewUrl::App("popup".into()), // Assuming same frontend build
            )
            .title("File List")
            .decorations(false) // Remove window decorations for a popup feel
            .transparent(true)
            .shadow(false)
            .resizable(false)
            .inner_size(popup_width, popup_height)
            .position(popup_x, popup_y)
            .always_on_top(true)
            .focused(false)
            .build()
            .map_err(|e| e.to_string())?;
            Ok::<(), String>(())
        });
    }
    Ok(())
}

#[tauri::command]
pub fn close_popup_window(app: AppHandle) -> Result<(), String> {
    let popup_window = app
        .get_webview_window("popup")
        .ok_or("Popup window not found")?;
    popup_window.close().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn open_settings_window(app: AppHandle) -> Result<(), String> {
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
        tauri::async_runtime::spawn(async move {
            WebviewWindowBuilder::new(&app, "settings", WebviewUrl::App("settings".into()))
                .title("Settings")
                .decorations(false)
                .transparent(true)
                .shadow(false)
                .inner_size(settings_width, settings_height)
                .focused(true)
                .build()
                .map_err(|e| e.to_string())?;
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