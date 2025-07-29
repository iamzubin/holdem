use std::sync::{Arc, Mutex};
use tauri::{AppHandle, State, Manager};
use crate::config::AppConfig;
use crate::file::FileMetadata;
use crate::analytics;
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
    _config: State<'_, Arc<Mutex<AppConfig>>>,
    file_paths: Vec<String>,
) -> Result<(), String> {
    println!(
        "Starting multi-file drag for files: {}",
        file_paths.join(", ")
    );

    // Send analytics event asynchronously using centralized service
    let num_files = file_paths.len();
    let app_handle = app.clone();
    
    tauri::async_runtime::spawn(async move {
        if let Err(e) = analytics::send_analytics_event(&app_handle, "files_dropped", Some(vec![
            ("num_files", serde_json::Value::Number((num_files as i64).into())),
        ])).await {
            eprintln!("[Analytics] Failed to send files_dropped event: {}", e);
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

    let window = app.get_webview_window("main")
        .ok_or("Main window not found")?;
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
            if let Err(e) = super::window_ops::close_popup_window(app_clone.clone()) {
                println!("Failed to close popup window: {}", e);
            }
        }
        if let Some(main_window) = app.get_webview_window("main") {
            if let Err(e) = main_window.hide() {
                println!("Failed to hide main window: {}", e);
            }
        }
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
