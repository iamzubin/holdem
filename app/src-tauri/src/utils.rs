use tauri::{Window, Manager};

#[cfg(target_os = "macos")]
use objc2_app_kit::NSScreen;
#[cfg(target_os = "macos")]
use objc2_foundation::NSArray;

#[derive(Debug, Clone)]
pub struct ScreenBounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl ScreenBounds {
    #[cfg(target_os = "macos")]
    pub fn from_window(window: &tauri::WebviewWindow) -> Result<Self, Box<dyn std::error::Error>> {
        // Get the screen that contains the window
        if let Some(screen) = window.current_monitor()? {
            Ok(ScreenBounds {
                x: screen.position().x as f64,
                y: screen.position().y as f64,
                width: screen.size().width as f64,
                height: screen.size().height as f64,
            })
        } else {
            // Fallback to primary monitor
            if let Some(screen) = window.primary_monitor()? {
                Ok(ScreenBounds {
                    x: screen.position().x as f64,
                    y: screen.position().y as f64,
                    width: screen.size().width as f64,
                    height: screen.size().height as f64,
                })
            } else {
                Err("No monitor found".into())
            }
        }
    }

    #[cfg(target_os = "windows")]
    pub fn from_window(window: &tauri::WebviewWindow) -> Result<Self, Box<dyn std::error::Error>> {
        // Get the screen that contains the window
        if let Some(screen) = window.current_monitor()? {
            Ok(ScreenBounds {
                x: screen.position().x as f64,
                y: screen.position().y as f64,
                width: screen.size().width as f64,
                height: screen.size().height as f64,
            })
        } else {
            // Fallback to primary monitor
            if let Some(screen) = window.primary_monitor()? {
                Ok(ScreenBounds {
                    x: screen.position().x as f64,
                    y: screen.position().y as f64,
                    width: screen.size().width as f64,
                    height: screen.size().height as f64,
                })
            } else {
                Err("No monitor found".into())
            }
        }
    }

    pub fn constrain_position(&self, x: f64, y: f64, margin: f64) -> (f64, f64) {
        let constrained_x = x.max(self.x + margin).min(self.x + self.width - margin);
        let constrained_y = y.max(self.y + margin).min(self.y + self.height - margin);
        (constrained_x, constrained_y)
    }
}

#[cfg(target_os = "macos")]
pub fn get_screen_bounds_from_handle(app_handle: &tauri::AppHandle) -> Result<ScreenBounds, Box<dyn std::error::Error>> {
    // Get the main window
    if let Some(window) = app_handle.get_webview_window("main") {
        ScreenBounds::from_window(&window)
    } else {
        Err("Main window not found".into())
    }
}

#[cfg(target_os = "windows")]
pub fn get_screen_bounds_from_handle(app_handle: &tauri::AppHandle) -> Result<ScreenBounds, Box<dyn std::error::Error>> {
    // Get the main window
    if let Some(window) = app_handle.get_webview_window("main") {
        ScreenBounds::from_window(&window)
    } else {
        Err("Main window not found".into())
    }
}