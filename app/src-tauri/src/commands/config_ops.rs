use std::sync::{Arc, Mutex};
use tauri::{AppHandle, State, Manager};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};
use tauri_plugin_autostart::ManagerExt;
use crate::config::AppConfig;
use crate::analytics;

#[tauri::command]
pub fn get_config(config: State<Arc<Mutex<AppConfig>>>) -> Result<AppConfig, String> {
    config.lock().map_err(|e| format!("Failed to lock config: {}", e)).map(|c| c.clone())
}

#[tauri::command]
pub fn save_config(
    new_config: AppConfig,
    config: State<Arc<Mutex<AppConfig>>>,
    app_handle: AppHandle,
) -> Result<(), String> {
    let mut config = config.lock().map_err(|e| format!("Failed to lock config: {}", e))?;
    *config = new_config;
    config.save(&app_handle)
}

#[tauri::command]
pub fn restart_app(app: AppHandle) -> Result<(), String> {
    // Send analytics event for app restart
    let app_clone = app.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = analytics::send_app_restarted_event(&app_clone).await {
            eprintln!("[Analytics] Failed to send app_restarted event: {}", e);
        }
    });
    
    app.restart();
}

#[tauri::command]
pub fn set_autostart(app_handle: AppHandle, enabled: bool) -> Result<(), String> {
    let autostart_manager = app_handle.autolaunch();

    if enabled {
        autostart_manager.enable().map_err(|e| e.to_string())?;
    } else {
        autostart_manager.disable().map_err(|e| e.to_string())?;
    }

    // Send analytics event
    tauri::async_runtime::spawn(async move {
        if let Err(e) = analytics::send_autostart_toggled_event(&app_handle, enabled).await {
            eprintln!("[Analytics] Failed to send autostart_toggled event: {}", e);
        }
    });

    Ok(())
}

#[tauri::command]
pub fn register_hotkey(app_handle: AppHandle, shortcut_str: String) -> Result<(), String> {
    if shortcut_str.is_empty() {
        return Ok(());
    }

    let app_handle_clone = app_handle.clone();
    println!("Registering hotkey: {}", shortcut_str);

    // Parse the shortcut string
    let mut modifiers = Modifiers::empty();
    let mut code = Code::KeyN; // Default key

    for part in shortcut_str.split('+') {
        let part = part.trim();
        println!("part: {}", part);
        match part.to_uppercase().as_str() {
            "CTRL" | "Ctrl" | "CONTROL" => modifiers |= Modifiers::CONTROL,
            "SHIFT" | "Shift" => modifiers |= Modifiers::SHIFT,
            "ALT" | "Alt" => modifiers |= Modifiers::ALT,
            "META" | "Meta" | "WIN" | "Win" | "COMMAND" => modifiers |= Modifiers::META,
            key => {
                // Handle letter keys
                if key.len() == 1 && key.chars().next().unwrap().is_alphabetic() {
                    code = match key {
                        "A" => Code::KeyA,
                        "B" => Code::KeyB,
                        "C" => Code::KeyC,
                        "D" => Code::KeyD,
                        "E" => Code::KeyE,
                        "F" => Code::KeyF,
                        "G" => Code::KeyG,
                        "H" => Code::KeyH,
                        "I" => Code::KeyI,
                        "J" => Code::KeyJ,
                        "K" => Code::KeyK,
                        "L" => Code::KeyL,
                        "M" => Code::KeyM,
                        "N" => Code::KeyN,
                        "O" => Code::KeyO,
                        "P" => Code::KeyP,
                        "Q" => Code::KeyQ,
                        "R" => Code::KeyR,
                        "S" => Code::KeyS,
                        "T" => Code::KeyT,
                        "U" => Code::KeyU,
                        "V" => Code::KeyV,
                        "W" => Code::KeyW,
                        "X" => Code::KeyX,
                        "Y" => Code::KeyY,
                        "Z" => Code::KeyZ,
                        _ => Code::KeyN,
                    };
                } else {
                    // Try to parse other keys
                    if let Ok(k) = key.parse::<Code>() {
                        code = k;
                    } else {
                        println!("Failed to parse key: {}", key);
                    }
                }
            }
        }
    }

    println!("Final code: {:?}", code);
    println!("Final modifiers: {:?}", modifiers);

    let shortcut = Shortcut::new(Some(modifiers), code);
    println!("Created shortcut: {:?}", shortcut);

    // First unregister all hotkeys
    if let Err(e) = app_handle.global_shortcut().unregister_all() {
        println!("Failed to unregister all hotkeys: {}", e);
    }

    // Register the shortcut
    app_handle
        .global_shortcut()
        .register(shortcut)
        .map_err(|e| format!("Failed to register shortcut: {}", e))?;

    // Set up the callback to show the window
    app_handle
        .global_shortcut()
        .on_shortcut(shortcut, move |_app, _shortcut, _event| {
            println!("Shortcut pressed");
            if let Some(window) = app_handle_clone.get_webview_window("main") {
                // Show the window first
                if let Err(e) = window.show() {
                    println!("Failed to show window: {}", e);
                    return;
                }

                // Then try to focus it
                if let Err(e) = window.set_focus() {
                    println!("Failed to focus window: {}", e);
                }
            }
        })
        .map_err(|e| format!("Failed to set shortcut callback: {}", e))?;

    println!("Hotkey registered successfully");

    // Send analytics event
    let app_handle_clone = app_handle.clone();
    let shortcut_str_clone = shortcut_str.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = analytics::send_hotkey_registered_event(&app_handle_clone, &shortcut_str_clone).await {
            eprintln!("[Analytics] Failed to send hotkey_registered event: {}", e);
        }
    });

    Ok(())
}

#[tauri::command]
pub fn accept_analytics_consent(
    config: State<Arc<Mutex<AppConfig>>>,
    app_handle: AppHandle,
) -> Result<(), String> {
    let mut config = config.lock().map_err(|e| format!("Failed to lock config: {}", e))?;
    config.analytics_enabled = true;
    config.save(&app_handle)?;
    
    println!("[Analytics] User accepted analytics consent");
    
    // Send initial analytics event after consent using centralized service
    tauri::async_runtime::spawn(async move {
        if let Err(e) = analytics::send_analytics_event(&app_handle, "consent_accepted", None).await {
            eprintln!("[Analytics] Failed to send consent_accepted event: {}", e);
        }
    });
    
    Ok(())
}

#[tauri::command]
pub fn decline_analytics_consent(
    config: State<Arc<Mutex<AppConfig>>>,
    app_handle: AppHandle,
) -> Result<(), String> {
    let mut config = config.lock().map_err(|e| format!("Failed to lock config: {}", e))?;
    config.analytics_enabled = false;
    config.save(&app_handle)?;
    
    println!("[Analytics] User declined analytics consent");
    
    // Send analytics event for declined consent
    tauri::async_runtime::spawn(async move {
        if let Err(e) = analytics::send_consent_declined_event(&app_handle).await {
            eprintln!("[Analytics] Failed to send consent_declined event: {}", e);
        }
    });
    
    Ok(())
}

#[tauri::command]
pub fn check_analytics_consent(config: State<Arc<Mutex<AppConfig>>>) -> Result<bool, String> {
    config.lock().map_err(|e| format!("Failed to lock config: {}", e)).map(|c| c.analytics_enabled)
}

#[tauri::command]
pub fn check_config_exists(app_handle: AppHandle) -> bool {
    AppConfig::config_exists(&app_handle)
} 