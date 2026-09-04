use crate::analytics;
use crate::file::FileMetadata;
use crate::thumbnail::get_thumbnail_base64;
use crate::{DragState, FileList};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use tauri::{AppHandle, Emitter, State};

#[tauri::command]
pub fn add_files(
    app_handle: AppHandle,
    file_list: State<'_, FileList>,
    drag_state: State<'_, Arc<DragState>>,
    files: Vec<String>,
) -> Result<(), String> {
    if !files.is_empty() {
        // Mark the drop before metadata work so shake-to-show does not close the window.
        drag_state.successful_drop.store(true, Ordering::Relaxed);
        drag_state.drag_started.store(false, Ordering::Relaxed);
    }

    let mut list = file_list
        .lock()
        .map_err(|_| "Failed to acquire lock".to_string())?;

    let mut changed = false;
    for path_str in files.iter() {
        let path = PathBuf::from(path_str);
        if path.exists() {
            let metadata = path.metadata().map_err(|e| e.to_string())?;

            // Directories store size 0 — no recursive walk (instant drops).
            let size = if metadata.is_dir() { 0 } else { metadata.len() };

            let file = FileMetadata {
                id: list.len() as u64,
                name: path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown")
                    .to_string(),
                path: path.clone(),
                size,
                file_type: if metadata.is_dir() {
                    "folder".to_string()
                } else {
                    path.extension()
                        .and_then(|ext| ext.to_str())
                        .unwrap_or("unknown")
                        .to_string()
                },
            };
            // Avoid duplicates
            if !list.iter().any(|f| f.path == file.path) {
                list.push(file);
                changed = true;
            }
        }
    }
    drop(list);

    // Single emit per batch — N emits for N files thrashed the frontend.
    if changed {
        app_handle
            .emit("files_updated", ())
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

fn sanitize_filename(name: &str) -> String {
    let sanitized: String = name
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c if c.is_control() => '_',
            other => other,
        })
        .collect();
    if sanitized.trim().is_empty() {
        "unnamed".to_string()
    } else {
        sanitized
    }
}

fn drop_folder() -> Result<PathBuf, String> {
    let folder_name = chrono::Local::now().format("%Y%m%d").to_string();
    let drop_folder = std::env::temp_dir().join("holdem_drops").join(&folder_name);
    std::fs::create_dir_all(&drop_folder)
        .map_err(|e| format!("Failed to create drop folder: {}", e))?;
    Ok(drop_folder)
}

/// Unique leaf name: <stem>_<HHMMSSmmm>_<rand8>.<ext> — avoids the old
/// second-resolution collisions when one physical drop fires multiple events.
fn unique_file_name(stem: &str, ext: &str) -> String {
    let stem = sanitize_filename(stem);
    let stem = stem.trim();
    let stem = if stem.is_empty() { "unnamed" } else { stem };
    // Truncate very long stems (Outlook subjects, URLs) to keep paths valid.
    let stem: String = stem.chars().take(80).collect();
    let ts = chrono::Local::now().format("%H%M%S%3f").to_string();
    let rand: String = uuid::Uuid::new_v4().to_string()[..8].to_string();
    if ext.is_empty() {
        format!("{}_{}_{}", stem, ts, rand)
    } else {
        format!("{}_{}_{}.{}", stem, ts, rand, sanitize_filename(ext))
    }
}

fn unique_path_in(folder: &std::path::Path, file_name: &str) -> PathBuf {
    let mut p = folder.join(file_name);
    // Belt-and-braces in the astronomically unlikely case of a collision.
    let mut n = 1;
    while p.exists() {
        let stem = std::path::Path::new(file_name)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("drop");
        let ext = std::path::Path::new(file_name)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        let alt = if ext.is_empty() {
            format!("{}_{}", stem, n)
        } else {
            format!("{}_{}.{}", stem, n, ext)
        };
        p = folder.join(alt);
        n += 1;
    }
    p
}

/// Write raw bytes to today's drop folder with a unique name.
/// Used by the native drop target for virtual files (FileGroupDescriptor),
/// bitmaps (CF_DIB/PNG) and fetched web images.
pub fn save_bytes_to_drop_folder(
    bytes: &[u8],
    suggested_name: &str,
    default_ext: &str,
) -> Result<String, String> {
    use std::io::Write;
    let folder = drop_folder()?;
    let (stem, ext) = split_stem_ext(suggested_name, default_ext);
    let name = unique_file_name(&stem, &ext);
    let path = unique_path_in(&folder, &name);
    let mut file = std::fs::File::create(&path).map_err(|e| e.to_string())?;
    file.write_all(bytes).map_err(|e| e.to_string())?;
    Ok(path.to_string_lossy().to_string())
}

fn split_stem_ext(suggested: &str, default_ext: &str) -> (String, String) {
    let clean = suggested.trim();
    if clean.is_empty() {
        return ("drop".to_string(), default_ext.to_string());
    }
    let p = std::path::Path::new(clean);
    let stem = p
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("drop")
        .to_string();
    let ext = p
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or(default_ext)
        .to_string();
    (stem, ext)
}

/// Save a URL that could not be fetched (auth-walled, blob:, non-image)
/// as a tiny `.url` placeholder so the drop is never silently lost.
pub fn save_url_placeholder(url: &str) -> Result<String, String> {
    let body = format!("[InternetShortcut]\nURL={}\n", url);
    save_bytes_to_drop_folder(body.as_bytes(), "link", "url")
}

#[tauri::command]
pub fn save_pasted_text(
    text: String,
    extension: String,
    original_name: Option<String>,
) -> Result<String, String> {
    use std::io::Write;
    let folder = drop_folder()?;
    let ext = if extension.trim().is_empty() {
        "txt"
    } else {
        extension.trim()
    };
    let file_name = if let Some(name) = original_name.as_deref().filter(|n| !n.trim().is_empty()) {
        // Even explicit names get uniquified to survive duplicate drop events.
        let (stem, ext2) = split_stem_ext(name, ext);
        unique_file_name(&stem, &ext2)
    } else {
        unique_file_name("note", ext)
    };
    let new_path = unique_path_in(&folder, &file_name);

    let mut file = std::fs::File::create(&new_path).map_err(|e| e.to_string())?;
    file.write_all(text.as_bytes()).map_err(|e| e.to_string())?;

    Ok(new_path.to_string_lossy().to_string())
}

#[tauri::command]
pub fn save_pasted_data_base64(
    data_base64: String,
    extension: String,
    original_name: Option<String>,
) -> Result<String, String> {
    use base64::{engine::general_purpose, Engine as _};

    // Accept both raw base64 and full `data:...;base64,...` URLs (frontend
    // blob: forwarder sends raw, browser <img> sends full).
    let b64 = data_base64
        .split_once(',')
        .map(|(_, rest)| rest)
        .unwrap_or(&data_base64);
    let b64 = b64.trim();
    let bytes = general_purpose::STANDARD
        .decode(b64)
        .map_err(|e| e.to_string())?;

    let ext = if extension.trim().is_empty() {
        "png"
    } else {
        extension.trim()
    };
    let file_name = if let Some(name) = original_name.as_deref().filter(|n| !n.trim().is_empty()) {
        let (stem, ext2) = split_stem_ext(name, ext);
        unique_file_name(&stem, &ext2)
    } else {
        unique_file_name("drop", ext)
    };
    let folder = drop_folder()?;
    let new_path = unique_path_in(&folder, &file_name);

    use std::io::Write;
    let mut file = std::fs::File::create(&new_path).map_err(|e| e.to_string())?;
    file.write_all(&bytes).map_err(|e| e.to_string())?;

    Ok(new_path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn download_image_to_shelf(
    url: String,
    referer: Option<String>,
) -> Result<String, String> {
    if url.starts_with("blob:") {
        // Opaque to backend by design — frontend must fetch it where the
        // drag originated. Save a placeholder so the drop isn't lost.
        return save_url_placeholder(&url);
    }

    let suggested = if let Ok(parsed_url) = url::Url::parse(&url) {
        let path_segment = parsed_url
            .path_segments()
            .and_then(|mut segs| segs.next_back())
            .unwrap_or("");
        let path_obj = std::path::Path::new(path_segment);
        let extracted_ext = path_obj
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        // Strip query-driven junk; keep real filenames, else generic.
        if !path_segment.is_empty()
            && !extracted_ext.is_empty()
            && extracted_ext.len() <= 5
        {
            sanitize_filename(path_segment)
        } else {
            "download".to_string()
        }
    } else {
        "download".to_string()
    };

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) Holdem/1.0")
        .build()
        .map_err(|e| e.to_string())?;
    let mut req = client
        .get(&url)
        .header(reqwest::header::ACCEPT, "image/*,*/*;q=0.8");
    if let Some(r) = referer.as_deref().filter(|s| !s.is_empty()) {
        req = req.header(reqwest::header::REFERER, r.to_string());
    }
    let mut response = req.send().await.map_err(|e| e.to_string())?;
    let status = response.status();
    if !status.is_success() {
        // Auth-walled / hotlink-protected (401/403) — keep URL, not HTML error page.
        return save_url_placeholder(&url);
    }
    // 25 MB cap — don't buffer huge videos disguised as images.
    // Check declared length up front, then stream with a running cap so a
    // lying/missing content-length can't OOM us in `bytes()`.
    const MAX_BYTES: usize = 25 * 1024 * 1024;
    if let Some(len) = response.content_length() {
        if len > MAX_BYTES as u64 {
            return Err("Downloaded file exceeds 25 MB limit".to_string());
        }
    }
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|e| e.to_string())? {
        bytes.extend_from_slice(&chunk);
        if bytes.len() > MAX_BYTES {
            return Err("Downloaded file exceeds 25 MB limit".to_string());
        }
    }
    let ext = if content_type.starts_with("image/png") {
        "png"
    } else if content_type.starts_with("image/jpeg") {
        "jpg"
    } else if content_type.starts_with("image/webp") {
        "webp"
    } else if content_type.starts_with("image/gif") {
        "gif"
    } else if content_type.starts_with("image/svg") {
        "svg"
    } else if content_type.starts_with("image/") {
        content_type
            .split('/')
            .nth(1)
            .and_then(|s| s.split(';').next())
            .unwrap_or("png")
    } else if content_type.is_empty() || content_type.starts_with("application/octet-stream") {
        // No content-type — fall back to URL extension guess.
        std::path::Path::new(&suggested)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("png")
    } else {
        // HTML login wall etc. — don't save corrupt file.
        return save_url_placeholder(&url);
    };

    save_bytes_to_drop_folder(&bytes, &suggested, ext)
}
#[tauri::command]
pub fn remove_files(
    app_handle: AppHandle,
    file_list: State<'_, FileList>,
    file_ids: Vec<u64>,
) -> Result<(), String> {
    let mut list = file_list
        .lock()
        .map_err(|_| "Failed to acquire lock".to_string())?;

    let mut removed_files = Vec::new();
    for file_id in file_ids {
        if let Some(pos) = list.iter().position(|f| f.id == file_id) {
            let file_name = list[pos].name.clone();
            list.remove(pos);
            removed_files.push(file_name);
            app_handle
                .emit("files_updated", ())
                .map_err(|e| e.to_string())?;
        } else {
            return Err(format!("File with ID {} not found", file_id));
        }
    }

    // Send analytics events for removed files (fire and forget)
    let app_handle_clone = app_handle.clone();
    tauri::async_runtime::spawn(async move {
        for _ in removed_files {
            let _ = analytics::send_file_removed_event(&app_handle_clone).await;
        }
    });

    Ok(())
}

#[tauri::command]
pub fn get_files(file_list: State<'_, FileList>) -> Result<Vec<FileMetadata>, String> {
    let list = file_list
        .lock()
        .map_err(|_| "Failed to acquire lock".to_string())?;
    Ok(list.clone())
}

#[tauri::command]
pub fn rename_file(
    app_handle: AppHandle,
    file_list: State<'_, FileList>,
    file_id: u64,
    new_name: String,
) -> Result<(), String> {
    let mut list = file_list
        .lock()
        .map_err(|_| "Failed to acquire lock".to_string())?;
    if let Some(file) = list.iter_mut().find(|f| f.id == file_id) {
        file.name = new_name.clone();

        // Send analytics event for file rename (fire and forget)
        let app_handle_clone = app_handle.clone();
        tauri::async_runtime::spawn(async move {
            let _ = analytics::send_file_renamed_event(&app_handle_clone).await;
        });

        app_handle
            .emit("files_updated", ())
            .map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err("File not found".to_string())
    }
}

#[tauri::command]
pub fn clear_files(app_handle: AppHandle, file_list: State<'_, FileList>) -> Result<(), String> {
    let mut list = file_list
        .lock()
        .map_err(|_| "Failed to acquire lock".to_string())?;

    let num_files = list.len();
    list.clear();

    // Send analytics event for clearing files (fire and forget)
    let app_handle_clone = app_handle.clone();
    tauri::async_runtime::spawn(async move {
        let _ = analytics::send_files_cleared_event(&app_handle_clone, num_files).await;
    });

    app_handle
        .emit("files_updated", ())
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn refresh_file_list(
    app_handle: AppHandle,
    file_list: State<'_, FileList>,
) -> Result<(), String> {
    let mut list = file_list
        .lock()
        .map_err(|_| "Failed to acquire lock".to_string())?;
    let mut needs_update = false;

    // Create a new list to store valid files
    let mut new_list = Vec::new();

    for file in list.iter() {
        if file.path.exists() {
            new_list.push(file.clone());
        } else {
            needs_update = true;
        }
    }

    if needs_update {
        *list = new_list;
        app_handle
            .emit("files_updated", ())
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub fn get_file_icon_base64(
    _app_handle: AppHandle,
    _file_list: State<'_, FileList>,
    file_path: &str,
) -> Result<String, String> {
    get_thumbnail_base64(std::path::Path::new(file_path)).map_err(|e| e.to_string())
}
