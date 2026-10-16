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
    /// Creates the service with the user's current consent. The PostHog
    /// client is created lazily (see [`ensure_client`]) and only once consent
    /// has been given.
    pub fn new(enabled: bool, uuid: String) -> Self {
        Self {
            client: None,
            enabled,
            uuid,
        }
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

            match client.capture_immediate(event).await {
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

/// Creates the PostHog client if analytics is enabled and no client exists
/// yet. Safe to call repeatedly; a no-op when disabled or already initialised.
pub async fn ensure_client(state: &AnalyticsState) -> Result<(), String> {
    let needs_client = {
        let guard = state
            .lock()
            .map_err(|e| format!("Failed to lock analytics service: {}", e))?;
        guard.enabled && guard.client.is_none()
    };
    if !needs_client {
        return Ok(());
    }

    let Some(posthog_key) = option_env!("POSTHOG_KEY") else {
        warn!("POSTHOG_KEY is not set; analytics will remain disabled");
        return Ok(());
    };

    info!("Initializing PostHog client");
    let new_client = Arc::new(client(posthog_key).await);

    let mut guard = state
        .lock()
        .map_err(|e| format!("Failed to lock analytics service: {}", e))?;
    if guard.client.is_none() {
        guard.client = Some(new_client);
        info!("PostHog client initialized successfully");
    }
    Ok(())
}

/// Applies the user's analytics consent to the running service immediately.
///
/// Disabling takes effect synchronously: every event dispatched after this
/// returns is dropped. Returns `true` when the enabled state actually changed.
pub fn set_analytics_enabled(app_handle: &AppHandle, enabled: bool) -> Result<bool, String> {
    let state = get_analytics_service(app_handle)?;
    let changed = {
        let mut guard = state
            .lock()
            .map_err(|e| format!("Failed to lock analytics service: {}", e))?;
        let changed = guard.enabled != enabled;
        guard.enabled = enabled;
        changed
    };

    if changed {
        info!(
            "Analytics {} at runtime",
            if enabled { "enabled" } else { "disabled" }
        );
    }

    Ok(changed)
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
        if !guard.enabled {
            return Ok(());
        }
        guard.clone()
    };
    service.send_event(event_name, properties).await
}

// Callers use `send_analytics_event` directly with the event name and
// optional properties — no per-event wrappers.
