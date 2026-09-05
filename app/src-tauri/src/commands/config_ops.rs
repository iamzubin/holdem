use crate::analytics;
use crate::config::AppConfig;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, State};
use tauri_plugin_autostart::ManagerExt;
use tracing::{error, info, warn};

use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};



#[tauri::command]
pub fn get_config(config: State<Arc<Mutex<AppConfig>>>) -> Result<AppConfig, String> {
    config
        .lock()
        .map_err(|e| format!("Failed to lock config: {}", e))
        .map(|c| c.clone())
}

#[tauri::command]
pub fn save_config(
    new_config: AppConfig,
    config: State<Arc<Mutex<AppConfig>>>,
    app_handle: AppHandle,
) -> Result<(), String> {
    let mut config = config
        .lock()
        .map_err(|e| format!("Failed to lock config: {}", e))?;
    *config = new_config;
    config.save(&app_handle)
}

#[tauri::command]
pub fn restart_app(app: AppHandle) -> Result<(), String> {
    let app_clone = app.clone();
    tauri::async_runtime::spawn(async move {
        let _ = analytics::send_analytics_event(&app_clone, "app_restarted", None).await;
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

    tauri::async_runtime::spawn(async move {
        let _ = analytics::send_analytics_event(
            &app_handle,
            "autostart_toggled",
            Some(vec![("enabled", serde_json::Value::Bool(enabled))]),
        )
        .await;
    });

    Ok(())
}

#[cfg(target_os = "windows")]
fn parse_windows_code(key: &str) -> Option<tauri_plugin_global_shortcut::Code> {
    let key_upper = key.to_uppercase();
    match key_upper.as_str() {
        "A" => Some(Code::KeyA),
        "B" => Some(Code::KeyB),
        "C" => Some(Code::KeyC),
        "D" => Some(Code::KeyD),
        "E" => Some(Code::KeyE),
        "F" => Some(Code::KeyF),
        "G" => Some(Code::KeyG),
        "H" => Some(Code::KeyH),
        "I" => Some(Code::KeyI),
        "J" => Some(Code::KeyJ),
        "K" => Some(Code::KeyK),
        "L" => Some(Code::KeyL),
        "M" => Some(Code::KeyM),
        "N" => Some(Code::KeyN),
        "O" => Some(Code::KeyO),
        "P" => Some(Code::KeyP),
        "Q" => Some(Code::KeyQ),
        "R" => Some(Code::KeyR),
        "S" => Some(Code::KeyS),
        "T" => Some(Code::KeyT),
        "U" => Some(Code::KeyU),
        "V" => Some(Code::KeyV),
        "W" => Some(Code::KeyW),
        "X" => Some(Code::KeyX),
        "Y" => Some(Code::KeyY),
        "Z" => Some(Code::KeyZ),
        "SPACE" => Some(Code::Space),
        "RETURN" | "ENTER" => Some(Code::Enter),
        "TAB" => Some(Code::Tab),
        "ESCAPE" | "ESC" => Some(Code::Escape),
        "F1" => Some(Code::F1),
        "F2" => Some(Code::F2),
        "F3" => Some(Code::F3),
        "F4" => Some(Code::F4),
        "F5" => Some(Code::F5),
        "F6" => Some(Code::F6),
        "F7" => Some(Code::F7),
        "F8" => Some(Code::F8),
        "F9" => Some(Code::F9),
        "F10" => Some(Code::F10),
        "F11" => Some(Code::F11),
        "F12" => Some(Code::F12),
        _ => key_upper.parse::<Code>().ok(),
    }
}

#[tauri::command]
pub fn register_hotkey(app_handle: AppHandle, shortcut_str: String) -> Result<(), String> {
    if shortcut_str.is_empty() {
        return Ok(());
    }

    let app_handle_clone = app_handle.clone();
    info!("Registering Windows hotkey: {}", shortcut_str);

    let mut modifiers = Modifiers::empty();
    let mut code = Code::KeyN;

    for part in shortcut_str.split('+') {
        let part = part.trim();
        match part.to_uppercase().as_str() {
            "CTRL" | "CONTROL" => modifiers |= Modifiers::CONTROL,
            "SHIFT" => modifiers |= Modifiers::SHIFT,
            "ALT" => modifiers |= Modifiers::ALT,
            "META" | "WIN" | "COMMAND" => modifiers |= Modifiers::META,
            key => {
                if let Some(parsed_code) = parse_windows_code(key) {
                    code = parsed_code;
                } else {
                    warn!("Failed to parse Windows hotkey key: {}", key);
                }
            }
        }
    }

    let shortcut = Shortcut::new(Some(modifiers), code);

    if let Err(e) = app_handle.global_shortcut().unregister_all() {
        warn!("Failed to unregister existing hotkeys: {}", e);
    }

    app_handle
        .global_shortcut()
        .register(shortcut)
        .map_err(|e| format!("Failed to register shortcut: {}", e))?;

    app_handle
        .global_shortcut()
        .on_shortcut(shortcut, move |_app, _shortcut, _event| {
            if let Err(e) = super::window_ops::reveal_main_window(&app_handle_clone) {
                error!("Failed to reveal window after hotkey press: {}", e);
            }
        })
        .map_err(|e| format!("Failed to set shortcut callback: {}", e))?;

    info!("Windows hotkey registered successfully");

    let app_handle_clone = app_handle.clone();
    let shortcut_str_clone = shortcut_str.clone();
    tauri::async_runtime::spawn(async move {
        let _ = analytics::send_analytics_event(
            &app_handle_clone,
            "hotkey_registered",
            Some(vec![(
                "hotkey",
                serde_json::Value::String(shortcut_str_clone),
            )]),
        )
        .await;
    });

    Ok(())
}

#[tauri::command]
pub fn accept_analytics_consent(
    config: State<Arc<Mutex<AppConfig>>>,
    app_handle: AppHandle,
) -> Result<(), String> {
    let mut config = config
        .lock()
        .map_err(|e| format!("Failed to lock config: {}", e))?;
    config.analytics_enabled = true;
    config.save(&app_handle)?;

    info!("Analytics consent accepted");

    tauri::async_runtime::spawn(async move {
        let _ = analytics::send_analytics_event(&app_handle, "consent_accepted", None).await;
    });

    Ok(())
}

#[tauri::command]
pub fn decline_analytics_consent(
    config: State<Arc<Mutex<AppConfig>>>,
    app_handle: AppHandle,
) -> Result<(), String> {
    let mut config = config
        .lock()
        .map_err(|e| format!("Failed to lock config: {}", e))?;
    config.analytics_enabled = false;
    config.save(&app_handle)?;

    info!("Analytics consent declined");

    tauri::async_runtime::spawn(async move {
        let _ = analytics::send_analytics_event(&app_handle, "consent_declined", None).await;
    });

    Ok(())
}

#[tauri::command]
pub fn check_analytics_consent(config: State<Arc<Mutex<AppConfig>>>) -> Result<bool, String> {
    config
        .lock()
        .map_err(|e| format!("Failed to lock config: {}", e))
        .map(|c| c.analytics_enabled)
}

#[tauri::command]
pub fn check_config_exists(app_handle: AppHandle) -> bool {
    AppConfig::get_config_path(&app_handle)
        .map(|p| p.exists())
        .unwrap_or(false)
}
