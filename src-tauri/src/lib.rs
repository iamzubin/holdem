// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[cfg(desktop)]
mod tray;

#[tauri::command]
fn start_drag(file_path: String) -> Result<(), String> {
    // Implement the OS-specific drag operation here
    // This is a placeholder and needs to be implemented based on the OS
    println!("Starting drag for file: {}", file_path);
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
