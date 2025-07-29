use std::sync::{Arc, Mutex};
use posthog_rs::{client, Event as PostHogEvent, Client};
use tauri::{AppHandle, Manager};

pub struct AnalyticsService {
    pub client: Option<Arc<Client>>,
    pub enabled: bool,
    pub uuid: String,
}

impl AnalyticsService {
    pub fn new() -> Self {
        Self {
            client: None,
            enabled: false,
            uuid: String::new(),
        }
    }

    pub async fn initialize(&mut self, analytics_enabled: bool, uuid: String) -> Result<(), String> {
        self.enabled = analytics_enabled;
        self.uuid = uuid;

        if !self.enabled {
            println!("[Analytics] Analytics disabled, skipping initialization");
            return Ok(());
        }

        // Use compile-time environment variable
        let posthog_key = env!("POSTHOG_KEY", "POSTHOG_KEY not set at compile time");
        println!("[Analytics] Initializing PostHog client...");
        let client = client(posthog_key).await;
        self.client = Some(Arc::new(client));
        println!("[Analytics] PostHog client initialized successfully");
        Ok(())
    }

    pub async fn send_event(&self, event_name: &str, properties: Option<Vec<(&str, serde_json::Value)>>) -> Result<(), String> {
        if !self.enabled {
            return Ok(());
        }

        if let Some(client) = &self.client {
            let mut event = PostHogEvent::new(event_name, &self.uuid);
            
            if let Some(props) = properties {
                for (key, value) in props {
                    let _ = event.insert_prop(key, value);
                }
            }

            match client.capture(event).await {
                Ok(_) => {
                    println!("[Analytics] Event '{}' sent successfully", event_name);
                    Ok(())
                }
                Err(e) => {
                    eprintln!("[Analytics] Error sending event '{}': {:?}", event_name, e);
                    Err(format!("Failed to send event: {}", e))
                }
            }
        } else {
            Err("Analytics client not initialized".to_string())
        }
    }

    pub async fn send_app_started(&self) -> Result<(), String> {
        self.send_event("app_started", None).await
    }

    pub async fn send_consent_accepted(&self) -> Result<(), String> {
        self.send_event("consent_accepted", None).await
    }

    pub async fn send_consent_declined(&self) -> Result<(), String> {
        self.send_event("consent_declined", None).await
    }

    pub async fn send_files_dropped(&self, num_files: usize) -> Result<(), String> {
        let properties = vec![
            ("num_files", serde_json::Value::Number((num_files as i64).into())),
        ];
        self.send_event("files_dropped", Some(properties)).await
    }

    pub async fn send_window_opened(&self, window_type: &str) -> Result<(), String> {
        let properties = vec![
            ("window_type", serde_json::Value::String(window_type.to_string())),
        ];
        self.send_event("window_opened", Some(properties)).await
    }

    pub async fn send_hotkey_registered(&self, hotkey: &str) -> Result<(), String> {
        let properties = vec![
            ("hotkey", serde_json::Value::String(hotkey.to_string())),
        ];
        self.send_event("hotkey_registered", Some(properties)).await
    }

    pub async fn send_autostart_toggled(&self, enabled: bool) -> Result<(), String> {
        let properties = vec![
            ("enabled", serde_json::Value::Bool(enabled)),
        ];
        self.send_event("autostart_toggled", Some(properties)).await
    }

    pub async fn send_settings_opened(&self) -> Result<(), String> {
        self.send_event("settings_opened", None).await
    }

    pub async fn send_mouse_shake_detected(&self, shake_count: u32) -> Result<(), String> {
        let properties = vec![
            ("shake_count", serde_json::Value::Number((shake_count as i64).into())),
        ];
        self.send_event("mouse_shake_detected", Some(properties)).await
    }

    pub async fn send_file_renamed(&self, old_name: &str, new_name: &str) -> Result<(), String> {
        let properties = vec![
            ("old_name", serde_json::Value::String(old_name.to_string())),
            ("new_name", serde_json::Value::String(new_name.to_string())),
        ];
        self.send_event("file_renamed", Some(properties)).await
    }

    pub async fn send_file_removed(&self, file_name: &str) -> Result<(), String> {
        let properties = vec![
            ("file_name", serde_json::Value::String(file_name.to_string())),
        ];
        self.send_event("file_removed", Some(properties)).await
    }

    pub async fn send_files_cleared(&self, num_files: usize) -> Result<(), String> {
        let properties = vec![
            ("num_files", serde_json::Value::Number((num_files as i64).into())),
        ];
        self.send_event("files_cleared", Some(properties)).await
    }

    pub async fn send_app_restarted(&self) -> Result<(), String> {
        self.send_event("app_restarted", None).await
    }

    pub async fn send_update_checked(&self, update_available: bool) -> Result<(), String> {
        let properties = vec![
            ("update_available", serde_json::Value::Bool(update_available)),
        ];
        self.send_event("update_checked", Some(properties)).await
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

// Global analytics service instance
pub type AnalyticsState = Arc<Mutex<AnalyticsService>>;

// Helper function to get analytics service from app state
pub fn get_analytics_service(app_handle: &AppHandle) -> Result<AnalyticsState, String> {
    app_handle.try_state::<AnalyticsState>().map(|state| state.inner().clone()).ok_or_else(|| "Analytics service not found".to_string())
}

// Helper function to send event using app handle
pub async fn send_analytics_event(
    app_handle: &AppHandle, 
    event_name: &str, 
    properties: Option<Vec<(&str, serde_json::Value)>>
) -> Result<(), String> {
    let analytics_service = get_analytics_service(app_handle)?;
    // Extract all needed data before await
    let (enabled, uuid, client) = {
        let service = analytics_service.lock().map_err(|e| format!("Failed to lock analytics service: {}", e))?;
        (service.enabled, service.uuid.clone(), service.client.clone())
    };
    if !enabled {
        return Ok(());
    }
    if let Some(client) = client {
        let mut event = PostHogEvent::new(event_name, &uuid);
        if let Some(props) = properties {
            for (key, value) in props {
                let _ = event.insert_prop(key, value);
            }
        }
        match client.capture(event).await {
            Ok(_) => {
                println!("[Analytics] Event '{}' sent successfully", event_name);
                Ok(())
            }
            Err(e) => {
                eprintln!("[Analytics] Error sending event '{}': {:?}", event_name, e);
                Err(format!("Failed to send event: {}", e))
            }
        }
    } else {
        Err("Analytics client not initialized".to_string())
    }
}

// Convenience functions for common analytics events
pub async fn send_window_opened_event(app_handle: &AppHandle, window_type: &str) -> Result<(), String> {
    send_analytics_event(app_handle, "window_opened", Some(vec![
        ("window_type", serde_json::Value::String(window_type.to_string())),
    ])).await
}

pub async fn send_popup_window_opened_event(app_handle: &AppHandle) -> Result<(), String> {
    send_analytics_event(app_handle, "popup_window_opened", None).await
}

pub async fn send_hotkey_registered_event(app_handle: &AppHandle, hotkey: &str) -> Result<(), String> {
    send_analytics_event(app_handle, "hotkey_registered", Some(vec![
        ("hotkey", serde_json::Value::String(hotkey.to_string())),
    ])).await
}

pub async fn send_autostart_toggled_event(app_handle: &AppHandle, enabled: bool) -> Result<(), String> {
    send_analytics_event(app_handle, "autostart_toggled", Some(vec![
        ("enabled", serde_json::Value::Bool(enabled)),
    ])).await
}

pub async fn send_settings_opened_event(app_handle: &AppHandle) -> Result<(), String> {
    send_analytics_event(app_handle, "settings_opened", None).await
}

pub async fn send_mouse_shake_detected_event(app_handle: &AppHandle, shake_count: u32) -> Result<(), String> {
    send_analytics_event(app_handle, "mouse_shake_detected", Some(vec![
        ("shake_count", serde_json::Value::Number((shake_count as i64).into())),
    ])).await
}

pub async fn send_file_renamed_event(app_handle: &AppHandle, old_name: &str, new_name: &str) -> Result<(), String> {
    send_analytics_event(app_handle, "file_renamed", Some(vec![
        ("old_name", serde_json::Value::String(old_name.to_string())),
        ("new_name", serde_json::Value::String(new_name.to_string())),
    ])).await
}

pub async fn send_file_removed_event(app_handle: &AppHandle, file_name: &str) -> Result<(), String> {
    send_analytics_event(app_handle, "file_removed", Some(vec![
        ("file_name", serde_json::Value::String(file_name.to_string())),
    ])).await
}

pub async fn send_files_cleared_event(app_handle: &AppHandle, num_files: usize) -> Result<(), String> {
    send_analytics_event(app_handle, "files_cleared", Some(vec![
        ("num_files", serde_json::Value::Number((num_files as i64).into())),
    ])).await
}

pub async fn send_app_restarted_event(app_handle: &AppHandle) -> Result<(), String> {
    send_analytics_event(app_handle, "app_restarted", None).await
}

pub async fn send_update_checked_event(app_handle: &AppHandle, update_available: bool) -> Result<(), String> {
    send_analytics_event(app_handle, "update_checked", Some(vec![
        ("update_available", serde_json::Value::Bool(update_available)),
    ])).await
}

pub async fn send_consent_declined_event(app_handle: &AppHandle) -> Result<(), String> {
    send_analytics_event(app_handle, "consent_declined", None).await
}