mod common;

#[cfg(target_os = "windows")]
mod win;

#[cfg(target_os = "macos")]
mod mac;

use crate::config::MouseMonitorConfig;
use crate::DragState;
use std::sync::Arc;
use tauri::AppHandle;

pub fn start_mouse_monitor(config: MouseMonitorConfig, app_handle: AppHandle, drag_state: Arc<DragState>) {
    #[cfg(target_os = "windows")]
    {
        win::start_mouse_monitor(config, app_handle, drag_state);
    }

    #[cfg(target_os = "macos")]
    {
        mac::start_mouse_monitor(config, app_handle, drag_state);
    }
}
