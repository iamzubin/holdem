use std::sync::{Arc, Mutex};
use tauri::{AppHandle, State, Manager};
use crate::config::AppConfig;
use crate::file::FileMetadata;
use dotenv::dotenv;
use posthog_rs::{Event as PostHogEvent};
use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_CONTROL, VK_SHIFT};

type FileList = Arc<Mutex<Vec<FileMetadata>>>;

// Helper function to check if control or shift is pressed
fn is_move_key_pressed() -> bool {
    unsafe {
        let ctrl_pressed = GetAsyncKeyState(VK_CONTROL.0 as i32) < 0;
        let shift_pressed: bool = GetAsyncKeyState(VK_SHIFT.0 as i32) < 0;
        ctrl_pressed || shift_pressed
    }
}

#[tauri::command]
pub fn start_multi_drag(
    app: AppHandle,
    _file_list: State<'_, FileList>,
    config: State<'_, Arc<Mutex<AppConfig>>>,
    file_paths: Vec<String>,
) -> Result<(), String> {
    println!(
        "Starting multi-file drag for files: {}",
        file_paths.join(", ")
    );

    // Send PostHog event asynchronously
    let num_files = file_paths.len();
    let config = config.lock().unwrap();
    let uuid = config.analytics_uuid.clone();
    drop(config); // Release lock before async block
    
    tauri::async_runtime::spawn(async move {
        dotenv().ok();
        match std::env::var("POSTHOG_KEY") {
            Ok(posthog_key) => {
                println!("[PostHog] Sending files dropped event ({} files)...", num_files);
                println!("[PostHog] Key loaded: {}...", &posthog_key[..6]);
                let client = posthog_rs::client(posthog_key.as_str()).await;
                let mut event = PostHogEvent::new("files dropped", &uuid);
                let _ = event.insert_prop("num_files", num_files as i64);
                let res = client.capture(event).await;
                match res {
                    Ok(_) => println!("[PostHog] files dropped event sent!"),
                    Err(e) => println!("[PostHog] Error sending files dropped event: {:?}", e),
                }
            }
            Err(_) => {
                println!("[PostHog] POSTHOG_KEY not set in environment");
            }
        }
    });

    let mut valid_paths = Vec::new();

    for file_path in &file_paths {
        match std::fs::canonicalize(file_path.clone()) {
            Ok(path) => {
                if path.exists() {
                    valid_paths.push(path);
                }
            }
            Err(e) => {
                println!("Error processing file: {}", e);
            }
        }
    }

    if valid_paths.is_empty() {
        return Err("No valid files to drag".to_string());
    }

    let item = drag::DragItem::Files(valid_paths);

    let window = app.get_webview_window("main").unwrap();
    let app_clone = app.clone();
    let is_move = is_move_key_pressed();

    let on_drop_callback = move |result: drag::DragResult, _: drag::CursorPosition| {
        // check if the file is dropped on the app window
        if matches!(result, drag::DragResult::Cancel) {
            return;
        }

        // Check if control or shift is pressed at drop time

        // check if the popup window is open
        if app_clone.get_webview_window("popup").is_some() {
            super::window_ops::close_popup_window(app_clone.clone()).unwrap();
        }
        let _ = app
            .get_webview_window("main")
            .unwrap()
            .hide()
            .map_err(|e| e.to_string());
    };

    drag::start_drag(
        &window,
        item,
        /* drag::Image */
        drag::Image::Raw(Vec::new()),
        on_drop_callback,
        drag::Options {
            skip_animatation_on_cancel_or_failure: true,
            mode: if is_move {drag::DragMode::Move} else {drag::DragMode::Copy},
        },
    )
    .map_err(|e| format!("Failed to start multi-file drag operation: {}", e))?;

    Ok(())
}
