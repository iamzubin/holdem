use crate::file::FileMetadata;
use crate::FileList;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter};
use tracing::{error, info};

pub fn handle_file_drop_from_paths(
    paths: Vec<PathBuf>,
    file_list: FileList,
    app_handle: AppHandle,
) {
    tauri::async_runtime::spawn(async move {
        // Calculate all files metadata first
        let mut new_files = Vec::new();

        for path in paths.iter() {
            // Keep the original reference. Dropping a file must never copy it.
            // Ids are assigned below once the list is locked.
            if let Some(file) = FileMetadata::from_path(0, path.clone()) {
                new_files.push(file);
            }
        }

        // Now lock and add to list
        let Ok(mut list) = file_list.lock() else {
            error!(
                "file_drop: FileList mutex poisoned, dropping {} path(s)",
                paths.len()
            );
            return;
        };
        let starting_id = list.len() as u64;

        for (i, mut file) in new_files.into_iter().enumerate() {
            file.id = starting_id + i as u64;
            // Avoid duplicates
            if !list.iter().any(|f| f.path == file.path) {
                info!("Added dropped file: {:?}", file.path);
                list.push(file);
            }
        }
        drop(list);

        if let Err(e) = app_handle.emit("files_updated", ()) {
            error!("Failed to emit files_updated event: {}", e);
        }

        // Cleanup old files
        cleanup_old_files();
    });
}

fn cleanup_old_files() {
    let temp_dir = std::env::temp_dir().join("holdem_drops");
    if let Ok(entries) = std::fs::read_dir(temp_dir) {
        let today = chrono::Local::now().format("%Y%m%d").to_string();
        for entry in entries.flatten() {
            if let Ok(file_type) = entry.file_type() {
                if file_type.is_dir() {
                    if let Some(folder_name) = entry.file_name().to_str() {
                        // If the folder is not from today, delete it
                        if folder_name != today {
                            let _ = std::fs::remove_dir_all(entry.path());
                        }
                    }
                }
            }
        }
    }
}
