use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub id: u64,
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
    pub file_type: String,
}

// Folder sizes are intentionally not computed: directory drops store size 0
// and the UI shows them as folders. This keeps drops instant even for huge
// trees (node_modules, photo libraries) and avoids recursive I/O on the
// drop hot path.

impl FileMetadata {
    /// Build metadata for an existing, readable path. Returns `None` for
    /// missing paths or unreadable metadata (callers skip those files).
    /// The caller owns id assignment (position in the shelf list).
    pub fn from_path(id: u64, path: PathBuf) -> Option<Self> {
        if !path.exists() {
            return None;
        }
        let metadata = path.metadata().ok()?;
        Some(Self {
            id,
            name: path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Unknown")
                .to_string(),
            path: path.clone(),
            // Directories store size 0 — no recursive walk (instant drops).
            size: if metadata.is_dir() {
                0
            } else {
                metadata.len()
            },
            file_type: if metadata.is_dir() {
                "folder".to_string()
            } else {
                path.extension()
                    .and_then(|ext| ext.to_str())
                    .unwrap_or("unknown")
                    .to_string()
            },
        })
    }
}
