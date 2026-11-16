//! Keeps the shelf across restarts and updates. The list of paths is written
//! to `shelf.json` in the app data folder whenever the shelf changes, and
//! read back at startup. Only paths are stored; names, sizes and types are
//! recomputed on load. Entries whose file is gone are dropped; ones on a
//! drive or share that is offline stay, marked unavailable.

use crate::file::FileMetadata;
use crate::shelves::{ShelfSet, Shelves, FIRST_SHELF};
use crate::presence::{self, Presence};
use crate::FileList;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Listener, Manager};
use tracing::{info, warn};

const STORE_FILE: &str = "shelf.json";
/// How long startup waits for drives and shares to answer.
const LOAD_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(2);

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("no app data folder: {0}")]
    DataDir(String),
    #[error("shelf file I/O failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("shelf file is not valid JSON: {0}")]
    Json(#[from] serde_json::Error),
}

fn store_path(app: &AppHandle) -> Result<PathBuf, StoreError> {
    app.path()
        .app_data_dir()
        .map(|dir| dir.join(STORE_FILE))
        .map_err(|e| StoreError::DataDir(e.to_string()))
}

/// What [`load`] found.
pub struct Loaded {
    pub files: Vec<FileMetadata>,
    pub shelves: ShelfSet,
    /// Saved entries whose file was deleted (not merely offline).
    pub missing: usize,
    /// The saved shelf could not be read. Cleanup must then not run: it
    /// would treat everything the lost shelf referenced as unused.
    pub failed: bool,
}

/// Shelf entries saved by the previous run, in their original order, with
/// fresh ids. A corrupt file is kept as `shelf.json.bak` for recovery.
pub fn load(app: &AppHandle) -> Loaded {
    let path = match store_path(app) {
        Ok(path) => path,
        Err(e) => {
            warn!("Could not restore the shelf: {}", e);
            return Loaded { files: Vec::new(), shelves: ShelfSet::default().normalized(), missing: 0, failed: true };
        }
    };
    match read_store(&path) {
        Ok(Saved { shelves, items }) => {
            let shelves = shelves.normalized();
            let saved = items.len();
            // Bounded: a network share that doesn't answer must not hold up
            // startup; its items load as offline and are checked again later.
            let only_paths: Vec<PathBuf> = items.iter().map(|item| item.path.clone()).collect();
            let found = presence::check(&only_paths, LOAD_TIMEOUT);
            let files: Vec<FileMetadata> = items
                .into_iter()
                .zip(found)
                .filter_map(|(item, presence)| {
                    let Item { path, pinned, shelf } = item;
                    // An item whose shelf is gone lands on the active one.
                    let shelf = if shelves.contains(shelf) { shelf } else { shelves.active };
                    let file = match presence {
                        Presence::Present => FileMetadata::from_path(path.clone())
                            // Vanished between the check and the read.
                            .unwrap_or_else(|| FileMetadata::unavailable(path, pinned)),
                        Presence::Unreachable => FileMetadata::unavailable(path, pinned),
                        Presence::Gone => return None,
                    };
                    Some(FileMetadata { pinned, shelf, ..file })
                })
                .collect();
            let missing = saved - files.len();
            let offline = files.iter().filter(|f| !f.available).count();
            info!(
                "Restored {} shelf item(s): {} offline, {} deleted",
                files.len(),
                offline,
                missing
            );
            Loaded { files, shelves, missing, failed: false }
        }
        Err(StoreError::Io(e)) if e.kind() == std::io::ErrorKind::NotFound => {
            Loaded { files: Vec::new(), shelves: ShelfSet::default().normalized(), missing: 0, failed: false }
        }
        Err(e) => {
            warn!("Could not restore the shelf: {}", e);
            let backup = path.with_extension("json.bak");
            match fs::copy(&path, &backup) {
                Ok(_) => warn!("Kept the unreadable shelf as {:?}", backup),
                Err(e) => warn!("Could not back up the unreadable shelf: {}", e),
            }
            Loaded { files: Vec::new(), shelves: ShelfSet::default().normalized(), missing: 0, failed: true }
        }
    }
}

/// One saved item. Holdem 3.1 stored bare paths; both forms load.
#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
enum Stored {
    Entry {
        path: PathBuf,
        #[serde(default)]
        pinned: bool,
        #[serde(default = "first_shelf")]
        shelf: u64,
    },
    Path(PathBuf),
}

fn first_shelf() -> u64 {
    FIRST_SHELF
}

/// The saved file: shelves and their items. A plain list is the older,
/// single-shelf format.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
enum StoreFile {
    Shelves { shelves: ShelfSet, items: Vec<Stored> },
    Items(Vec<Stored>),
}

#[derive(Debug, Clone, PartialEq)]
struct Item {
    path: PathBuf,
    pinned: bool,
    shelf: u64,
}

impl From<Stored> for Item {
    fn from(stored: Stored) -> Self {
        match stored {
            Stored::Entry { path, pinned, shelf } => Item { path, pinned, shelf },
            Stored::Path(path) => Item { path, pinned: false, shelf: FIRST_SHELF },
        }
    }
}

#[derive(Debug)]
struct Saved {
    shelves: ShelfSet,
    items: Vec<Item>,
}

fn read_store(path: &Path) -> Result<Saved, StoreError> {
    let contents = fs::read_to_string(path)?;
    Ok(match serde_json::from_str::<StoreFile>(&contents)? {
        StoreFile::Shelves { shelves, items } => {
            Saved { shelves, items: items.into_iter().map(Item::from).collect() }
        }
        StoreFile::Items(items) => {
            Saved { shelves: ShelfSet::default(), items: items.into_iter().map(Item::from).collect() }
        }
    })
}

/// Writes through a temporary file and a rename, so a crash mid-write leaves
/// the previous shelf intact instead of a truncated file.
fn write_store(path: &Path, shelves: &ShelfSet, items: &[Item]) -> Result<(), StoreError> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let file = StoreFile::Shelves {
        shelves: shelves.clone(),
        items: items
            .iter()
            .map(|i| Stored::Entry { path: i.path.clone(), pinned: i.pinned, shelf: i.shelf })
            .collect(),
    };
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, serde_json::to_vec(&file)?)?;
    fs::rename(&tmp, path).inspect_err(|_| {
        let _ = fs::remove_file(&tmp);
    })?;
    Ok(())
}

/// Saves the shelf every time it changes (`files_updated`).
pub fn save_on_change(app: &AppHandle) {
    // Serialises writes. Each write snapshots the shelf only once it holds the
    // lock, so the file always ends up matching the latest state even when
    // saves finish out of order.
    let write_lock = Arc::new(Mutex::new(()));
    let handle = app.clone();
    app.listen_any("files_updated", move |_| {
        let handle = handle.clone();
        let write_lock = write_lock.clone();
        tauri::async_runtime::spawn_blocking(move || {
            let Ok(_guard) = write_lock.lock() else { return };
            let Some(list) = handle.try_state::<FileList>() else { return };
            let Some(shelves) = handle.try_state::<Shelves>() else { return };
            let items: Vec<Item> = match list.lock() {
                Ok(list) => list
                    .iter()
                    .map(|f| Item { path: f.path.clone(), pinned: f.pinned, shelf: f.shelf })
                    .collect(),
                Err(_) => return,
            };
            let Ok(shelves) = shelves.lock().map(|s| s.clone()) else { return };
            if let Err(e) = store_path(&handle).and_then(|path| write_store(&path, &shelves, &items)) {
                warn!("Could not save the shelf: {}", e);
            }
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shelves::Shelf;

    fn temp_store() -> PathBuf {
        std::env::temp_dir()
            .join(format!("holdem_store_test_{}", uuid::Uuid::new_v4()))
            .join(STORE_FILE)
    }

    #[test]
    fn round_trips_shelves_and_items_in_order() {
        let path = temp_store();
        let shelves = ShelfSet {
            shelves: vec![Shelf { id: 1, name: String::new() }, Shelf { id: 2, name: "Work".into() }],
            active: 2,
            // Closed shelves are saved too, so they can be reopened after a restart.
            closed: vec![crate::shelves::ClosedShelf {
                shelf: Shelf { id: 3, name: "Old".into() },
                index: 1,
                items: vec![crate::shelves::ClosedItem { path: PathBuf::from(r"C:\c.txt"), pinned: true }],
                closed_at: 1_800_000_000,
            }],
        };
        let items = vec![
            Item { path: PathBuf::from(r"C:\b.txt"), pinned: true, shelf: 2 },
            Item { path: PathBuf::from(r"C:\a dir\ä.png"), pinned: false, shelf: 1 },
        ];
        write_store(&path, &shelves, &items).expect("write");
        let saved = read_store(&path).expect("read");
        assert_eq!(saved.shelves, shelves);
        assert_eq!(saved.items, items);
        assert!(!path.with_extension("json.tmp").exists());
        let _ = fs::remove_dir_all(path.parent().expect("parent"));
    }

    #[test]
    fn reads_both_older_formats() {
        let path = temp_store();
        fs::create_dir_all(path.parent().expect("parent")).expect("dir");
        fs::write(&path, br#"["C:\\old.txt", {"path": "C:\\pinned.txt", "pinned": true}]"#)
            .expect("write");
        let saved = read_store(&path).expect("read");
        assert_eq!(saved.shelves, ShelfSet::default());
        assert_eq!(
            saved.items,
            vec![
                Item { path: PathBuf::from(r"C:\old.txt"), pinned: false, shelf: FIRST_SHELF },
                Item { path: PathBuf::from(r"C:\pinned.txt"), pinned: true, shelf: FIRST_SHELF },
            ]
        );
        let _ = fs::remove_dir_all(path.parent().expect("parent"));
    }

    #[test]
    fn corrupt_file_is_an_error_not_a_panic() {
        let path = temp_store();
        fs::create_dir_all(path.parent().expect("parent")).expect("dir");
        fs::write(&path, b"{not json").expect("write");
        assert!(matches!(read_store(&path), Err(StoreError::Json(_))));
        let _ = fs::remove_dir_all(path.parent().expect("parent"));
    }
}
