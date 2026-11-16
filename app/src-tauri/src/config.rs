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
    /// The app version that was running when the user last closed the
    /// "what's new" window. Configs from 3.2.0 and earlier don't have it,
    /// which is how an upgrade is told apart from a fresh install: a missing
    /// field reads as `None`, while a fresh install starts from `Default`.
    ///
    /// The field-level `#[serde(default)]` is what makes a missing field
    /// `None`. Without it, the struct-level default would fill in
    /// `Default`'s current version and no upgrader would see the window.
    #[serde(default)]
    pub last_seen_version: Option<String>,
}

/// The running version, from Cargo.toml.
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// The release the "what's new" window describes. Users whose
/// `last_seen_version` is older (or missing) see the window once. Raise it
/// together with the window's text when a later release needs one.
pub const WHATS_NEW_RELEASE: &str = "3.2.0";

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
            // A fresh install, or a config that failed to load, has nothing
            // to compare with, so it counts as already seen.
            last_seen_version: Some(APP_VERSION.to_string()),
        }
    }
}

/// `major.minor.patch` as numbers; missing or non-numeric parts read as 0,
/// and anything after the digits ("3.2.0-beta") is ignored.
fn version_key(version: &str) -> [u64; 3] {
    let mut key = [0; 3];
    for (slot, part) in key.iter_mut().zip(version.trim().split('.')) {
        let digits: String = part.chars().take_while(char::is_ascii_digit).collect();
        *slot = digits.parse().unwrap_or(0);
    }
    key
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

    /// Settings sends back a copy of the config it loaded earlier. Fields the
    /// app keeps for itself, rather than settings, come from `current`, so
    /// that copy can't undo them.
    pub fn keeping_app_state(mut self, current: &AppConfig) -> Self {
        self.last_seen_version = current.last_seen_version.clone();
        self
    }

    /// True when the user updated from a version older than
    /// [`WHATS_NEW_RELEASE`] and hasn't closed the "what's new" window since.
    pub fn needs_whats_new(&self) -> bool {
        match &self.last_seen_version {
            None => true,
            Some(seen) => version_key(seen) < version_key(WHATS_NEW_RELEASE),
        }
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
        Self::load_from(&config_path)
    }

    /// Reads `config_path`. A missing, unreadable or unparseable file gives
    /// `Default` (and an unparseable one is backed up first).
    fn load_from(config_path: &Path) -> Self {
        info!("Loading config from {:?}", config_path);

        let contents = match fs::read_to_string(config_path) {
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
                match fs::copy(config_path, &backup) {
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
    fn removed_screen_edge_setting_is_ignored() {
        // 3.2.0 had a screen-edge trigger; configs that still carry it load.
        let config = AppConfig::from_json(
            r#"{"hotkey": "Alt+Q", "mouse_monitor": {"required_shakes": 4, "screen_edge": "right"}}"#,
        )
        .expect("parse");
        assert_eq!(config.hotkey, "Alt+Q");
        assert_eq!(config.mouse_monitor.required_shakes, 4);
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
mod whats_new_tests {
    use super::*;

    #[test]
    fn config_from_before_the_field_is_an_upgrade() {
        // A 3.1.x config: the field doesn't exist yet.
        let config = AppConfig::from_json(r#"{"hotkey": "Win+Shift+H", "autostart": true}"#).unwrap();
        assert_eq!(config.last_seen_version, None);
        assert!(config.needs_whats_new());
    }

    #[test]
    fn fresh_install_and_failed_load_skip_it() {
        // `load` falls back to `Default` for a missing or unreadable file.
        assert!(!AppConfig::default().needs_whats_new());
    }

    #[test]
    fn shown_only_for_versions_older_than_the_release() {
        let seen = |v: &str| AppConfig {
            last_seen_version: Some(v.to_string()),
            ..Default::default()
        };
        assert!(seen("3.1.1").needs_whats_new());
        assert!(seen("2.9.10").needs_whats_new());
        assert!(!seen(WHATS_NEW_RELEASE).needs_whats_new());
        assert!(!seen("3.2.1").needs_whats_new());
        assert!(!seen("3.10.0").needs_whats_new());
    }

    #[test]
    fn seen_version_survives_a_save() {
        let config = AppConfig {
            last_seen_version: Some("3.2.0".to_string()),
            ..Default::default()
        };
        let json = serde_json::to_string(&config).unwrap();
        assert_eq!(AppConfig::from_json(&json).unwrap().last_seen_version.as_deref(), Some("3.2.0"));
    }

    #[test]
    fn settings_copy_cannot_reset_the_seen_version() {
        // Settings loaded its copy before the window was closed.
        let stale = AppConfig { last_seen_version: None, mica: false, ..Default::default() };
        let current = AppConfig { last_seen_version: Some("3.2.1".to_string()), ..Default::default() };
        let saved = stale.keeping_app_state(&current);
        assert_eq!(saved.last_seen_version.as_deref(), Some("3.2.1"));
        assert!(!saved.mica, "the user's settings still come from the copy");
    }

    #[test]
    fn missing_config_file_skips_the_window() {
        let path = std::env::temp_dir().join(format!("holdem-absent-{}.json", Uuid::new_v4()));
        assert!(!AppConfig::load_from(&path).needs_whats_new());
    }

    #[test]
    fn null_seen_version_is_an_upgrade() {
        let config = AppConfig::from_json(r#"{"last_seen_version": null}"#).unwrap();
        assert!(config.needs_whats_new());
    }

    #[test]
    fn unparseable_config_file_skips_the_window() {
        let dir = std::env::temp_dir().join(format!("holdem-config-test-{}", Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("config.json");
        fs::write(&path, "{ not json").unwrap();
        let config = AppConfig::load_from(&path);
        assert!(!config.needs_whats_new());
        assert!(path.with_extension("json.bak").exists());
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn version_key_reads_partial_and_suffixed_versions() {
        assert_eq!(version_key("3.2"), [3, 2, 0]);
        assert_eq!(version_key("3.2.0-beta.1"), [3, 2, 0]);
        assert_eq!(version_key(""), [0, 0, 0]);
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
