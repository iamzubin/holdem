mod win;

use crate::config::MouseMonitorConfig;
use crate::DragState;
use std::sync::Arc;
use tauri::AppHandle;

pub fn start_mouse_monitor(
    config: MouseMonitorConfig,
    app_handle: AppHandle,
    drag_state: Arc<DragState>,
) {
    win::start_mouse_monitor(config, app_handle, drag_state);
}
