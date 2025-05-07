use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub id: u64,
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
    pub file_type: String,
}

// Calculate the size of a directory by recursively summing all file sizes
pub fn get_dir_size(path: &PathBuf) -> io::Result<u64> {
    let mut total_size = 0;

    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                // Recursively calculate the size of subdirectories
                match get_dir_size(&path) {
                    Ok(size) => total_size += size,
                    Err(_) => {} // Skip directories we can't access
                }
            } else {
                // Add the file size
                match fs::metadata(&path) {
                    Ok(metadata) => total_size += metadata.len(),
                    Err(_) => {} // Skip files we can't access
                }
            }
        }
    }

    Ok(total_size)
}
