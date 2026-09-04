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
