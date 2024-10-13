// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// only compile the mouse monitor if it's windows
#[cfg(target_os = "windows")]
#[path = "windowsutils/mouse_monitor.rs"]
// change this to the path of the module you want to use
#[cfg(target_os = "macos")]
#[path = "macutils/mouse_monitor.rs"]
mod mouse_monitor;

fn main() {
    holdem_lib::run()
}
