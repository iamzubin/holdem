#[cfg(target_os = "windows")]
use tauri::{Manager, WebviewWindow};

#[cfg(target_os = "windows")]
#[derive(Debug, Clone)]
pub struct ScreenBounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[cfg(target_os = "windows")]
impl ScreenBounds {
    pub fn from_window(window: &WebviewWindow) -> Result<Self, Box<dyn std::error::Error>> {
        if let Some(screen) = window.current_monitor()? {
            Ok(ScreenBounds {
                x: screen.position().x as f64,
                y: screen.position().y as f64,
                width: screen.size().width as f64,
                height: screen.size().height as f64,
            })
        } else if let Some(screen) = window.primary_monitor()? {
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

    pub fn constrain_position(&self, x: f64, y: f64, margin: f64) -> (f64, f64) {
        let constrained_x = x.max(self.x + margin).min(self.x + self.width - margin);
        let constrained_y = y.max(self.y + margin).min(self.y + self.height - margin);
        (constrained_x, constrained_y)
    }
}
