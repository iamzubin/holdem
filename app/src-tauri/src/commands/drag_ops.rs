use tauri::{AppHandle, State, Manager};
use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_CONTROL, VK_SHIFT};
use crate::FileList;

// Helper function to check if control or shift is pressed
fn is_move_key_pressed() -> bool {
    unsafe {
        let ctrl_pressed = GetAsyncKeyState(VK_CONTROL.0 as i32) < 0;
        let shift_pressed = GetAsyncKeyState(VK_SHIFT.0 as i32) < 0;
        ctrl_pressed || shift_pressed
    }
}

#[tauri::command]
pub fn start_drag(app: AppHandle, file_path: String) -> Result<(), String> {
    let item = match std::fs::canonicalize(file_path.clone()) {
        Ok(path) => {
            if !path.exists() {
                return Err(format!("File not found: {}", file_path));
            }
            drag::DragItem::Files(vec![path])
        }
        Err(e) => {
            return Err(format!("Error finding file: {} ({})", file_path, e));
        }
    };

    let window = app.get_webview_window("main").unwrap().clone();
    let _app_clone = app.clone();
    let file_path_clone = file_path.clone();
    
    // Define the on_drop_callback function
    let on_drop_callback = move |result: drag::DragResult, _: drag::CursorPosition| {
        // Check if control or shift is pressed at drop time
        let is_move = is_move_key_pressed();
        
        // If this was a move operation and the drop was successful, delete the file
        if is_move && !matches!(result, drag::DragResult::Cancel) {
            if let Ok(path) = std::fs::canonicalize(&file_path_clone) {
                if path.exists() {
                    if path.is_dir() {
                        let _ = std::fs::remove_dir_all(&path);
                    } else {
                        let _ = std::fs::remove_file(&path);
                    }
                }
            }
        }
    };

    // Start the drag operation
    drag::start_drag(
        &(window.hwnd().unwrap().0 as isize),
        item,
        on_drop_callback,
        drag::Options::default(),
    )
    .expect("Failed to start drag operation");

    Ok(())
}

#[tauri::command]
pub fn start_multi_drag(
    app: AppHandle,
    _file_list: State<'_, FileList>,
    file_paths: Vec<String>,
) -> Result<(), String> {
    println!(
        "Starting multi-file drag for files: {}",
        file_paths.join(", ")
    );

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
    let file_paths_clone = file_paths.clone();

    let on_drop_callback = move |result: drag::DragResult, _: drag::CursorPosition| {
        // check if the file is dropped on the app window
        if matches!(result, drag::DragResult::Cancel) {
            return;
        }

        // Check if control or shift is pressed at drop time
        let is_move = is_move_key_pressed();

        // If this was a move operation and the drop was successful, delete the files
        if is_move {
            for path_str in &file_paths_clone {
                if let Ok(path) = std::fs::canonicalize(path_str) {
                    if path.exists() {
                        if path.is_dir() {
                            let _ = std::fs::remove_dir_all(&path);
                        } else {
                            let _ = std::fs::remove_file(&path);
                        }
                    }
                }
            }
        }

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
        &(window.hwnd().unwrap().0 as isize),
        item,
        on_drop_callback,
        drag::Options::default(),
    )
    .map_err(|e| format!("Failed to start multi-file drag operation: {}", e))?;

    Ok(())
} 