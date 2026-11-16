use crate::drop_report::{DropIssue, DropReport};
use crate::file::FileMetadata;
use crate::FileList;
use chrono::NaiveDate;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Emitter, Manager};
use tracing::{error, info, warn};

/// Set when the saved shelf could not be read at startup. Cleanup is then
/// skipped for the whole session: it can't tell what the lost shelf used.
pub static CLEANUP_BLOCKED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Saved drops stay at least this long after they leave the shelf, however
/// they left it (Remove, Clear, or a shelf file that went missing).
const KEEP_DAYS: u64 = 7;

/// Name format of the per-day folders under [`drops_root`].
pub(crate) const DROP_FOLDER_DATE_FORMAT: &str = "%Y%m%d";

/// Where Holdem keeps content that exists only on the shelf: dropped text,
/// screenshots, web images and email attachments. Deliberately not %TEMP%,
/// which Storage Sense and cleanup tools empty without asking.
pub fn drops_root() -> PathBuf {
    std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join("Holdem")
        .join("drops")
}

/// A path's last component, for messages.
fn display_name(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string_lossy().into_owned())
}

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
    add_paths(paths, DropReport::new(), file_list, app_handle);
}

/// Adds `paths` to the active shelf, then tells the user what happened to
/// the whole drop: `report` holds what went wrong before this step (items
/// that could not be saved), and items that can't be read or are already
/// on the shelf are added to it here.
pub fn add_paths(
    paths: Vec<PathBuf>,
    report: DropReport,
    file_list: FileList,
    app_handle: AppHandle,
) {
    add_paths_to(paths, None, report, file_list, app_handle);
}

/// [`add_paths`], onto `target_shelf` instead of the active shelf.
pub fn add_paths_to(
    paths: Vec<PathBuf>,
    target_shelf: Option<u64>,
    mut report: DropReport,
    file_list: FileList,
    app_handle: AppHandle,
) {
    tauri::async_runtime::spawn(async move {
        // Calculate all files metadata first. Keep the original reference:
        // dropping a file must never copy it. Each entry gets a unique id
        // from the global counter (never the list length, which repeats
        // after a removal).
        let mut new_files: Vec<FileMetadata> = Vec::with_capacity(paths.len());
        for path in &paths {
            match FileMetadata::from_path(path.clone()) {
                Some(file) => new_files.push(file),
                None => report.add_named(DropIssue::Unreadable, &display_name(path)),
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

        // The shelf these were made for may have been closed meanwhile.
        let open_shelves: Option<Vec<u64>> = app_handle
            .try_state::<crate::shelves::Shelves>()
            .and_then(|s| s.lock().ok().map(|set| set.shelves.iter().map(|s| s.id).collect()));
        let mut added = 0usize;
        for mut file in new_files {
            if let Some(shelf) = target_shelf {
                file.shelf = shelf;
            }
            if open_shelves.as_ref().is_some_and(|open| !open.contains(&file.shelf)) {
                file.shelf = crate::shelves::active_id();
            }
            // Avoid duplicates on the same shelf
            if list.iter().any(|f| f.path == file.path && f.shelf == file.shelf) {
                report.add_named(DropIssue::AlreadyOnShelf, &file.name);
            } else {
                // Paths only at debug level: logs shouldn't list a user's files.
                tracing::debug!("Added dropped file: {:?}", file.path);
                list.push(file);
                added += 1;
            }
        }
        // Snapshot what is on the shelf, plus what Undo and "Reopen closed
        // shelf" can still bring back, so cleanup never deletes any of it.
        // The list stays locked while the closed shelves are read: closing
        // and reopening move items between the two under both locks.
        let mut shelf_paths: Vec<PathBuf> = list.iter().map(|f| f.path.clone()).collect();
        if let Some(shelves) = app_handle.try_state::<crate::shelves::Shelves>() {
            if let Ok(set) = shelves.lock() {
                shelf_paths.extend(set.closed_paths().cloned());
            }
        }
        // Undo's items too, still under the list lock: restoring moves
        // them from the stash to the list under both locks.
        if let Some(stash) = app_handle.try_state::<crate::RemovedStash>() {
            if let Ok(stash) = stash.lock() {
                shelf_paths.extend(stash.iter().map(|f| f.path.clone()));
            }
        }
        drop(list);

        if let Err(e) = app_handle.emit("files_updated", ()) {
            error!("Failed to emit files_updated event: {}", e);
        }
        if added > 0 {
            crate::analytics::track(&app_handle, "files_added", vec![("count", added.into())]);
        }
        report.set_added(added);
        report.send(&app_handle);

        if CLEANUP_BLOCKED.load(std::sync::atomic::Ordering::Relaxed) {
            return;
        }
        // File-system work, some of it slow: off the async workers. One at a
        // time: two drops close together would otherwise both ask about a
        // folder that can't be recycled.
        if CLEANING.swap(true, std::sync::atomic::Ordering::SeqCst) {
            return;
        }
        tauri::async_runtime::spawn_blocking(move || {
            let _done = ClearOnDrop(&CLEANING);
            let today = chrono::Local::now().date_naive();
            let left_before = std::time::SystemTime::now() - KEEP;
            match cleanup_old_drop_folders(
                &drops_root(),
                today,
                left_before,
                &shelf_paths,
                move_to_recycle_bin,
            ) {
                Ok(removed) if removed > 0 => {
                    info!("Removed {} orphaned drop folder(s)", removed);
                }
                Ok(_) => {}
                Err(e) => warn!("Drop folder cleanup skipped: {}", e),
            }
        });
    });
}

/// Set while a drop-folder cleanup runs.
static CLEANING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Folders whose removal was refused this session (the user answered "no"
/// to deleting one that can't be recycled): not asked about again.
static REFUSED: std::sync::Mutex<Vec<PathBuf>> = std::sync::Mutex::new(Vec::new());

/// Clears a flag when dropped, also on a panic.
struct ClearOnDrop(&'static std::sync::atomic::AtomicBool);

impl Drop for ClearOnDrop {
    fn drop(&mut self) {
        self.0.store(false, std::sync::atomic::Ordering::SeqCst);
    }
}

/// [`KEEP_DAYS`] as a duration.
const KEEP: std::time::Duration = std::time::Duration::from_secs(KEEP_DAYS * 24 * 60 * 60);

/// The per-day drop folder `path` lives in, if it is one of Holdem's.
/// Plain string work: no file-system calls, so offline paths cost nothing.
fn day_folder_of(path: &Path, root: &Path) -> Option<PathBuf> {
    let root_key = root.to_string_lossy().replace('/', "\\").to_lowercase();
    let root_key = root_key.trim_end_matches('\\');
    let path_key = path.to_string_lossy().replace('/', "\\").to_lowercase();
    let rest = path_key.strip_prefix(root_key)?.strip_prefix('\\')?;
    let day = rest.split('\\').next().filter(|d| !d.is_empty())?;
    Some(root.join(day))
}

/// True when `path` may point into `root`, decided without slow file-system
/// calls: it does in the usual spelling, and any other spelling of a path
/// on the same local drive (a `\\?\\` prefix, an 8.3 short name) is
/// resolved to check. Paths on other drives and network shares, which can
/// take many seconds to resolve when offline, can't be in the drop folder.
fn may_be_in(path: &Path, root: &Path) -> bool {
    if day_folder_of(path, root).is_some() {
        return true;
    }
    let drive = |p: &Path| {
        let s = p.to_string_lossy().to_lowercase();
        let s = s.strip_prefix(r"\\?\").unwrap_or(&s).to_string();
        let b = s.as_bytes();
        (b.len() >= 2 && b[1] == b':').then(|| b[0])
    };
    drive(path).is_some() && drive(path) == drive(root)
}

/// Records that saved items left the shelf (and Undo): their day folders
/// count as used now, so the cleanup keeps them for another [`KEEP_DAYS`].
/// The folders' modified time is the record, so it needs no extra file.
pub fn mark_left_shelf<'a>(paths: impl IntoIterator<Item = &'a Path>) {
    mark_left_shelf_in(&drops_root(), paths);
}

fn mark_left_shelf_in<'a>(root: &Path, paths: impl IntoIterator<Item = &'a Path>) {
    let mut days: Vec<PathBuf> = paths.into_iter().filter_map(|p| day_folder_of(p, root)).collect();
    days.sort();
    days.dedup();
    for day in days {
        if let Err(e) = touch(&day) {
            warn!("Couldn't mark a drop folder as recently used: {}", e);
        }
    }
}

/// Sets a folder's modified time to now.
fn touch(dir: &Path) -> std::io::Result<()> {
    use std::os::windows::fs::OpenOptionsExt;
    const FILE_WRITE_ATTRIBUTES: u32 = 0x100;
    // Needed to open a folder rather than a file.
    const FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x0200_0000;
    std::fs::OpenOptions::new()
        .access_mode(FILE_WRITE_ATTRIBUTES)
        .custom_flags(FILE_FLAG_BACKUP_SEMANTICS)
        .open(dir)?
        .set_modified(std::time::SystemTime::now())
}

/// Moves a folder to the Recycle Bin, so a cleanup mistake is recoverable.
/// When the Recycle Bin can't take it (recycling is off for the drive, or
/// the folder is too large), Windows asks before deleting it for good
/// (FOF_WANTNUKEWARNING); without that flag, FOF_NOCONFIRMATION would answer
/// "yes" and the content would be gone.
fn move_to_recycle_bin(path: &Path) -> std::io::Result<()> {
    if REFUSED.lock().is_ok_and(|refused| refused.iter().any(|p| p == path)) {
        return Err(std::io::Error::other("kept: deleting it was refused earlier"));
    }
    use std::os::windows::ffi::OsStrExt;
    use windows::core::PCWSTR;
    use windows::Win32::UI::Shell::{
        SHFileOperationW, FOF_ALLOWUNDO, FOF_NOCONFIRMATION, FOF_NOERRORUI, FOF_SILENT,
        FOF_WANTNUKEWARNING, FO_DELETE, SHFILEOPSTRUCTW,
    };
    // pFrom is a list of paths ending in two NULs.
    let mut from: Vec<u16> = path.as_os_str().encode_wide().collect();
    from.extend([0, 0]);
    let mut op = SHFILEOPSTRUCTW {
        wFunc: FO_DELETE,
        pFrom: PCWSTR(from.as_ptr()),
        fFlags: (FOF_ALLOWUNDO
            | FOF_NOCONFIRMATION
            | FOF_WANTNUKEWARNING
            | FOF_NOERRORUI
            | FOF_SILENT)
            .0 as u16,
        ..Default::default()
    };
    let code = unsafe { SHFileOperationW(&mut op) };
    if op.fAnyOperationsAborted.as_bool() {
        if let Ok(mut refused) = REFUSED.lock() {
            refused.push(path.to_path_buf());
        }
        return Err(std::io::Error::other("kept: deleting it was refused"));
    }
    if code != 0 {
        return Err(std::io::Error::other(format!("SHFileOperation failed ({code})")));
    }
    Ok(())
}

/// Removes orphaned per-day folders under `drops_root` with `remove` (the
/// Recycle Bin in the app).
///
/// A folder is removed only when all of these hold:
/// - its name parses as a `YYYYMMDD` date (unknown folders are left alone);
/// - that date is more than [`KEEP_DAYS`] ago, and the folder was last
///   modified before `left_before`: items that left the shelf mark their
///   folder (see [`mark_left_shelf`]), so they can still be restored for a
///   week from the folder itself;
/// - no shelf entry, and nothing Undo can bring back, lives inside it.
///   Pasted text, screenshots and downloaded images exist only in these
///   folders, so removing a referenced one would destroy shelf content.
///
/// Returns the number of folders removed.
fn cleanup_old_drop_folders(
    drops_root: &Path,
    today: NaiveDate,
    left_before: std::time::SystemTime,
    shelf_paths: &[PathBuf],
    remove: impl Fn(&Path) -> std::io::Result<()>,
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
    let Some(cutoff) = today.checked_sub_days(chrono::Days::new(KEEP_DAYS)) else {
        return Ok(0);
    };
    // Canonicalizing every shelf path is filesystem work, so only do it once a
    // folder is actually old enough to delete.
    let mut shelf_keys: Option<Vec<String>> = None;

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
        let recently_used = entry
            .metadata()
            .and_then(|m| m.modified())
            .map_or(true, |modified| modified >= left_before);
        if recently_used {
            continue;
        }
        let folder = entry.path();
        // Only paths inside the drop folder can be in it: the rest (often
        // on network shares, where resolving a path can take many seconds
        // when offline) are left out without touching the disk.
        let keys = shelf_keys.get_or_insert_with(|| {
            shelf_paths
                .iter()
                .filter(|p| may_be_in(p, drops_root))
                .map(|p| path_key(p))
                .collect()
        });
        if is_referenced(&folder, keys) {
            continue;
        }
        match remove(&folder) {
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
pub(crate) fn path_key(path: &Path) -> String {
    let resolved = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let s = resolved.to_string_lossy().replace('/', "\\");
    let s = s.strip_prefix("\\\\?\\").unwrap_or(&s);
    s.trim_end_matches('\\').to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rm(path: &Path) -> std::io::Result<()> {
        std::fs::remove_dir_all(path)
    }

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

    /// A "left before" time after every folder in these tests was touched.
    fn long_after() -> std::time::SystemTime {
        std::time::SystemTime::now() + std::time::Duration::from_secs(60)
    }

    #[test]
    fn keeps_old_folder_whose_items_just_left_the_shelf() {
        use std::os::windows::fs::OpenOptionsExt;
        let root = TempRoot::new();
        let old = root.day_folder("20260101");
        std::fs::write(old.join("note.txt"), b"x").expect("write");
        // Untouched for a month: it goes.
        let month_ago = std::time::SystemTime::now() - KEEP * 4;
        std::fs::OpenOptions::new()
            .access_mode(0x100)
            .custom_flags(0x0200_0000)
            .open(&old)
            .and_then(|f| f.set_modified(month_ago))
            .expect("age the folder");
        let left_before = std::time::SystemTime::now() - KEEP;
        let would_remove = cleanup_old_drop_folders(&root.0, date(2026, 10, 16), left_before, &[], |_| Ok(()))
            .expect("cleanup");
        assert_eq!(would_remove, 1);

        // An item from it just left the shelf: it stays.
        mark_left_shelf_in(&root.0, [old.join("note.txt").as_path()]);
        let removed = cleanup_old_drop_folders(&root.0, date(2026, 10, 16), left_before, &[], rm)
            .expect("cleanup");
        assert_eq!(removed, 0);
        assert!(old.exists());
    }

    #[test]
    fn other_spellings_of_drop_paths_still_count() {
        let root = TempRoot::new();
        let old = root.day_folder("20260101");
        let note = old.join("note.txt");
        std::fs::write(&note, b"x").expect("write");
        let verbatim = PathBuf::from(format!(r"\\?\{}", note.display()));
        assert!(may_be_in(&verbatim, &root.0));
        let removed = cleanup_old_drop_folders(&root.0, date(2026, 10, 16), long_after(), &[verbatim], rm)
            .expect("cleanup");
        assert_eq!(removed, 0);
        assert!(note.exists());
        assert!(!may_be_in(Path::new(r"\\nas\share\a.txt"), &root.0));
    }

    #[test]
    fn day_folders_are_found_without_the_disk() {
        let root = PathBuf::from(r"C:\Users\a\AppData\Local\Holdem\drops");
        let item = PathBuf::from(r"c:/users/A/AppData/Local/Holdem/drops/20260101/note-1/n.txt");
        assert_eq!(day_folder_of(&item, &root), Some(root.join("20260101")));
        assert_eq!(day_folder_of(Path::new(r"\\nas\share\a.txt"), &root), None);
        assert_eq!(day_folder_of(Path::new(r"C:\Users\a\AppData\Local\Holdem\dropsX\1"), &root), None);
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
            long_after(),
            std::slice::from_ref(&note),
            rm,
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
            cleanup_old_drop_folders(&root.0, date(2026, 10, 16), long_after(), &[], rm).expect("cleanup");

        assert_eq!(removed, 1);
        assert!(!old.exists());
    }

    #[test]
    fn keeps_last_week_and_unknown_folders() {
        let root = TempRoot::new();
        let today = root.day_folder("20261016");
        let yesterday = root.day_folder("20261015");
        let week_ago = root.day_folder("20261009");
        let other = root.day_folder("not-a-date");

        let removed =
            cleanup_old_drop_folders(&root.0, date(2026, 10, 16), long_after(), &[], rm).expect("cleanup");

        assert_eq!(removed, 0);
        assert!(today.exists() && yesterday.exists() && week_ago.exists() && other.exists());
    }

    #[test]
    fn sibling_with_shared_prefix_is_not_a_reference() {
        let root = TempRoot::new();
        let old = root.day_folder("20260101");
        let sibling = root.day_folder("20260101_extra");
        let kept = sibling.join("a.txt");
        std::fs::write(&kept, b"x").expect("write");

        let removed =
            cleanup_old_drop_folders(&root.0, date(2026, 10, 16), long_after(), &[kept], rm).expect("cleanup");

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
            cleanup_old_drop_folders(&root.0, date(2026, 10, 16), long_after(), &[alt], rm).expect("cleanup");

        assert_eq!(removed, 0);
        assert!(file.exists());
    }

    #[test]
    fn missing_root_is_ok() {
        let missing = std::env::temp_dir()
            .join(format!("holdem_cleanup_missing_{}", uuid::Uuid::new_v4()));
        let removed =
            cleanup_old_drop_folders(&missing, date(2026, 10, 16), long_after(), &[], rm).expect("cleanup");
        assert_eq!(removed, 0);
    }
}
