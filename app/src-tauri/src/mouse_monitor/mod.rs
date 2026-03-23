mod common;

#[cfg(target_os = "windows")]
mod win;

#[cfg(target_os = "macos")]
mod mac;

use crate::config::MouseMonitorConfig;
use tauri::AppHandle;

pub fn start_mouse_monitor(config: MouseMonitorConfig, app_handle: AppHandle) {
    #[cfg(target_os = "windows")]
    {
        win::start_mouse_monitor(config, app_handle);
    }

    #[cfg(target_os = "macos")]
    {
        mac::start_mouse_monitor(config, app_handle);
    }
}
