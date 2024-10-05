use tauri::Manager;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[cfg(desktop)]
mod tray;

#[tauri::command]
fn start_drag(app: tauri::AppHandle, file_path: String) -> Result<(), String> {
    println!("Starting drag for file: {}", file_path);
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

    let window = app.get_webview_window("main").unwrap();
    // Define the on_drop_callback function
    let on_drop_callback = |result: drag::DragResult, _: drag::CursorPosition| {
        println!("Drag result: {:?}", result);
    };

    // Start the drag operation
    drag::start_drag(
        &(window.hwnd().unwrap().0 as isize),
        item,
        on_drop_callback,
        drag::Options::default(),
    )
    .expect("Failed to start drag operation");

    println!("Starting drag for file: {}", file_path);
    Ok(())
}

#[tauri::command]
fn start_multi_drag(app: tauri::AppHandle, file_paths: Vec<String>) -> Result<(), String> {
    println!(
        "Starting multi-file drag for files: {}",
        file_paths.join(", ")
    );

    let mut valid_paths = Vec::new();

    for file_path in file_paths {
        match std::fs::canonicalize(file_path.clone()) {
            Ok(path) => {
                if path.exists() {
                    valid_paths.push(path);
                } else {
                    println!("File not found: {}", file_path);
                }
            }
            Err(e) => {
                println!("Error finding file: {} ({})", file_path, e);
            }
        }
    }

    if valid_paths.is_empty() {
        return Err("No valid files to drag".to_string());
    }

    let item = drag::DragItem::Files(valid_paths);

    let window = app.get_webview_window("main").unwrap();
    let on_drop_callback = |result: drag::DragResult, _: drag::CursorPosition| {
        println!("Multi-file drag result: {:?}", result);
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![start_drag, start_multi_drag])
        .setup(|app| {
            #[cfg(all(desktop))]
            {
                let handle = app.handle();
                tray::create_tray(handle)?;
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
