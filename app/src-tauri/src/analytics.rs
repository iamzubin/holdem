use posthog_rs::{client, Client, Event as PostHogEvent};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager};
use tracing::{error, info, warn};

#[derive(Clone)]
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

    pub async fn initialize(
        &mut self,
        analytics_enabled: bool,
        uuid: String,
    ) -> Result<(), String> {
        self.enabled = analytics_enabled;
        self.uuid = uuid;

        if !self.enabled {
            info!("Analytics disabled; skipping initialization");
            return Ok(());
        }

        // Use compile-time environment variable
        if let Some(posthog_key) = option_env!("POSTHOG_KEY") {
            info!("Initializing PostHog client");
            let client = client(posthog_key).await;
            self.client = Some(Arc::new(client));
            info!("PostHog client initialized successfully");
        } else {
            warn!("POSTHOG_KEY is not set; analytics will remain disabled");
        }
        Ok(())
    }

    pub async fn send_event(
        &self,
        event_name: &str,
        properties: Option<Vec<(&str, serde_json::Value)>>,
    ) -> Result<(), String> {
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
                    info!("Analytics event sent: {}", event_name);
                    Ok(())
                }
                Err(e) => {
                    error!("Failed to send analytics event '{}': {:?}", event_name, e);
                    Err(format!("Failed to send event: {}", e))
                }
            }
        } else {
            Err("Analytics client not initialized".to_string())
        }
    }

}

// Global analytics service instance
pub type AnalyticsState = Arc<Mutex<AnalyticsService>>;

// Helper function to get analytics service from app state
pub fn get_analytics_service(app_handle: &AppHandle) -> Result<AnalyticsState, String> {
    app_handle
        .try_state::<AnalyticsState>()
        .map(|state| state.inner().clone())
        .ok_or_else(|| "Analytics service not found".to_string())
}

// Helper function to send event using app handle
pub async fn send_analytics_event(
    app_handle: &AppHandle,
    event_name: &str,
    properties: Option<Vec<(&str, serde_json::Value)>>,
) -> Result<(), String> {
    let analytics_service = get_analytics_service(app_handle)?;
    let service = {
        let guard = analytics_service
            .lock()
            .map_err(|e| format!("Failed to lock analytics service: {}", e))?;
        guard.clone()
    };
    service.send_event(event_name, properties).await
}

// Callers use `send_analytics_event` directly with the event name and
// optional properties — no per-event wrappers.
