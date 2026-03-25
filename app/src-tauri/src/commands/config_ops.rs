use std::sync::{Arc, Mutex};
use tauri::{AppHandle, State, Manager, Listener};
use tauri_plugin_autostart::ManagerExt;
use crate::config::AppConfig;
use crate::analytics;

#[cfg(target_os = "windows")]
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};

#[cfg(target_os = "macos")]
use tauri_plugin_key_intercept::{Hotkey, KeyInterceptExt, Modifiers as MacModifiers};

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
    let app_clone = app.clone();
    tauri::async_runtime::spawn(async move {
        let _ = analytics::send_app_restarted_event(&app_clone).await;
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
        let _ = analytics::send_autostart_toggled_event(&app_handle, enabled).await;
    });

    Ok(())
}

#[cfg(target_os = "macos")]
fn parse_keycode(key: &str) -> Option<i64> {
    let key_upper = key.to_uppercase();
    match key_upper.as_str() {
        "A" => Some(0),
        "B" => Some(1),
        "C" => Some(2),
        "D" => Some(3),
        "E" => Some(4),
        "F" => Some(5),
        "G" => Some(6),
        "H" => Some(7),
        "I" => Some(8),
        "J" => Some(9),
        "K" => Some(10),
        "L" => Some(11),
        "M" => Some(12),
        "N" => Some(13),
        "O" => Some(14),
        "P" => Some(15),
        "Q" => Some(16),
        "R" => Some(17),
        "S" => Some(18),
        "T" => Some(19),
        "U" => Some(20),
        "V" => Some(21),
        "W" => Some(22),
        "X" => Some(23),
        "Y" => Some(24),
        "Z" => Some(25),
        "SPACE" | "SPACEBAR" => Some(49),
        "RETURN" | "ENTER" => Some(36),
        "TAB" => Some(48),
        "DELETE" => Some(51),
        "ESCAPE" | "ESC" => Some(53),
        "F1" => Some(122),
        "F2" => Some(120),
        "F3" => Some(99),
        "F4" => Some(118),
        "F5" => Some(96),
        "F6" => Some(97),
        "F7" => Some(98),
        "F8" => Some(100),
        "F9" => Some(101),
        "F10" => Some(109),
        "F11" => Some(103),
        "F12" => Some(111),
        "UP" | "ARROWUP" => Some(126),
        "DOWN" | "ARROWDOWN" => Some(125),
        "LEFT" | "ARROWLEFT" => Some(123),
        "RIGHT" | "ARROWRIGHT" => Some(124),
        _ => None,
    }
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

    #[cfg(target_os = "windows")]
    {
        register_hotkey_windows(app_handle, shortcut_str)
    }

    #[cfg(target_os = "macos")]
    {
        register_hotkey_mac(app_handle, shortcut_str)
    }
}

#[cfg(target_os = "windows")]
fn register_hotkey_windows(app_handle: AppHandle, shortcut_str: String) -> Result<(), String> {
    let app_handle_clone = app_handle.clone();
    println!("Registering hotkey (Windows): {}", shortcut_str);

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
                    println!("Failed to parse key: {}", key);
                }
            }
        }
    }

    let shortcut = Shortcut::new(Some(modifiers), code);

    if let Err(e) = app_handle.global_shortcut().unregister_all() {
        println!("Failed to unregister all hotkeys: {}", e);
    }

    app_handle
        .global_shortcut()
        .register(shortcut)
        .map_err(|e| format!("Failed to register shortcut: {}", e))?;

    app_handle
        .global_shortcut()
        .on_shortcut(shortcut, move |_app, _shortcut, _event| {
            println!("Shortcut pressed");
            if let Some(window) = app_handle_clone.get_webview_window("main") {
                if let Err(e) = window.show() {
                    println!("Failed to show window: {}", e);
                    return;
                }
                if let Err(e) = window.set_focus() {
                    println!("Failed to focus window: {}", e);
                }
            }
        })
        .map_err(|e| format!("Failed to set shortcut callback: {}", e))?;

    println!("Hotkey registered successfully");

    let app_handle_clone = app_handle.clone();
    let shortcut_str_clone = shortcut_str.clone();
    tauri::async_runtime::spawn(async move {
        let _ = analytics::send_hotkey_registered_event(&app_handle_clone, &shortcut_str_clone).await;
    });

    Ok(())
}

#[cfg(target_os = "macos")]
fn register_hotkey_mac(app_handle: AppHandle, shortcut_str: String) -> Result<(), String> {
    let app_handle_clone = app_handle.clone();
    println!("Registering hotkey (macOS): {}", shortcut_str);

    let mut mac_modifiers = MacModifiers::empty();
    let mut keycode: Option<i64> = None;

    for part in shortcut_str.split('+') {
        let part = part.trim();
        match part.to_uppercase().as_str() {
            "CTRL" | "CONTROL" => mac_modifiers.control = true,
            "SHIFT" => mac_modifiers.shift = true,
            "ALT" | "OPT" | "OPTION" => mac_modifiers.option = true,
            "META" | "CMD" | "COMMAND" => mac_modifiers.command = true,
            key => {
                if let Some(kc) = parse_keycode(key) {
                    keycode = Some(kc);
                } else {
                    println!("Failed to parse key: {}", key);
                }
            }
        }
    }

    let keycode = keycode.ok_or_else(|| "No valid keycode found".to_string())?;

    let event_name = format!("hotkey-{}", shortcut_str.replace('+', "-").to_lowercase());

    let hotkey = Hotkey {
        keycodes: vec![keycode],
        modifiers: mac_modifiers,
        consume: false,
        event_name: event_name.clone(),
    };

    let monitor_state = app_handle.key_intercept();
    let manager = monitor_state.manager.lock()
        .map_err(|e| format!("Failed to lock manager: {}", e))?;

    manager.register(hotkey)
        .map_err(|e| format!("Failed to register hotkey: {}", e))?;

    drop(manager);

    let _listener = app_handle.listen(event_name.clone(), move |_event| {
        println!("Hotkey triggered: {}", event_name);
        if let Some(window) = app_handle_clone.get_webview_window("main") {
            if let Err(e) = window.show() {
                println!("Failed to show window: {}", e);
                return;
            }
            if let Err(e) = window.unminimize() {
                println!("Failed to unminimize window: {}", e);
            }
            if let Err(e) = window.set_focus() {
                println!("Failed to focus window: {}", e);
            }
        }
    });

    println!("Hotkey registered successfully");

    let app_handle_clone = app_handle.clone();
    let shortcut_str_clone = shortcut_str.clone();
    tauri::async_runtime::spawn(async move {
        let _ = analytics::send_hotkey_registered_event(&app_handle_clone, &shortcut_str_clone).await;
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
    let mut config = config.lock().map_err(|e| format!("Failed to lock config: {}", e))?;
    config.analytics_enabled = false;
    config.save(&app_handle)?;
    
    println!("[Analytics] User declined analytics consent");
    
    tauri::async_runtime::spawn(async move {
        let _ = analytics::send_consent_declined_event(&app_handle).await;
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

#[tauri::command]
#[cfg(target_os = "macos")]
pub fn check_input_monitoring_permission(_app_handle: AppHandle) -> Result<bool, String> {
    Ok(false)
}

#[tauri::command]
#[cfg(target_os = "macos")]
pub fn open_input_monitoring_settings(_app_handle: AppHandle) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
#[cfg(target_os = "windows")]
pub fn check_input_monitoring_permission(_app_handle: AppHandle) -> Result<bool, String> {
    Ok(true)
}

#[tauri::command]
#[cfg(target_os = "windows")]
pub fn open_input_monitoring_settings(_app_handle: AppHandle) -> Result<(), String> {
    Ok(())
}
