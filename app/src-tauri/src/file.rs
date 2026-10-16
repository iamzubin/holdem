use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

/// Process-wide source of shelf file ids. Ids must never be reused while the
/// app runs: deriving them from the list length handed out duplicates after a
/// removal, so select/remove/drag could hit the wrong file. The shelf lives
/// only in memory, so a per-process counter is sufficient. Starts at 1 and
/// stays far below 2^53, so the frontend's `number` represents it exactly.
static NEXT_FILE_ID: AtomicU64 = AtomicU64::new(1);

/// Allocate a fresh, unique, monotonically increasing file id.
pub fn next_file_id() -> u64 {
    NEXT_FILE_ID.fetch_add(1, Ordering::Relaxed)
}

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
    /// Each call allocates a fresh unique id via [`next_file_id`]; ids skipped
    /// because the entry turned out to be a duplicate are simply never used.
    pub fn from_path(path: PathBuf) -> Option<Self> {
        if !path.exists() {
            return None;
        }
        let metadata = path.metadata().ok()?;
        Some(Self {
            id: next_file_id(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_unique_and_monotonic() {
        let a = next_file_id();
        let b = next_file_id();
        let c = next_file_id();
        assert!(a < b && b < c);
    }

    #[test]
    fn ids_not_reused_after_removal() {
        let dir = std::env::temp_dir();
        let mut list: Vec<FileMetadata> = (0..3)
            .filter_map(|_| FileMetadata::from_path(dir.clone()))
            .collect();
        assert_eq!(list.len(), 3);
        list.remove(0);
        let added = FileMetadata::from_path(dir).expect("temp dir exists");
        assert!(list.iter().all(|f| f.id != added.id));
    }

    #[test]
    fn missing_path_yields_none() {
        let missing = std::env::temp_dir()
            .join(format!("holdem_missing_{}", uuid::Uuid::new_v4()));
        assert!(FileMetadata::from_path(missing).is_none());
    }
}
