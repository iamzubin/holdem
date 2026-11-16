//! Things to do with shelf items besides dragging them: put them in a ZIP
//! archive (which lands on the shelf), or copy them into a folder.

use crate::file::FileMetadata;
use crate::FileList;
use std::collections::HashSet;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use tauri::{AppHandle, Emitter, State, WebviewWindow};
use tracing::warn;
use zip::write::SimpleFileOptions;

/// Available shelf items among `ids`, in shelf order, with the first one's shelf.
fn usable_paths(file_list: &FileList, ids: &[u64]) -> Result<(Vec<PathBuf>, u64), String> {
    let list = file_list.lock().map_err(|_| "Failed to acquire lock".to_string())?;
    let items: Vec<&FileMetadata> =
        list.iter().filter(|f| ids.contains(&f.id) && f.available).collect();
    let shelf = items.first().map(|f| f.shelf).unwrap_or_else(crate::shelves::active_id);
    let paths = items.into_iter().map(|f| f.path.clone()).collect::<Vec<_>>();
    if paths.is_empty() {
        return Err("Nothing to use: the items are offline or gone".to_string());
    }
    Ok((paths, shelf))
}

// ------------------------------------------------------------------ ZIP

/// The file's modified time in the ZIP's format (local time, from 1980).
fn zip_time(path: &Path) -> Option<zip::DateTime> {
    use chrono::{Datelike, Timelike};
    let modified = std::fs::metadata(path).ok()?.modified().ok()?;
    let t: chrono::DateTime<chrono::Local> = modified.into();
    zip::DateTime::from_date_and_time(
        u16::try_from(t.year()).ok()?,
        t.month() as u8,
        t.day() as u8,
        t.hour() as u8,
        t.minute() as u8,
        t.second() as u8,
    )
    .ok()
}

/// `name`, or `name (2)`, `name (3)`… if the archive already has it.
fn unique_name(taken: &mut HashSet<String>, name: &str) -> String {
    let (stem, ext) = match name.rfind('.') {
        Some(dot) if dot > 0 => (&name[..dot], &name[dot..]),
        _ => (name, ""),
    };
    let mut candidate = name.to_string();
    let mut n = 2;
    while !taken.insert(candidate.to_lowercase()) {
        candidate = format!("{stem} ({n}){ext}");
        n += 1;
    }
    candidate
}

fn add_path<W: Write + std::io::Seek>(
    zip: &mut zip::ZipWriter<W>,
    path: &Path,
    name: &str,
    skip: &str,
) -> Result<(), String> {
    let base = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    let options = match zip_time(path) {
        Some(time) => base.last_modified_time(time),
        None => base,
    };
    let meta = std::fs::symlink_metadata(path).map_err(|e| e.to_string())?;
    if meta.is_dir() {
        // The folder the archive is being written into, when a zipped
        // folder holds Holdem's own drop folder: the archive would end up
        // inside itself.
        if crate::file_drop::path_key(path) == skip {
            return Ok(());
        }
        zip.add_directory(format!("{name}/"), options).map_err(|e| e.to_string())?;
        let mut children: Vec<_> = std::fs::read_dir(path)
            .map_err(|e| e.to_string())?
            .filter_map(Result::ok)
            .collect();
        children.sort_by_key(|c| c.file_name());
        for child in children {
            let child_name = format!("{name}/{}", child.file_name().to_string_lossy());
            add_path(zip, &child.path(), &child_name, skip)?;
        }
    } else if meta.is_file() {
        let options = options.large_file(meta.len() > u32::MAX as u64);
        zip.start_file(name, options).map_err(|e| e.to_string())?;
        let mut source = File::open(path).map_err(|e| e.to_string())?;
        std::io::copy(&mut source, zip).map_err(|e| e.to_string())?;
    }
    // Links and other special entries are skipped.
    Ok(())
}

fn write_zip(dest: &Path, paths: &[PathBuf]) -> Result<(), String> {
    let file = File::create(dest).map_err(|e| e.to_string())?;
    // The archive's own entry folder, left out wherever it turns up.
    let skip = dest.parent().map(crate::file_drop::path_key).unwrap_or_default();
    let mut zip = zip::ZipWriter::new(std::io::BufWriter::new(file));
    let mut taken = HashSet::new();
    for path in paths {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "item".to_string());
        add_path(&mut zip, path, &unique_name(&mut taken, &name), &skip)?;
    }
    zip.finish().map_err(|e| e.to_string())?;
    Ok(())
}

/// Compresses the items into one ZIP file and puts it on their shelf.
/// One item is named after itself; several get `archive_name` (translated
/// by the page). Returns the archive's file name.
#[tauri::command]
pub async fn compress_items(
    app: AppHandle,
    file_list: State<'_, FileList>,
    ids: Vec<u64>,
    archive_name: String,
) -> Result<String, String> {
    let (paths, shelf) = usable_paths(&file_list, &ids)?;
    let stem = match paths.as_slice() {
        [only] => only
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| archive_name.clone()),
        _ => archive_name,
    };
    let folder = crate::commands::file_ops::new_entry_folder("zip-")?;
    let dest = folder.join(crate::commands::file_ops::clean_file_name(&stem, "zip"));
    let written = {
        let dest = dest.clone();
        tauri::async_runtime::spawn_blocking(move || write_zip(&dest, &paths))
            .await
            .map_err(|e| e.to_string())?
    };
    if let Err(e) = written {
        warn!("Could not create the archive: {}", e);
        let _ = std::fs::remove_dir_all(&folder);
        return Err(e);
    }
    let mut file = FileMetadata::from_path(dest.clone()).ok_or("The archive disappeared")?;
    file.shelf = shelf;
    let name = file.name.clone();
    file_list.lock().map_err(|_| "Failed to acquire lock".to_string())?.push(file);
    app.emit("files_updated", ()).map_err(|e| e.to_string())?;
    Ok(name)
}

// ------------------------------------------------------------------ Copy to folder

/// Double-NUL-terminated path list, as SHFileOperation wants it.
fn path_list(paths: &[PathBuf]) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    let mut out = Vec::new();
    for path in paths {
        out.extend(path.as_os_str().encode_wide());
        out.push(0);
    }
    out.push(0);
    out
}

/// Asks for a folder with Windows' own folder picker. None when cancelled.
fn pick_folder(owner: Option<windows::Win32::Foundation::HWND>, title: &str) -> Result<Option<PathBuf>, String> {
    use windows::core::HSTRING;
    use windows::Win32::System::Com::{CoCreateInstance, CoTaskMemFree, CLSCTX_INPROC_SERVER};
    use windows::Win32::UI::Shell::{
        FileOpenDialog, IFileOpenDialog, FOS_FORCEFILESYSTEM, FOS_PICKFOLDERS, SIGDN_FILESYSPATH,
    };
    const ERROR_CANCELLED: i32 = 0x800704C7u32 as i32;

    // COM is set up by the caller (`on_sta_thread`), for the copy as well.
    unsafe {
        let dialog: IFileOpenDialog =
            CoCreateInstance(&FileOpenDialog, None, CLSCTX_INPROC_SERVER).map_err(|e| e.to_string())?;
        let options = dialog.GetOptions().map_err(|e| e.to_string())?;
        dialog
            .SetOptions(options | FOS_PICKFOLDERS | FOS_FORCEFILESYSTEM)
            .map_err(|e| e.to_string())?;
        if !title.is_empty() {
            let _ = dialog.SetTitle(&HSTRING::from(title));
        }
        if let Err(e) = dialog.Show(owner) {
            return if e.code().0 == ERROR_CANCELLED { Ok(None) } else { Err(e.to_string()) };
        }
        let item = dialog.GetResult().map_err(|e| e.to_string())?;
        let name = item.GetDisplayName(SIGDN_FILESYSPATH).map_err(|e| e.to_string())?;
        let path = name.to_string().map_err(|e| e.to_string());
        CoTaskMemFree(Some(name.0 as *const _));
        Ok(Some(PathBuf::from(path?)))
    }
}

/// Runs `f` on a new thread in a single-threaded COM apartment, as the
/// folder picker and the shell copy engine need. A pooled thread may already
/// be in the multithreaded apartment, so a fresh one is used.
fn on_sta_thread<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> Result<T, String> {
    use windows::Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED};
    std::thread::spawn(move || unsafe {
        let init = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        let result = f();
        if init.is_ok() {
            CoUninitialize();
        }
        result
    })
    .join()
    .map_err(|_| "The folder copy stopped unexpectedly".to_string())
}

/// Copies files with Windows' copy engine: its progress window, its
/// "replace or skip" questions, and Ctrl+Z in Explorer to undo.
fn shell_copy(owner: Option<windows::Win32::Foundation::HWND>, paths: &[PathBuf], to: &Path) -> Result<bool, String> {
    use windows::core::PCWSTR;
    use windows::Win32::UI::Shell::{
        SHFileOperationW, FOF_ALLOWUNDO, FOF_NOCONFIRMMKDIR, FO_COPY, SHFILEOPSTRUCTW,
    };
    let from = path_list(paths);
    let dest = path_list(&[to.to_path_buf()]);
    let mut op = SHFILEOPSTRUCTW {
        hwnd: owner.unwrap_or_default(),
        wFunc: FO_COPY,
        pFrom: PCWSTR(from.as_ptr()),
        pTo: PCWSTR(dest.as_ptr()),
        fFlags: (FOF_ALLOWUNDO | FOF_NOCONFIRMMKDIR).0 as u16,
        ..Default::default()
    };
    let code = unsafe { SHFileOperationW(&mut op) };
    // Cancelled in Windows' own progress or conflict dialog: some items
    // weren't copied, which the caller reports as such, not as a failure.
    if code != 0 && !op.fAnyOperationsAborted.as_bool() {
        return Err(format!("Windows couldn't copy the items (code {code:#x})"));
    }
    // False when the user cancelled part of it.
    Ok(!op.fAnyOperationsAborted.as_bool())
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CopyResult {
    /// The folder's name, for the message; None when no folder was picked.
    folder: Option<String>,
    count: usize,
    complete: bool,
}

/// Lets the user pick a folder, then copies the items there.
#[tauri::command]
pub async fn copy_items_to_folder(
    window: WebviewWindow,
    file_list: State<'_, FileList>,
    ids: Vec<u64>,
    title: String,
) -> Result<CopyResult, String> {
    let (paths, _) = usable_paths(&file_list, &ids)?;
    // Owned by the calling window, so the dialogs stay above it (it is
    // always on top) and it can't close underneath them.
    let owner = window.hwnd().ok().map(|h| h.0 as isize);
    // The popup hides when another window comes to the front; the dialogs
    // are Holdem's own, but keep it open explicitly while they are up.
    let from_popup = window.label() == "popup";
    if from_popup {
        crate::commands::window_ops::POPUP_KEEP_OPEN.store(true, Ordering::SeqCst);
    }
    let result = tauri::async_runtime::spawn_blocking(move || on_sta_thread(move || {
        let owner = owner.map(|h| windows::Win32::Foundation::HWND(h as _));
        let Some(folder) = pick_folder(owner, &title)? else {
            return Ok(CopyResult { folder: None, count: 0, complete: false });
        };
        let complete = shell_copy(owner, &paths, &folder)?;
        let name = folder
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| folder.to_string_lossy().into_owned());
        Ok(CopyResult { folder: Some(name), count: paths.len(), complete })
    }))
    .await;
    if from_popup {
        crate::commands::window_ops::POPUP_KEEP_OPEN.store(false, Ordering::SeqCst);
    }
    result.map_err(|e| e.to_string())??
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn an_archive_never_contains_itself() {
        let root = std::env::temp_dir().join(format!("holdem_zip_self_{}", uuid::Uuid::new_v4()));
        let entry = root.join("zip-1");
        std::fs::create_dir_all(&entry).expect("folders");
        std::fs::write(root.join("note.txt"), b"hello").expect("write");
        let dest = entry.join("archive.zip");
        write_zip(&dest, std::slice::from_ref(&root)).expect("zip");

        let mut archive = zip::ZipArchive::new(File::open(&dest).expect("open")).expect("read");
        let names: Vec<String> = (0..archive.len())
            .map(|i| archive.by_index(i).expect("entry").name().expect("name").to_string())
            .collect();
        let _ = std::fs::remove_dir_all(&root);
        assert!(names.iter().any(|n| n.ends_with("note.txt")), "{names:?}");
        assert!(!names.iter().any(|n| n.contains("zip-1")), "{names:?}");
    }

    #[test]
    fn duplicate_names_get_numbered() {
        let mut taken = HashSet::new();
        assert_eq!(unique_name(&mut taken, "a.txt"), "a.txt");
        assert_eq!(unique_name(&mut taken, "A.txt"), "A (2).txt");
        assert_eq!(unique_name(&mut taken, "a.txt"), "a (3).txt");
        assert_eq!(unique_name(&mut taken, "README"), "README");
        assert_eq!(unique_name(&mut taken, "README"), "README (2)");
    }

    #[test]
    fn zips_files_and_folders() {
        let dir = std::env::temp_dir().join(format!("holdem_zip_{}", uuid::Uuid::new_v4()));
        let sub = dir.join("folder");
        std::fs::create_dir_all(sub.join("inner")).expect("dirs");
        std::fs::write(dir.join("one.txt"), "one").expect("write");
        std::fs::write(sub.join("inner").join("two.txt"), "two").expect("write");
        let dest = dir.join("out.zip");
        write_zip(&dest, &[dir.join("one.txt"), sub]).expect("zip");

        let mut archive = zip::ZipArchive::new(File::open(&dest).expect("open")).expect("read");
        let mut names: Vec<String> = archive.file_names().map(|n| n.expect("name").into_owned()).collect();
        names.sort();
        assert_eq!(names, ["folder/", "folder/inner/", "folder/inner/two.txt", "one.txt"]);
        let mut text = String::new();
        archive.by_name("folder/inner/two.txt").expect("entry").read_to_string(&mut text).expect("read");
        assert_eq!(text, "two");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn path_lists_end_in_two_nuls() {
        let list = path_list(&[PathBuf::from("a"), PathBuf::from("b")]);
        assert_eq!(list, vec![97, 0, 98, 0, 0]);
    }
}
