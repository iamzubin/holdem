use tauri::Manager;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[cfg(desktop)]
mod tray;

#[tauri::command]
fn start_drag(app: tauri::AppHandle, file_path: String) -> Result<(), String> {

    let item = match std::fs::canonicalize("../icons/icon.png") {
        Ok(path) => drag::DragItem::Files(vec![path]),
        Err(e) => {
            eprintln!("Error finding icon file: {}", e);
            return Ok(());
        }
    };
    let window = app.get_webview_window("main").unwrap();
    // Define the on_drop_callback function
    let on_drop_callback = |result: drag::DragResult, _: drag::CursorPosition| {
        println!("Drag result: {:?}", result);
    };

    // Start the drag operation
    drag::start_drag(
        window.ns_window().expect("Failed to get ns_window"),
        item,
        on_drop_callback,
        drag::Options::default(),
    ).expect("Failed to start drag operation");

    println!("Starting drag for file sss: {}", file_path);
    Ok(())
}

#[tauri::command]
fn start_multi_drag(file_paths: Vec<String>) -> Result<(), String> {
    // Implement the OS-specific drag operation for multiple files here
    // This is a placeholder and needs to be implemented based on the OS
    println!("Starting multi-file drag for files: {}", file_paths.join(", "));
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
