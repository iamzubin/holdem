use crate::analytics;
use crate::config::{AppConfig, APP_VERSION};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager, State};
use tauri_plugin_autostart::ManagerExt;
use tracing::{info, warn};

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
    let new_config = new_config.clamped();
    let analytics_enabled = new_config.analytics_enabled;
    if !analytics_enabled {
        analytics::apply_consent(&app_handle, false).map_err(|e| e.to_string())?;
    }
    {
        let mut config = config
            .lock()
            .map_err(|e| format!("Failed to lock config: {}", e))?;
        let new_config = new_config.keeping_app_state(&config);
        // Write first so a failed save leaves memory matching the disk.
        new_config.save(&app_handle)?;
        *config = new_config;
    }
    if analytics_enabled {
        analytics::apply_consent(&app_handle, true).map_err(|e| e.to_string())?;
    }
    Ok(())
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
    #[error("Hotkey \"{0}\" needs Ctrl, Alt, Shift or Win; only F1-F24 work alone")]
    MissingModifier(String),
}

/// F1-F24 are rarely typed, so they may be a hotkey on their own. Any other
/// bare key would be swallowed system-wide and break typing in every app.
fn is_function_key(code: Code) -> bool {
    matches!(
        code,
        Code::F1
            | Code::F2
            | Code::F3
            | Code::F4
            | Code::F5
            | Code::F6
            | Code::F7
            | Code::F8
            | Code::F9
            | Code::F10
            | Code::F11
            | Code::F12
            | Code::F13
            | Code::F14
            | Code::F15
            | Code::F16
            | Code::F17
            | Code::F18
            | Code::F19
            | Code::F20
            | Code::F21
            | Code::F22
            | Code::F23
            | Code::F24
    )
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
/// non-modifier key is required, plus at least one modifier unless the key is
/// F1-F24; unknown keys are an error rather than being silently replaced.
/// Also accepts the `KeyboardEvent.key` strings older
/// builds saved (e.g. `Ctrl+,`, `Ctrl+ArrowUp`, and `Ctrl+ ` for Space).
pub fn parse_hotkey(shortcut_str: &str) -> Result<Shortcut, HotkeyError> {
    let mut modifiers = Modifiers::empty();
    let mut code: Option<Code> = None;

    for raw in shortcut_str.split('+') {
        let part = raw.trim();
        // Older builds saved `KeyboardEvent.key`, which is " " for Space.
        let part = if part.is_empty() && !raw.is_empty() {
            "Space"
        } else {
            part
        };
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
    if modifiers.is_empty() && !is_function_key(code) {
        return Err(HotkeyError::MissingModifier(shortcut_str.to_string()));
    }
    Ok(Shortcut::new(Some(modifiers), code))
}

/// The shortcut currently registered by [`register_hotkey`], kept so a failed
/// registration can restore it.
static ACTIVE_HOTKEY: Mutex<Option<Shortcut>> = Mutex::new(None);

/// True when `shortcut_str` is the hotkey currently registered. Lets Settings
/// retry a saved hotkey whose registration failed (e.g. at startup).
#[tauri::command]
pub fn is_hotkey_active(shortcut_str: String) -> Result<bool, String> {
    let active = ACTIVE_HOTKEY
        .lock()
        .map_err(|e| format!("Failed to lock hotkey state: {}", e))?;
    Ok(match parse_hotkey(&shortcut_str) {
        Ok(shortcut) => *active == Some(shortcut),
        Err(_) => false,
    })
}

#[tauri::command]
pub fn register_hotkey(app_handle: AppHandle, shortcut_str: String) -> Result<(), String> {
    let mut active = ACTIVE_HOTKEY
        .lock()
        .map_err(|e| format!("Failed to lock hotkey state: {}", e))?;

    if shortcut_str.trim().is_empty() {
        // Clearing the hotkey must also drop the currently registered one.
        app_handle
            .global_shortcut()
            .unregister_all()
            .map_err(|e| format!("Failed to unregister hotkey: {}", e))?;
        *active = None;
        info!("Hotkey cleared");
        return Ok(());
    }

    // Parse before touching the existing registration so an invalid hotkey
    // leaves the previous one working.
    let shortcut = parse_hotkey(&shortcut_str).map_err(|e| e.to_string())?;
    if *active == Some(shortcut) {
        return Ok(());
    }

    info!("Registering Windows hotkey: {}", shortcut_str);

    if let Err(e) = app_handle.global_shortcut().unregister_all() {
        warn!("Failed to unregister existing hotkeys: {}", e);
    }

    // Presses are handled by the plugin-wide handler in lib.rs, so only the
    // registration is needed here.
    if let Err(e) = app_handle.global_shortcut().register(shortcut) {
        // e.g. another app owns the combo: keep the previous hotkey working.
        if let Some(previous) = *active {
            if let Err(restore) = app_handle.global_shortcut().register(previous) {
                warn!("Failed to restore previous hotkey: {}", restore);
                *active = None;
            }
        }
        return Err(format!("Failed to register shortcut: {}", e));
    }
    *active = Some(shortcut);
    drop(active);

    info!("Windows hotkey registered successfully");

    // Only that a hotkey was set, never which keys.
    let app_handle_clone = app_handle.clone();
    tauri::async_runtime::spawn(async move {
        let _ = analytics::send_analytics_event(&app_handle_clone, "hotkey_registered", None).await;
    });

    Ok(())
}

/// Saves the first-run choices: analytics consent, and (when given) whether
/// Holdem starts on login, which also updates the Windows startup entry.
fn persist_analytics_consent(
    config: &Mutex<AppConfig>,
    app_handle: &AppHandle,
    enabled: bool,
    start_on_login: Option<bool>,
) -> Result<(), String> {
    let mut config = config
        .lock()
        .map_err(|e| format!("Failed to lock config: {}", e))?;
    let mut updated = config.clone();
    updated.analytics_enabled = enabled;
    if let Some(start) = start_on_login {
        // Compare with the real startup entry, not the stored flag: the
        // default is "on" but nothing is registered until this choice.
        let manager = app_handle.autolaunch();
        if manager.is_enabled().unwrap_or(!start) != start {
            let result = if start { manager.enable() } else { manager.disable() };
            result.map_err(|e| e.to_string())?;
        }
        updated.autostart = start;
    }
    updated.save(app_handle)?;
    *config = updated;
    Ok(())
}

#[tauri::command]
pub fn accept_analytics_consent(
    config: State<Arc<Mutex<AppConfig>>>,
    app_handle: AppHandle,
    start_on_login: Option<bool>,
) -> Result<(), String> {
    persist_analytics_consent(&config, &app_handle, true, start_on_login)?;
    info!("Analytics consent accepted");
    analytics::apply_consent(&app_handle, true).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn decline_analytics_consent(
    config: State<Arc<Mutex<AppConfig>>>,
    app_handle: AppHandle,
    start_on_login: Option<bool>,
) -> Result<(), String> {
    // Stop the live service first so nothing is sent even if persisting fails.
    // No "consent_declined" event is sent: opting out means no further events.
    analytics::apply_consent(&app_handle, false).map_err(|e| e.to_string())?;
    persist_analytics_consent(&config, &app_handle, false, start_on_login)?;
    info!("Analytics consent declined");
    Ok(())
}

/// Records the running version as seen, so the "what's new" window doesn't
/// open again. Saved at once, so it holds even if Holdem is quit right after.
/// Quitting Holdem while the window is still open doesn't close it in a way
/// that reaches this (the event loop exits without destroying windows), so
/// the window shows again on the next start.
pub(crate) fn mark_whats_new_seen(app_handle: &AppHandle) -> Result<(), String> {
    let config = app_handle.state::<Arc<Mutex<AppConfig>>>();
    let mut config = config
        .lock()
        .map_err(|e| format!("Failed to lock config: {}", e))?;
    if config.last_seen_version.as_deref() == Some(APP_VERSION) {
        return Ok(());
    }
    let mut updated = config.clone();
    updated.last_seen_version = Some(APP_VERSION.to_string());
    updated.save(app_handle)?;
    *config = updated;
    info!("What's new window closed; recorded as seen");
    Ok(())
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
            parse_hotkey("Ctrl+"),
            Err(HotkeyError::EmptyPart("Ctrl+".to_string()))
        );
        assert_eq!(
            parse_hotkey("Ctrl++N"),
            Err(HotkeyError::EmptyPart("Ctrl++N".to_string()))
        );
        assert_eq!(
            parse_hotkey("Ctrl+A+B"),
            Err(HotkeyError::MultipleKeys("Ctrl+A+B".to_string()))
        );
    }

    #[test]
    fn rejects_bare_keys_except_function_keys() {
        for bare in ["N", "Escape", "Tab", "Space", "1", "ArrowUp"] {
            assert_eq!(
                parse_hotkey(bare),
                Err(HotkeyError::MissingModifier(bare.to_string()))
            );
        }
        assert_eq!(
            parse_hotkey("F24"),
            Ok(shortcut(Modifiers::empty(), Code::F24))
        );
    }

    #[test]
    fn parses_legacy_event_key_strings() {
        // Older builds saved `KeyboardEvent.key` for non-letter, non-digit keys.
        assert_eq!(
            parse_hotkey("Ctrl+ "),
            Ok(shortcut(Modifiers::CONTROL, Code::Space))
        );
        assert_eq!(
            parse_hotkey("Ctrl+Shift+,"),
            Ok(shortcut(Modifiers::CONTROL | Modifiers::SHIFT, Code::Comma))
        );
        assert_eq!(
            parse_hotkey("Alt+ArrowUp"),
            Ok(shortcut(Modifiers::ALT, Code::ArrowUp))
        );
        assert_eq!(
            parse_hotkey("Ctrl+Enter"),
            Ok(shortcut(Modifiers::CONTROL, Code::Enter))
        );
    }
}
