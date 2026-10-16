use crate::file::FileMetadata;
use crate::FileList;
use chrono::NaiveDate;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Emitter};
use tracing::{error, info, warn};

/// Name format of the per-day folders under `holdem_drops`.
const DROP_FOLDER_DATE_FORMAT: &str = "%Y%m%d";

#[derive(Debug, thiserror::Error)]
enum CleanupError {
    #[error("failed to read drops directory {path:?}: {source}")]
    ReadDir {
        path: PathBuf,
        source: std::io::Error,
    },
}

pub fn handle_file_drop_from_paths(
    paths: Vec<PathBuf>,
    file_list: FileList,
    app_handle: AppHandle,
) {
    tauri::async_runtime::spawn(async move {
        // Calculate all files metadata first. Keep the original reference:
        // dropping a file must never copy it. Each entry gets a unique id
        // from the global counter (never the list length, which repeats
        // after a removal).
        let new_files: Vec<FileMetadata> = paths
            .iter()
            .filter_map(|path| FileMetadata::from_path(path.clone()))
            .collect();

        // Now lock and add to list
        let Ok(mut list) = file_list.lock() else {
            error!(
                "file_drop: FileList mutex poisoned, dropping {} path(s)",
                paths.len()
            );
            return;
        };

        for file in new_files {
            // Avoid duplicates
            if !list.iter().any(|f| f.path == file.path) {
                info!("Added dropped file: {:?}", file.path);
                list.push(file);
            }
        }
        // Snapshot what is on the shelf so cleanup never deletes it.
        let shelf_paths: Vec<PathBuf> = list.iter().map(|f| f.path.clone()).collect();
        drop(list);

        if let Err(e) = app_handle.emit("files_updated", ()) {
            error!("Failed to emit files_updated event: {}", e);
        }

        let drops_root = std::env::temp_dir().join("holdem_drops");
        let today = chrono::Local::now().date_naive();
        match cleanup_old_drop_folders(&drops_root, today, &shelf_paths) {
            Ok(removed) if removed > 0 => {
                info!("Removed {} orphaned drop folder(s)", removed);
            }
            Ok(_) => {}
            Err(e) => warn!("Drop folder cleanup skipped: {}", e),
        }
    });
}

/// Delete orphaned per-day folders under `drops_root`.
///
/// A folder is removed only when all of these hold:
/// - its name parses as a `YYYYMMDD` date (unknown folders are left alone);
/// - that date is before yesterday, so content saved just before midnight
///   but not yet added to the shelf survives the date rollover;
/// - no current shelf entry lives inside it. Pasted text, screenshots and
///   downloaded images exist only in these folders, so deleting a referenced
///   one would destroy shelf content.
///
/// Returns the number of folders removed.
fn cleanup_old_drop_folders(
    drops_root: &Path,
    today: NaiveDate,
    shelf_paths: &[PathBuf],
) -> Result<usize, CleanupError> {
    let entries = match std::fs::read_dir(drops_root) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(0),
        Err(source) => {
            return Err(CleanupError::ReadDir {
                path: drops_root.to_path_buf(),
                source,
            })
        }
    };
    let Some(cutoff) = today.pred_opt() else {
        return Ok(0);
    };
    let shelf_keys: Vec<String> = shelf_paths.iter().map(|p| path_key(p)).collect();

    let mut removed = 0;
    for entry in entries.flatten() {
        let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
        if !is_dir {
            continue;
        }
        let Some(folder_date) = entry
            .file_name()
            .to_str()
            .and_then(|name| {
                NaiveDate::parse_from_str(name, DROP_FOLDER_DATE_FORMAT).ok()
            })
        else {
            continue;
        };
        if folder_date >= cutoff {
            continue;
        }
        let folder = entry.path();
        if is_referenced(&folder, &shelf_keys) {
            continue;
        }
        match std::fs::remove_dir_all(&folder) {
            Ok(()) => removed += 1,
            Err(e) => warn!("Failed to remove old drop folder {:?}: {}", folder, e),
        }
    }
    Ok(removed)
}

/// True when any shelf entry is the folder itself or lives under it.
fn is_referenced(folder: &Path, shelf_keys: &[String]) -> bool {
    let folder_key = path_key(folder);
    let folder_prefix = format!("{}\\", folder_key.trim_end_matches('\\'));
    shelf_keys
        .iter()
        .any(|k| *k == folder_key || k.starts_with(&folder_prefix))
}

/// Comparable form of a path: canonicalized when it exists (resolves 8.3
/// short names and `..`), `\\?\` prefix stripped, separators unified and
/// lowercased (Windows paths are case-insensitive). Errs on the side of
/// matching, which only ever keeps a folder rather than deleting it.
fn path_key(path: &Path) -> String {
    let resolved = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let s = resolved.to_string_lossy().replace('/', "\\");
    let s = s.strip_prefix("\\\\?\\").unwrap_or(&s);
    s.trim_end_matches('\\').to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TempRoot(PathBuf);

    impl TempRoot {
        fn new() -> Self {
            let root = std::env::temp_dir()
                .join(format!("holdem_cleanup_test_{}", uuid::Uuid::new_v4()));
            std::fs::create_dir_all(&root).expect("create temp root");
            Self(root)
        }

        fn day_folder(&self, name: &str) -> PathBuf {
            let dir = self.0.join(name);
            std::fs::create_dir_all(&dir).expect("create day folder");
            dir
        }
    }

    impl Drop for TempRoot {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).expect("valid date")
    }

    #[test]
    fn keeps_old_folder_with_shelf_file() {
        let root = TempRoot::new();
        let old = root.day_folder("20260101");
        let note = old.join("note.txt");
        std::fs::write(&note, b"pasted").expect("write note");

        let removed = cleanup_old_drop_folders(
            &root.0,
            date(2026, 10, 16),
            std::slice::from_ref(&note),
        )
        .expect("cleanup");

        assert_eq!(removed, 0);
        assert!(note.exists());
    }

    #[test]
    fn removes_orphaned_old_folder() {
        let root = TempRoot::new();
        let old = root.day_folder("20260101");
        std::fs::write(old.join("gone.png"), b"x").expect("write");

        let removed =
            cleanup_old_drop_folders(&root.0, date(2026, 10, 16), &[]).expect("cleanup");

        assert_eq!(removed, 1);
        assert!(!old.exists());
    }

    #[test]
    fn keeps_today_yesterday_and_unknown_folders() {
        let root = TempRoot::new();
        let today = root.day_folder("20261016");
        let yesterday = root.day_folder("20261015");
        let other = root.day_folder("not-a-date");

        let removed =
            cleanup_old_drop_folders(&root.0, date(2026, 10, 16), &[]).expect("cleanup");

        assert_eq!(removed, 0);
        assert!(today.exists() && yesterday.exists() && other.exists());
    }

    #[test]
    fn sibling_with_shared_prefix_is_not_a_reference() {
        let root = TempRoot::new();
        let old = root.day_folder("20260101");
        let sibling = root.day_folder("20260101_extra");
        let kept = sibling.join("a.txt");
        std::fs::write(&kept, b"x").expect("write");

        let removed =
            cleanup_old_drop_folders(&root.0, date(2026, 10, 16), &[kept]).expect("cleanup");

        assert_eq!(removed, 1);
        assert!(!old.exists());
    }

    #[test]
    fn reference_matching_ignores_case_and_separators() {
        let root = TempRoot::new();
        let old = root.day_folder("20260101");
        let file = old.join("shot.png");
        std::fs::write(&file, b"x").expect("write");
        let alt = PathBuf::from(file.to_string_lossy().to_uppercase().replace('\\', "/"));

        let removed =
            cleanup_old_drop_folders(&root.0, date(2026, 10, 16), &[alt]).expect("cleanup");

        assert_eq!(removed, 0);
        assert!(file.exists());
    }

    #[test]
    fn missing_root_is_ok() {
        let missing = std::env::temp_dir()
            .join(format!("holdem_cleanup_missing_{}", uuid::Uuid::new_v4()));
        let removed =
            cleanup_old_drop_folders(&missing, date(2026, 10, 16), &[]).expect("cleanup");
        assert_eq!(removed, 0);
    }
}
