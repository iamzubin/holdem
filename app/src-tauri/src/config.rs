use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tauri::AppHandle;
use tauri::Manager;
use tracing::{error, info, warn};
use uuid::Uuid;

// `#[serde(default)]` fills any field missing from an older (or newer)
// config.json from `Default`, so adding a setting never makes existing
// configs unparseable and silently resets them.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct AppConfig {
    pub mouse_monitor: MouseMonitorConfig,
    pub autostart: bool,
    pub hotkey: String,
    pub analytics_enabled: bool,
    pub analytics_uuid: String,
    /// Windows 11 Mica backdrop behind Holdem's windows.
    pub mica: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct MouseMonitorConfig {
    pub required_shakes: u32,
    pub shake_time_limit: u64,
    pub shake_threshold: i32,
    pub window_close_delay: u64,
    /// When non-empty, shaking works only in these apps (older configs).
    pub whitelist: Vec<String>,
    /// Apps where shaking never opens the shelf.
    pub blocklist: Vec<String>,
    /// Screen edge that opens the shelf when a drag rests against it.
    pub screen_edge: ScreenEdge,
}

/// Where a drag can be pushed against the screen edge to open the shelf.
/// Off unless the user turns it on.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ScreenEdge {
    Left,
    Right,
    Top,
    /// Also what an unknown value (from a newer version) reads as.
    #[default]
    #[serde(other)]
    Off,
}

pub const DEFAULT_HOTKEY: &str = "Win+Shift+H";

/// Apps where shaking never opens the shelf, out of the box: drawing, 3D and
/// CAD tools, where a held button and fast back-and-forth strokes are normal
/// input. (Full-screen apps such as games are always skipped.)
pub const DEFAULT_SHAKE_OFF_APPS: &[&str] = &[
    "blender.exe",
    "photoshop.exe",
    "illustrator.exe",
    "krita.exe",
    "mspaint.exe",
    "clipstudiopaint.exe",
    "acad.exe",
    "freecad.exe",
    "zwcad.exe",
    "sketchup.exe",
];

/// The allowlists earlier versions shipped as defaults. A config still
/// holding one of these is migrated to "any app" (see [`AppConfig::from_json`]).
const OLD_DEFAULT_ALLOWLISTS: &[&[&str]] = &[&["explorer.exe"], OLD_WIDE_ALLOWLIST];
const OLD_WIDE_ALLOWLIST: &[&str] = &[
    "explorer.exe",
    "dopus.exe",
    "totalcmd64.exe",
    "xyplorer.exe",
    "onecommander.exe",
    "msedge.exe",
    "chrome.exe",
    "firefox.exe",
    "brave.exe",
    "opera.exe",
    "vivaldi.exe",
    "outlook.exe",
    "olk.exe",
    "thunderbird.exe",
    "discord.exe",
    "slack.exe",
    "ms-teams.exe",
    "telegram.exe",
    "whatsapp.exe",
];

impl Default for MouseMonitorConfig {
    fn default() -> Self {
        Self {
            required_shakes: 3,
            shake_time_limit: 1500,
            shake_threshold: 100,
            window_close_delay: 3000,
            whitelist: Vec::new(),
            blocklist: DEFAULT_SHAKE_OFF_APPS.iter().map(|app| app.to_string()).collect(),
            screen_edge: ScreenEdge::Off,
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            mouse_monitor: MouseMonitorConfig::default(),
            autostart: true,
            // New installs get a hotkey; Windows itself leaves Win+Shift+H
            // free. Saved configs keep theirs, including an empty one.
            hotkey: DEFAULT_HOTKEY.to_string(),
            analytics_enabled: false,
            analytics_uuid: Uuid::new_v4().to_string(),
            mica: true,
        }
    }
}

impl AppConfig {
    /// Keeps the shake settings in usable ranges, whatever the file or the
    /// settings page sent (0 required shakes would open the shelf on every
    /// held click).
    pub fn clamped(mut self) -> Self {
        let m = &mut self.mouse_monitor;
        m.required_shakes = m.required_shakes.clamp(1, 20);
        m.shake_threshold = m.shake_threshold.clamp(20, 1000);
        m.shake_time_limit = m.shake_time_limit.clamp(300, 10_000);
        m.window_close_delay = m.window_close_delay.clamp(500, 30_000);
        self
    }

    /// Parses config.json contents. Missing fields take their defaults; an
    /// empty analytics id (written by very old builds) gets a fresh one.
    fn from_json(contents: &str) -> Result<Self, serde_json::Error> {
        let mut config = serde_json::from_str::<AppConfig>(contents)?.clamped();
        // An allowlist that is still exactly an old default was never chosen
        // by the user: switch it to "any app except the exclusion list".
        let allow = &config.mouse_monitor.whitelist;
        if OLD_DEFAULT_ALLOWLISTS.iter().any(|old| {
            allow.len() == old.len() && allow.iter().zip(old.iter()).all(|(a, b)| a.eq_ignore_ascii_case(b))
        }) {
            config.mouse_monitor.whitelist.clear();
            info!("Shake now works in any app (old default allowlist removed)");
        }
        if config.analytics_uuid.is_empty() {
            config.analytics_uuid = Uuid::new_v4().to_string();
            info!("Generated new analytics UUID while loading config");
        }
        Ok(config)
    }

    pub fn load(app_handle: &AppHandle) -> Self {
        let config_path = match Self::get_config_path(app_handle) {
            Ok(path) => path,
            Err(e) => {
                error!("Failed to get config path: {}", e);
                warn!("Using default config due to config path error");
                return Self::default();
            }
        };

        info!("Loading config from {:?}", config_path);

        let contents = match fs::read_to_string(&config_path) {
            Ok(contents) => contents,
            Err(e) => {
                warn!("Config file not found or unreadable: {}", e);
                return Self::default();
            }
        };

        match Self::from_json(&contents) {
            Ok(config) => config,
            Err(e) => {
                error!("Failed to parse config file: {}", e);
                // The next save overwrites config.json, so keep the unreadable
                // file for recovery instead of losing the user's settings.
                let backup = config_path.with_extension("json.bak");
                match fs::copy(&config_path, &backup) {
                    Ok(_) => warn!("Backed up unreadable config to {:?}", backup),
                    Err(e) => error!("Failed to back up unreadable config: {}", e),
                }
                warn!("Using default config due to config parse error");
                Self::default()
            }
        }
    }

    pub fn save(&self, app_handle: &AppHandle) -> Result<(), String> {
        let config_path = Self::get_config_path(app_handle)?;
        info!("Saving config to {:?}", config_path);
        self.write_to(&config_path)?;
        info!("Config saved successfully");
        Ok(())
    }

    /// Writes to a temporary file and renames it over `path`, so a crash or
    /// full disk mid-write never leaves a truncated config behind.
    fn write_to(&self, path: impl AsRef<Path>) -> Result<(), String> {
        let path = path.as_ref();
        let config_dir = path.parent().ok_or("Invalid config path")?;
        fs::create_dir_all(config_dir)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;

        let contents = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;
        let tmp_path = path.with_extension("json.tmp");
        fs::write(&tmp_path, contents)
            .map_err(|e| format!("Failed to write config file: {}", e))?;
        fs::rename(&tmp_path, path).map_err(|e| {
            let _ = fs::remove_file(&tmp_path);
            format!("Failed to replace config file: {}", e)
        })
    }

    pub fn get_config_path(app_handle: &AppHandle) -> Result<PathBuf, String> {
        let app_dir = app_handle
            .path()
            .app_config_dir()
            .map_err(|e| format!("Failed to get app config directory: {}", e))?;
        let config_path = app_dir.join("config.json");
        Ok(config_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_screen_edge_reads_as_off() {
        let config = AppConfig::from_json(r#"{"mouse_monitor":{"screen_edge":"bottom"}}"#)
            .expect("parse");
        assert_eq!(config.mouse_monitor.screen_edge, ScreenEdge::Off);
        let config = AppConfig::from_json(r#"{"mouse_monitor":{"screen_edge":"right"}}"#)
            .expect("parse");
        assert_eq!(config.mouse_monitor.screen_edge, ScreenEdge::Right);
    }

    #[test]
    fn missing_fields_keep_the_rest_of_the_config() {
        // A config from a build without analytics or whitelist support.
        let config = AppConfig::from_json(
            r#"{
                "mouse_monitor": {
                    "required_shakes": 7,
                    "shake_time_limit": 900,
                    "shake_threshold": 50,
                    "window_close_delay": 1000
                },
                "autostart": true,
                "hotkey": "Ctrl+Shift+H"
            }"#,
        )
        .expect("partial config should parse");

        assert_eq!(config.hotkey, "Ctrl+Shift+H");
        assert!(config.autostart);
        assert_eq!(config.mouse_monitor.required_shakes, 7);
        assert!(config.mouse_monitor.whitelist.is_empty());
        assert_eq!(config.mouse_monitor.blocklist, DEFAULT_SHAKE_OFF_APPS);
        assert!(!config.analytics_enabled);
        assert!(!config.analytics_uuid.is_empty());
    }

    #[test]
    fn missing_mouse_monitor_fields_use_defaults() {
        let config = AppConfig::from_json(r#"{"hotkey": "Alt+Q", "mouse_monitor": {}}"#)
            .expect("partial config should parse");
        assert_eq!(config.hotkey, "Alt+Q");
        assert_eq!(config.mouse_monitor.required_shakes, 3);
        assert_eq!(config.mouse_monitor.window_close_delay, 3000);
    }

    #[test]
    fn empty_analytics_uuid_is_regenerated() {
        let config = AppConfig::from_json(r#"{"analytics_uuid": ""}"#).unwrap();
        assert!(Uuid::parse_str(&config.analytics_uuid).is_ok());
    }

    #[test]
    fn malformed_json_is_an_error() {
        assert!(AppConfig::from_json("{ not json").is_err());
    }

    #[test]
    fn write_replaces_existing_file_and_leaves_no_temp_file() {
        let dir = std::env::temp_dir().join(format!("holdem-config-test-{}", Uuid::new_v4()));
        let path = dir.join("config.json");

        let mut config = AppConfig {
            hotkey: "Ctrl+1".to_string(),
            ..Default::default()
        };
        config.write_to(&path).unwrap();
        config.hotkey = "Ctrl+2".to_string();
        config.write_to(&path).unwrap();

        let saved = AppConfig::from_json(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(saved.hotkey, "Ctrl+2");
        assert!(!path.with_extension("json.tmp").exists());
        fs::remove_dir_all(dir).unwrap();
    }
}

#[cfg(test)]
mod migration_tests {
    use super::*;

    #[test]
    fn old_default_allowlists_become_any_app() {
        let old = r#"{"mouse_monitor": {"whitelist": ["explorer.exe"]}}"#;
        assert!(AppConfig::from_json(old).unwrap().mouse_monitor.whitelist.is_empty());
        let custom = r#"{"mouse_monitor": {"whitelist": ["explorer.exe", "dopus.exe"]}}"#;
        assert_eq!(
            AppConfig::from_json(custom).unwrap().mouse_monitor.whitelist,
            vec!["explorer.exe", "dopus.exe"]
        );
    }
}

#[cfg(test)]
mod clamp_tests {
    use super::*;

    #[test]
    fn out_of_range_shake_settings_are_clamped() {
        let config = AppConfig::from_json(
            r#"{"mouse_monitor": {"required_shakes": 0, "shake_threshold": 5, "shake_time_limit": 1, "window_close_delay": 0}}"#,
        )
        .unwrap();
        let m = config.mouse_monitor;
        assert_eq!((m.required_shakes, m.shake_threshold, m.shake_time_limit, m.window_close_delay), (1, 20, 300, 500));
    }
}
