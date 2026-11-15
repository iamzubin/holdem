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
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct MouseMonitorConfig {
    pub required_shakes: u32,
    pub shake_time_limit: u64,
    pub shake_threshold: i32,
    pub window_close_delay: u64,
    pub whitelist: Vec<String>,
}

impl Default for MouseMonitorConfig {
    fn default() -> Self {
        Self {
            required_shakes: 5,
            shake_time_limit: 1500,
            shake_threshold: 100,
            window_close_delay: 3000,
            whitelist: vec!["explorer.exe".to_string()],
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            mouse_monitor: MouseMonitorConfig::default(),
            autostart: false,
            hotkey: "".to_string(),
            analytics_enabled: false,
            analytics_uuid: Uuid::new_v4().to_string(),
        }
    }
}

impl AppConfig {
    /// Parses config.json contents. Missing fields take their defaults; an
    /// empty analytics id (written by very old builds) gets a fresh one.
    fn from_json(contents: &str) -> Result<Self, serde_json::Error> {
        let mut config = serde_json::from_str::<AppConfig>(contents)?;
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
        assert_eq!(config.mouse_monitor.whitelist, vec!["explorer.exe"]);
        assert!(!config.analytics_enabled);
        assert!(!config.analytics_uuid.is_empty());
    }

    #[test]
    fn missing_mouse_monitor_fields_use_defaults() {
        let config = AppConfig::from_json(r#"{"hotkey": "Alt+Q", "mouse_monitor": {}}"#)
            .expect("partial config should parse");
        assert_eq!(config.hotkey, "Alt+Q");
        assert_eq!(config.mouse_monitor.required_shakes, 5);
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
