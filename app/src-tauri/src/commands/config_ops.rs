use crate::analytics;
use crate::config::AppConfig;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, State};
use tauri_plugin_autostart::ManagerExt;
use tracing::{error, info, warn};

use thiserror::Error;

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
    let analytics_enabled = new_config.analytics_enabled;
    if !analytics_enabled {
        apply_analytics_consent(&app_handle, false)?;
    }
    {
        let mut config = config
            .lock()
            .map_err(|e| format!("Failed to lock config: {}", e))?;
        *config = new_config;
        config.save(&app_handle)?;
    }
    if analytics_enabled {
        apply_analytics_consent(&app_handle, true)?;
    }
    Ok(())
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

#[derive(Debug, Error, PartialEq, Eq)]
pub enum HotkeyError {
    #[error("Hotkey \"{0}\" has no key; add a key such as \"Ctrl+Shift+N\"")]
    MissingKey(String),
    #[error("Hotkey \"{0}\" contains an empty part")]
    EmptyPart(String),
    #[error("Hotkey \"{0}\" has more than one non-modifier key")]
    MultipleKeys(String),
    #[error("Unsupported hotkey key \"{0}\"")]
    UnsupportedKey(String),
}

/// Resolves a single non-modifier key name (e.g. `N`, `1`, `Space`,
/// `ArrowUp`, `F5`, `Comma`, `Numpad1`) to a [`Code`]. Delegates to
/// global-hotkey's key table, which accepts both `KeyboardEvent.code` names
/// and common short aliases, case-insensitively.
fn parse_key_code(key: &str) -> Result<Code, HotkeyError> {
    key.parse::<Shortcut>()
        .map(|shortcut| shortcut.key)
        .map_err(|_| HotkeyError::UnsupportedKey(key.to_string()))
}

/// Parses a `Mod+Mod+Key` string into a [`Shortcut`]. Exactly one
/// non-modifier key is required; unknown keys are an error rather than being
/// silently replaced.
pub fn parse_hotkey(shortcut_str: &str) -> Result<Shortcut, HotkeyError> {
    let mut modifiers = Modifiers::empty();
    let mut code: Option<Code> = None;

    for part in shortcut_str.split('+') {
        let part = part.trim();
        if part.is_empty() {
            return Err(HotkeyError::EmptyPart(shortcut_str.to_string()));
        }
        match part.to_uppercase().as_str() {
            "CTRL" | "CONTROL" => modifiers |= Modifiers::CONTROL,
            "SHIFT" => modifiers |= Modifiers::SHIFT,
            "ALT" | "OPTION" => modifiers |= Modifiers::ALT,
            "META" | "WIN" | "SUPER" | "COMMAND" | "CMD" => modifiers |= Modifiers::META,
            _ => {
                if code.is_some() {
                    return Err(HotkeyError::MultipleKeys(shortcut_str.to_string()));
                }
                code = Some(parse_key_code(part)?);
            }
        }
    }

    let code = code.ok_or_else(|| HotkeyError::MissingKey(shortcut_str.to_string()))?;
    Ok(Shortcut::new(Some(modifiers), code))
}

#[tauri::command]
pub fn register_hotkey(app_handle: AppHandle, shortcut_str: String) -> Result<(), String> {
    if shortcut_str.trim().is_empty() {
        // Clearing the hotkey must also drop the currently registered one.
        app_handle
            .global_shortcut()
            .unregister_all()
            .map_err(|e| format!("Failed to unregister hotkey: {}", e))?;
        info!("Hotkey cleared");
        return Ok(());
    }

    // Parse before touching the existing registration so an invalid hotkey
    // leaves the previous one working.
    let shortcut = parse_hotkey(&shortcut_str).map_err(|e| e.to_string())?;

    let app_handle_clone = app_handle.clone();
    info!("Registering Windows hotkey: {}", shortcut_str);

    if let Err(e) = app_handle.global_shortcut().unregister_all() {
        warn!("Failed to unregister existing hotkeys: {}", e);
    }

    // `on_shortcut` registers the shortcut and its handler in one step; a
    // separate `register` call first makes it fail with "already registered".
    app_handle
        .global_shortcut()
        .on_shortcut(shortcut, move |_app, _shortcut, _event| {
            if let Err(e) = super::window_ops::reveal_main_window(&app_handle_clone) {
                error!("Failed to reveal window after hotkey press: {}", e);
            }
        })
        .map_err(|e| format!("Failed to register shortcut: {}", e))?;

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

/// Pushes the consent choice into the running analytics service. Opting out
/// takes effect before this returns and sends nothing; opting in lazily
/// creates the PostHog client and records `consent_accepted` once per
/// transition.
fn apply_analytics_consent(app_handle: &AppHandle, enabled: bool) -> Result<(), String> {
    let changed = analytics::set_analytics_enabled(app_handle, enabled)?;
    if !enabled {
        return Ok(());
    }

    let state = analytics::get_analytics_service(app_handle)?;
    let app_handle = app_handle.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = analytics::ensure_client(&state).await {
            warn!("Failed to initialize analytics: {}", e);
            return;
        }
        if changed {
            let _ = analytics::send_analytics_event(&app_handle, "consent_accepted", None).await;
        }
    });
    Ok(())
}

fn persist_analytics_consent(
    config: &Mutex<AppConfig>,
    app_handle: &AppHandle,
    enabled: bool,
) -> Result<(), String> {
    let mut config = config
        .lock()
        .map_err(|e| format!("Failed to lock config: {}", e))?;
    config.analytics_enabled = enabled;
    config.save(app_handle)
}

#[tauri::command]
pub fn accept_analytics_consent(
    config: State<Arc<Mutex<AppConfig>>>,
    app_handle: AppHandle,
) -> Result<(), String> {
    persist_analytics_consent(&config, &app_handle, true)?;
    info!("Analytics consent accepted");
    apply_analytics_consent(&app_handle, true)
}

#[tauri::command]
pub fn decline_analytics_consent(
    config: State<Arc<Mutex<AppConfig>>>,
    app_handle: AppHandle,
) -> Result<(), String> {
    // Stop the live service first so nothing is sent even if persisting fails.
    // No "consent_declined" event is sent: opting out means no further events.
    apply_analytics_consent(&app_handle, false)?;
    persist_analytics_consent(&config, &app_handle, false)?;
    info!("Analytics consent declined");
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

#[cfg(test)]
mod tests {
    use super::*;

    fn shortcut(mods: Modifiers, code: Code) -> Shortcut {
        Shortcut::new(Some(mods), code)
    }

    #[test]
    fn parses_letters_and_modifiers() {
        assert_eq!(
            parse_hotkey("Ctrl+Shift+N"),
            Ok(shortcut(Modifiers::CONTROL | Modifiers::SHIFT, Code::KeyN))
        );
        assert_eq!(
            parse_hotkey("control + alt + q"),
            Ok(shortcut(Modifiers::CONTROL | Modifiers::ALT, Code::KeyQ))
        );
        assert_eq!(
            parse_hotkey("Win+E"),
            Ok(shortcut(Modifiers::META, Code::KeyE))
        );
    }

    #[test]
    fn parses_digits() {
        assert_eq!(
            parse_hotkey("Ctrl+1"),
            Ok(shortcut(Modifiers::CONTROL, Code::Digit1))
        );
        assert_eq!(
            parse_hotkey("Alt+0"),
            Ok(shortcut(Modifiers::ALT, Code::Digit0))
        );
        assert_eq!(
            parse_hotkey("Ctrl+Digit9"),
            Ok(shortcut(Modifiers::CONTROL, Code::Digit9))
        );
    }

    #[test]
    fn parses_space_arrows_and_function_keys() {
        assert_eq!(
            parse_hotkey("Ctrl+Space"),
            Ok(shortcut(Modifiers::CONTROL, Code::Space))
        );
        assert_eq!(
            parse_hotkey("Ctrl+ArrowUp"),
            Ok(shortcut(Modifiers::CONTROL, Code::ArrowUp))
        );
        assert_eq!(
            parse_hotkey("Alt+Left"),
            Ok(shortcut(Modifiers::ALT, Code::ArrowLeft))
        );
        assert_eq!(
            parse_hotkey("F5"),
            Ok(shortcut(Modifiers::empty(), Code::F5))
        );
        assert_eq!(
            parse_hotkey("Shift+F12"),
            Ok(shortcut(Modifiers::SHIFT, Code::F12))
        );
    }

    #[test]
    fn parses_punctuation_and_numpad_codes() {
        assert_eq!(
            parse_hotkey("Ctrl+Comma"),
            Ok(shortcut(Modifiers::CONTROL, Code::Comma))
        );
        assert_eq!(
            parse_hotkey("Ctrl+/"),
            Ok(shortcut(Modifiers::CONTROL, Code::Slash))
        );
        assert_eq!(
            parse_hotkey("Ctrl+Numpad1"),
            Ok(shortcut(Modifiers::CONTROL, Code::Numpad1))
        );
        assert_eq!(
            parse_hotkey("Ctrl+Escape"),
            Ok(shortcut(Modifiers::CONTROL, Code::Escape))
        );
    }

    #[test]
    fn rejects_unknown_keys_instead_of_falling_back() {
        assert_eq!(
            parse_hotkey("Ctrl+Foo"),
            Err(HotkeyError::UnsupportedKey("Foo".to_string()))
        );
        assert_eq!(
            parse_hotkey("Ctrl+IntlBackslash"),
            Err(HotkeyError::UnsupportedKey("IntlBackslash".to_string()))
        );
    }

    #[test]
    fn rejects_malformed_hotkeys() {
        assert_eq!(
            parse_hotkey("Ctrl+Shift"),
            Err(HotkeyError::MissingKey("Ctrl+Shift".to_string()))
        );
        assert_eq!(
            parse_hotkey("Ctrl+ "),
            Err(HotkeyError::EmptyPart("Ctrl+ ".to_string()))
        );
        assert_eq!(
            parse_hotkey("Ctrl+A+B"),
            Err(HotkeyError::MultipleKeys("Ctrl+A+B".to_string()))
        );
    }
}
