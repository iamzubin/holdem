use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::AppHandle;
use tauri::Manager;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub mouse_monitor: MouseMonitorConfig,
    pub autostart: bool,
    pub hotkey: String,
    pub analytics_enabled: bool,
    pub analytics_uuid: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MouseMonitorConfig {
    pub required_shakes: u32,
    pub shake_time_limit: u64,
    pub shake_threshold: i32,
    pub window_close_delay: u64,
    #[serde(default = "default_whitelist")]
    pub whitelist: Vec<String>,
}

fn default_whitelist() -> Vec<String> {
    vec!["explorer.exe".to_string()]
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            mouse_monitor: MouseMonitorConfig {
                required_shakes: 5,
                shake_time_limit: 1500,
                shake_threshold: 100,
                window_close_delay: 3000,
                whitelist: default_whitelist(),
            },
            autostart: false,
            hotkey: "".to_string(),
            analytics_enabled: false,
            analytics_uuid: uuid::Uuid::new_v4().to_string(),
        }
    }
}

impl AppConfig {
    pub fn config_exists(app_handle: &AppHandle) -> bool {
        if let Ok(config_path) = Self::get_config_path(app_handle) {
            config_path.exists()
        } else {
            false
        }
    }

    pub fn load(app_handle: &AppHandle) -> Self {
        let config_path = match Self::get_config_path(app_handle) {
            Ok(path) => path,
            Err(e) => {
                eprintln!("[Config] Failed to get config path: {}", e);
                println!("[Config] Using default config due to path error");
                return Self::default();
            }
        };
        
        println!("Loading config from: {:?}", config_path);

        if let Ok(contents) = fs::read_to_string(&config_path) {
            match serde_json::from_str::<AppConfig>(&contents) {
                Ok(mut config) => {
                    // Check if analytics fields are missing (for backward compatibility)
                    if config.analytics_uuid.is_empty() {
                        config.analytics_uuid = Uuid::new_v4().to_string();
                        println!("[Config] Generated new analytics UUID: {}", config.analytics_uuid);
                    }
                    return config;
                }
                Err(e) => {
                    eprintln!("[Config] Failed to parse config file: {}", e);
                    println!("[Config] Using default config due to parse error");
                }
            }
        } else {
            println!("[Config] Config file not found or unreadable");
        }

        // If loading fails, return default config
        println!("[Config] Using default config (analytics_enabled: false)");
        Self::default()
    }

    pub fn save(&self, app_handle: &AppHandle) -> Result<(), String> {
        let config_path = Self::get_config_path(app_handle)?;
        println!("Saving config to: {:?}", config_path);

        let config_dir = config_path.parent().ok_or("Invalid config path")?;
        println!("Config directory: {:?}", config_dir);

        // Create config directory if it doesn't exist
        if !config_dir.exists() {
            println!("Creating config directory...");
            fs::create_dir_all(config_dir)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }

        let contents = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;
        fs::write(&config_path, contents)
            .map_err(|e| format!("Failed to write config file: {}", e))?;

        println!("Config saved successfully");
        Ok(())
    }

    fn get_config_path(app_handle: &AppHandle) -> Result<PathBuf, String> {
        let app_dir = app_handle.path().app_config_dir()
            .map_err(|e| format!("Failed to get app config directory: {}", e))?;
        let config_path = app_dir.join("config.json");
        println!("Config path: {:?}", config_path);
        Ok(config_path)
    }
}
