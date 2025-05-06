use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::AppHandle;
use tauri::Manager;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub mouse_monitor: MouseMonitorConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MouseMonitorConfig {
    pub required_shakes: u32,
    pub shake_time_limit: u64,
    pub shake_threshold: i32,
    pub window_close_delay: u64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            mouse_monitor: MouseMonitorConfig {
                required_shakes: 5,
                shake_time_limit: 1500,
                shake_threshold: 100,
                window_close_delay: 3000,
            },
        }
    }
}

impl AppConfig {
    pub fn load(app_handle: &AppHandle) -> Self {
        let config_path = Self::get_config_path(app_handle);
        
        if let Ok(contents) = fs::read_to_string(&config_path) {
            if let Ok(config) = serde_json::from_str(&contents) {
                return config;
            }
        }
        
        // If loading fails, return default config
        Self::default()
    }

    pub fn save(&self, app_handle: &AppHandle) -> Result<(), String> {
        let config_path = Self::get_config_path(app_handle);
        let config_dir = config_path.parent().ok_or("Invalid config path")?;
        
        // Create config directory if it doesn't exist
        if !config_dir.exists() {
            fs::create_dir_all(config_dir).map_err(|e| e.to_string())?;
        }
        
        let contents = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(&config_path, contents).map_err(|e| e.to_string())?;
        
        Ok(())
    }

    fn get_config_path(app_handle: &AppHandle) -> PathBuf {
        let app_dir = app_handle.path().app_config_dir().unwrap();
        app_dir.join("config.json")
    }
} 