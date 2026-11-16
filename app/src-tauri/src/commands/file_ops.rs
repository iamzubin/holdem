use crate::analytics;
use crate::file::FileMetadata;
use crate::presence::{self, Presence};
use crate::thumbnail::get_thumbnail_base64;
use crate::{DragState, FileList, RemovedStash};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, State};

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
        // Missing/unreadable paths are skipped instead of failing the batch.
        // Ids come from a global counter, never the list length: a length-based
        // id repeats after a removal and makes two shelf entries share an id.
        let Some(file) = FileMetadata::from_path(PathBuf::from(path_str)) else {
            continue;
        };
        // Avoid duplicates on the same shelf
        if !list.iter().any(|f| f.path == file.path && f.shelf == file.shelf) {
            list.push(file);
            changed = true;
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

/// A new, empty folder for one dropped item: `<drops>/<YYYYMMDD>/<prefix><HHMMSSmmm>_<rand8>`.
/// Each item gets its own folder so its file can keep a clean, readable name
/// (the attachment's real name, "github.com.url", the note's first line)
/// without colliding with anything else dropped the same day.
pub(crate) fn new_entry_folder(prefix: &str) -> Result<PathBuf, SaveError> {
    let day = chrono::Local::now()
        .format(crate::file_drop::DROP_FOLDER_DATE_FORMAT)
        .to_string();
    let stamp = chrono::Local::now().format("%H%M%S%3f").to_string();
    let rand = &uuid::Uuid::new_v4().simple().to_string()[..8];
    let folder = crate::file_drop::drops_root()
        .join(day)
        .join(format!("{prefix}{stamp}_{rand}"));
    std::fs::create_dir_all(&folder)?;
    Ok(folder)
}

/// File name for saved content: sanitized, stem capped at 80 characters
/// (Outlook subjects and URLs get long) so paths stay valid.
pub(crate) fn clean_file_name(stem: &str, ext: &str) -> String {
    let stem = sanitize_filename(stem);
    let stem: String = stem.trim().trim_end_matches('.').chars().take(80).collect();
    let stem = if stem.trim().is_empty() { "unnamed".to_string() } else { stem };
    // CON, NUL, COM1… name devices, not files: "nul.pdf" would write
    // nowhere and still report success.
    let stem = if is_reserved_name(&stem) { format!("_{stem}") } else { stem };
    if ext.is_empty() {
        stem
    } else {
        format!("{}.{}", stem, sanitize_filename(ext))
    }
}

/// True for Windows' reserved device names, alone or before a dot.
fn is_reserved_name(stem: &str) -> bool {
    let base = stem.split('.').next().unwrap_or("").trim().to_ascii_uppercase();
    matches!(base.as_str(), "CON" | "PRN" | "AUX" | "NUL" | "CONIN$" | "CONOUT$")
        || ((base.starts_with("COM") || base.starts_with("LPT"))
            && base.len() == 4
            && base.as_bytes()[3].is_ascii_digit()
            && base.as_bytes()[3] != b'0')
}

/// Why saving dropped content failed. The I/O error is kept so the user can
/// be told "disk full" or "access denied" (see `drop_report::save_issue`).
#[derive(Debug, thiserror::Error)]
pub enum SaveError {
    #[error("{0}")]
    Io(#[from] std::io::Error),
    /// Not a web link, so no shortcut is made (mailto:, file:, javascript:).
    #[error("{0}")]
    NotALink(String),
}

impl From<SaveError> for String {
    fn from(error: SaveError) -> Self {
        error.to_string()
    }
}

fn save_in_new_entry(bytes: &[u8], file_name: &str, prefix: &str) -> Result<String, SaveError> {
    let path = new_entry_folder(prefix)?.join(file_name);
    std::fs::write(&path, bytes)?;
    Ok(path.to_string_lossy().to_string())
}

/// Saves bytes that only exist in the drop (virtual files from Outlook or
/// chat apps, bitmaps, fetched web images) under their suggested name.
pub fn save_bytes_to_drop_folder(
    bytes: &[u8],
    suggested_name: &str,
    default_ext: &str,
) -> Result<String, SaveError> {
    let (stem, ext) = split_stem_ext(suggested_name, default_ext);
    save_in_new_entry(bytes, &clean_file_name(&stem, &ext), "")
}

/// Name for a text snippet: its first non-empty line, shortened. Falls back
/// to "Note" for text that is all whitespace or symbols.
fn note_title(text: &str) -> String {
    let first = text.lines().map(str::trim).find(|l| !l.is_empty()).unwrap_or("");
    let short: String = first.chars().take(40).collect();
    if !short.chars().any(char::is_alphanumeric) {
        return "Note".to_string();
    }
    let cleaned = sanitize_filename(short.trim());
    if cleaned.chars().any(char::is_alphanumeric) {
        cleaned.trim().to_string()
    } else {
        "Note".to_string()
    }
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

/// Extension for a known image MIME fragment — either an HTTP
/// `Content-Type` value or a `data:` URL header. `None` for anything else.
pub fn ext_from_image_mime(fragment: &str) -> Option<&'static str> {
    if fragment.contains("image/png") {
        Some("png")
    } else if fragment.contains("image/jpeg") || fragment.contains("image/jpg") {
        Some("jpg")
    } else if fragment.contains("image/webp") {
        Some("webp")
    } else if fragment.contains("image/gif") {
        Some("gif")
    } else if fragment.contains("image/svg") {
        Some("svg")
    } else {
        None
    }
}

/// Save a URL that could not be fetched (auth-walled, blob:, non-image)
/// as a tiny `.url` placeholder so the drop is never silently lost.
pub fn save_url_placeholder(url: &str) -> Result<String, SaveError> {
    let url = shortcut_target(url).map_err(SaveError::NotALink)?;
    let body = format!("[InternetShortcut]\r\nURL={}\r\n", url);
    save_bytes_to_drop_folder(body.as_bytes(), &placeholder_name(&url), "url")
}

/// The URL to write into a `.url` file: parsed and re-serialised (which
/// drops any CR/LF that could inject extra shortcut lines such as an
/// `IconFile` on a network share), and limited to web schemes.
fn shortcut_target(url: &str) -> Result<String, String> {
    let parsed = url::Url::parse(url.trim()).map_err(|e| format!("Not a valid link: {}", e))?;
    // Not blob: either: it lives only inside the page it came from, so a
    // shortcut to it would open nothing.
    if !matches!(parsed.scheme(), "http" | "https") {
        return Err(format!("Unsupported link type: {}", parsed.scheme()));
    }
    let clean = parsed.as_str().replace(['\r', '\n'], "");
    Ok(clean)
}

/// Tags a saved file as downloaded from the internet (the "Mark of the
/// Web"), so Windows SmartScreen and Office Protected View treat it the way
/// they would a browser download. Best effort: drives without alternate data
/// streams (FAT, exFAT) simply can't carry the mark.
pub fn mark_from_internet(path: &std::path::Path, host_url: Option<&str>, referrer: Option<&str>) {
    let clean = |s: &str| s.replace(['\r', '\n'], "");
    let mut zone = String::from("[ZoneTransfer]\r\nZoneId=3\r\n");
    if let Some(r) = referrer.filter(|r| r.starts_with("http")) {
        zone.push_str(&format!("ReferrerUrl={}\r\n", clean(r)));
    }
    if let Some(h) = host_url.filter(|h| h.starts_with("http")) {
        zone.push_str(&format!("HostUrl={}\r\n", clean(h)));
    }
    let stream = format!("{}:Zone.Identifier", path.display());
    if let Err(e) = std::fs::write(&stream, zone) {
        tracing::warn!("Could not mark {:?} as downloaded: {}", path, e);
    }
}

/// Names a link file after its site (`github.com.url`), so several dropped
/// links are told apart on the shelf; `link.url` when there is no host.
fn placeholder_name(url: &str) -> String {
    let host = url::Url::parse(url)
        .ok()
        .and_then(|u| u.host_str().map(|h| h.trim_start_matches("www.").to_string()))
        .filter(|h| !h.is_empty());
    match host {
        Some(host) => format!("{}.url", sanitize_filename(&host)),
        None => "link.url".to_string(),
    }
}

/// Resolve the suggested filename for pasted content: an explicit
/// `original_name` wins, otherwise the per-kind fallback stem. Either way
/// `save_bytes_to_drop_folder` uniquifies it against duplicate events.
fn paste_suggested_name(
    original_name: Option<String>,
    fallback_stem: &str,
) -> String {
    original_name
        .as_deref()
        .filter(|n| !n.trim().is_empty())
        .unwrap_or(fallback_stem)
        .to_string()
}

pub fn save_pasted_text(
    text: String,
    extension: String,
    original_name: Option<String>,
) -> Result<String, SaveError> {
    let ext = if extension.trim().is_empty() {
        "txt"
    } else {
        extension.trim()
    };
    let stem = paste_suggested_name(original_name, &note_title(&text));
    save_in_new_entry(
        text.as_bytes(),
        &clean_file_name(&stem, ext),
        crate::file::NOTE_DIR_PREFIX,
    )
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
    Ok(save_bytes_to_drop_folder(
        &bytes,
        &paste_suggested_name(original_name, "drop"),
        ext,
    )?)
}

/// Why a dropped image URL didn't end up on the shelf.
#[derive(Debug, thiserror::Error)]
pub enum FetchError {
    /// Offline, timed out, too large: worth keeping the link instead.
    #[error("{0}")]
    Download(String),
    /// The download worked but couldn't be saved (disk full, blocked).
    #[error("{0}")]
    Save(#[from] SaveError),
}

impl From<String> for FetchError {
    fn from(error: String) -> Self {
        Self::Download(error)
    }
}

/// What a dropped image URL became on the shelf.
pub enum Fetched {
    /// The image itself.
    Image(String),
    /// Only a link to it: it needs a sign-in, isn't an image, or the page
    /// holds it (blob:).
    Link(String),
}

pub async fn download_image_to_shelf(
    url: String,
    referer: Option<String>,
) -> Result<Fetched, FetchError> {
    let link = |url: &str| save_url_placeholder(url).map(Fetched::Link).map_err(FetchError::Save);
    let suggested = if let Ok(parsed_url) = url::Url::parse(&url) {
        // Decoded: "My%20Photo.jpg" is saved as "My Photo.jpg".
        let path_segment = percent_decode(
            parsed_url
                .path_segments()
                .and_then(|mut segs| segs.next_back())
                .unwrap_or(""),
        );
        let path_segment = path_segment.as_str();
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
    // Like browsers' default policy across sites: send only the page's
    // origin, never its path or query.
    if let Some(origin) = referer
        .as_deref()
        .and_then(|r| url::Url::parse(r).ok())
        .map(|u| u.origin().ascii_serialization())
        .filter(|o| o != "null")
    {
        req = req.header(reqwest::header::REFERER, origin);
    }
    let mut response = req.send().await.map_err(|e| e.to_string())?;
    let status = response.status();
    if !status.is_success() {
        // Auth-walled / hotlink-protected (401/403) — keep URL, not HTML error page.
        return link(&url);
    }
    // 25 MB cap — don't buffer huge videos disguised as images.
    // Check declared length up front, then stream with a running cap so a
    // lying/missing content-length can't OOM us in `bytes()`.
    const MAX_BYTES: usize = 25 * 1024 * 1024;
    if let Some(len) = response.content_length() {
        if len > MAX_BYTES as u64 {
            return Err(FetchError::Download("Downloaded file exceeds 25 MB limit".to_string()));
        }
    }
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    // Decide from the headers, before reading the body, so dropping a link to
    // a web page saves a shortcut without downloading the page.
    let ext: &str = match ext_from_image_mime(&content_type) {
        Some(known) => known,
        None if content_type.starts_with("image/") => content_type
            .split('/')
            .nth(1)
            .and_then(|s| s.split(';').next())
            .unwrap_or("png"),
        None
            if content_type.is_empty()
                || content_type.starts_with("application/octet-stream") =>
        {
            // No content-type — fall back to URL extension guess.
            std::path::Path::new(&suggested)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("png")
        }
        None => {
            // A web page, HTML login wall etc. — keep the link instead.
            return link(&url);
        }
    };

    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|e| e.to_string())? {
        bytes.extend_from_slice(&chunk);
        if bytes.len() > MAX_BYTES {
            return Err(FetchError::Download("Downloaded file exceeds 25 MB limit".to_string()));
        }
    }

    // Keep it only if the bytes really are an image: a server can call
    // anything image/* or octet-stream, and an .exe must never land on the
    // shelf looking like a picture.
    let ext = match image::guess_format(&bytes) {
        Ok(format) => format.extensions_str().first().copied().unwrap_or(ext),
        Err(_) if ext == "svg" && looks_like_svg(&bytes) => "svg",
        Err(_) => return link(&url),
    };
    let (stem, _) = split_stem_ext(&suggested, ext);
    let path = save_bytes_to_drop_folder(&bytes, &stem, ext)?;
    mark_from_internet(std::path::Path::new(&path), Some(&url), referer.as_deref());
    Ok(Fetched::Image(path))
}

/// Decodes %XX escapes in a URL path segment (as UTF-8, lossily).
fn percent_decode(segment: &str) -> String {
    let bytes = segment.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let hex = |b: u8| (b as char).to_digit(16);
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(h), Some(l)) = (hex(bytes[i + 1]), hex(bytes[i + 2])) {
                out.push((h * 16 + l) as u8);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn looks_like_svg(bytes: &[u8]) -> bool {
    let head = String::from_utf8_lossy(&bytes[..bytes.len().min(1024)]).to_ascii_lowercase();
    head.contains("<svg")
}
/// Takes the given ids off the shelf (never touching the files themselves)
/// and keeps them for [`restore_removed`]. Ids that are already gone, e.g.
/// because another window cleared the shelf, are skipped.
#[tauri::command]
pub fn remove_files(
    app_handle: AppHandle,
    file_list: State<'_, FileList>,
    stash: State<'_, RemovedStash>,
    file_ids: Vec<u64>,
) -> Result<usize, String> {
    let mut list = file_list
        .lock()
        .map_err(|_| "Failed to acquire lock".to_string())?;
    let (removed, kept): (Vec<FileMetadata>, Vec<FileMetadata>) =
        list.drain(..).partition(|f| file_ids.contains(&f.id));
    *list = kept;
    let count = removed.len();
    if count > 0 {
        // Into Undo before the list is released, so the drop-folder cleanup
        // never sees these items in neither place.
        crate::file_drop::mark_left_shelf(removed.iter().map(|f| f.path.as_path()));
        stash_removed(&stash, removed);
    }
    drop(list);
    tracing::info!("Removed {} of {} requested item(s) from the shelf", count, file_ids.len());
    if count > 0 {
        app_handle
            .emit("files_updated", ())
            .map_err(|e| e.to_string())?;
        send_count_event(&app_handle, "files_removed", count);
    }
    Ok(count)
}

#[tauri::command]
pub fn get_files(file_list: State<'_, FileList>) -> Result<Vec<FileMetadata>, String> {
    let list = file_list
        .lock()
        .map_err(|_| "Failed to acquire lock".to_string())?;
    Ok(list.clone())
}

#[tauri::command]
pub fn clear_files(
    app_handle: AppHandle,
    file_list: State<'_, FileList>,
    stash: State<'_, RemovedStash>,
) -> Result<usize, String> {
    // Only the active shelf is cleared, and its pinned items stay.
    let shelf = crate::shelves::active_id();
    let count = {
        let mut list = file_list
            .lock()
            .map_err(|_| "Failed to acquire lock".to_string())?;
        let (kept, removed): (Vec<FileMetadata>, Vec<FileMetadata>) =
            list.drain(..).partition(|f| f.pinned || f.shelf != shelf);
        *list = kept;
        let count = removed.len();
        // Nothing cleared: keep the last Remove or Clear undoable.
        if count > 0 {
            // Into Undo before the list is released (see remove_files).
            crate::file_drop::mark_left_shelf(removed.iter().map(|f| f.path.as_path()));
            stash_removed(&stash, removed);
        }
        count
    };
    tracing::info!("Cleared {} item(s) from the shelf", count);
    if count == 0 {
        return Ok(0);
    }
    app_handle
        .emit("files_updated", ())
        .map_err(|e| e.to_string())?;
    send_count_event(&app_handle, "files_cleared", count);
    Ok(count)
}

/// Undoes the last Remove or Clear. Entries go back in their original order
/// (ids only ever increase); ones already back on the shelf are skipped.
/// Returns how many came back. A shelf check follows, so an entry whose file
/// was deleted in the meantime leaves again, with the usual message.
#[tauri::command]
pub fn restore_removed(
    app_handle: AppHandle,
    file_list: State<'_, FileList>,
    stash: State<'_, RemovedStash>,
) -> Result<usize, String> {
    // The list first, then the stash (as in the drop cleanup), so a cleanup
    // running now sees the items in one of the two.
    let mut list = file_list
        .lock()
        .map_err(|_| "Failed to acquire lock".to_string())?;
    let stashed = std::mem::take(&mut *stash.lock().map_err(|_| "Failed to acquire lock")?);
    let before = list.len();
    let shelves = app_handle.state::<crate::shelves::Shelves>();
    for mut file in stashed {
        // Its shelf may have been deleted since: bring it back to the active one.
        if !shelves.lock().is_ok_and(|s| s.contains(file.shelf)) {
            file.shelf = crate::shelves::active_id();
        }
        if !list.iter().any(|f| f.path == file.path && f.shelf == file.shelf) {
            list.push(file);
        }
    }
    list.sort_by_key(|f| f.id);
    let restored = list.len() - before;
    tracing::info!("Undo brought back {} item(s)", restored);
    drop(list);
    if restored > 0 {
        app_handle
            .emit("files_updated", ())
            .map_err(|e| e.to_string())?;
        tauri::async_runtime::spawn(refresh_shelf(app_handle));
    }
    Ok(restored)
}

/// Shelf message for entries whose file was moved or deleted outside Holdem.
pub(crate) fn notify_missing(app_handle: &AppHandle, count: usize) {
    if count == 0 {
        return;
    }
    let payload = serde_json::json!({ "kind": "missing", "count": count });
    let _ = app_handle.emit("shelf-notice", payload);
}

fn stash_removed(stash: &RemovedStash, removed: Vec<FileMetadata>) {
    if let Ok(mut stash) = stash.lock() {
        *stash = removed;
    }
}

/// One analytics event carrying only a count (fire and forget).
fn send_count_event(app_handle: &AppHandle, event: &'static str, count: usize) {
    let app_handle = app_handle.clone();
    tauri::async_runtime::spawn(async move {
        let _ = analytics::send_analytics_event(
            &app_handle,
            event,
            Some(vec![("count", serde_json::Value::from(count))]),
        )
        .await;
    });
}

/// Set while a shelf check runs; a request during one asks for a rerun.
static REFRESHING: AtomicBool = AtomicBool::new(false);
static REFRESH_AGAIN: AtomicBool = AtomicBool::new(false);
/// How long a check waits for drives and network shares to answer.
const REFRESH_TIMEOUT: Duration = Duration::from_secs(3);
/// Until this time, items whose file is gone leave the shelf without the
/// "moved or deleted" message: the user just moved them out (Shift-drag).
static QUIET_UNTIL: std::sync::Mutex<Option<std::time::Instant>> = std::sync::Mutex::new(None);

/// Called after a drag out of the shelf was dropped somewhere.
pub fn expect_moved_out(for_how_long: Duration) {
    if let Ok(mut until) = QUIET_UNTIL.lock() {
        *until = Some(std::time::Instant::now() + for_how_long);
    }
}

fn removals_expected() -> bool {
    QUIET_UNTIL
        .lock()
        .ok()
        .and_then(|until| *until)
        .is_some_and(|until| std::time::Instant::now() < until)
}

/// Re-checks every shelf item without blocking any window: deleted files
/// leave the shelf (with a message), items on a drive or share that went
/// offline are greyed out, and ones whose location is back become usable.
pub async fn refresh_shelf(app_handle: AppHandle) -> Result<(), String> {
    if REFRESHING.swap(true, Ordering::SeqCst) {
        REFRESH_AGAIN.store(true, Ordering::SeqCst);
        return Ok(());
    }
    loop {
        let result = loop {
            if let Err(e) = refresh_once(&app_handle).await {
                break Err(e);
            }
            if !REFRESH_AGAIN.swap(false, Ordering::SeqCst) {
                break Ok(());
            }
        };
        REFRESHING.store(false, Ordering::SeqCst);
        // A request that arrived between the last check and now found the
        // run still marked as going, and left it to this one.
        if result.is_err()
            || !REFRESH_AGAIN.swap(false, Ordering::SeqCst)
            || REFRESHING.swap(true, Ordering::SeqCst)
        {
            return result;
        }
    }
}

async fn refresh_once(app_handle: &AppHandle) -> Result<(), String> {
    let file_list = app_handle.state::<FileList>().inner().clone();
    let snapshot: Vec<(u64, PathBuf, bool)> = file_list
        .lock()
        .map_err(|_| "Failed to acquire lock".to_string())?
        .iter()
        .map(|f| (f.id, f.path.clone(), f.available))
        .collect();
    if snapshot.is_empty() {
        return Ok(());
    }

    // File-system calls on an unreachable share can block for many seconds,
    // so they run on a worker thread, bounded by REFRESH_TIMEOUT.
    let checked = tauri::async_runtime::spawn_blocking(move || {
        let paths: Vec<PathBuf> = snapshot.iter().map(|(_, path, _)| path.clone()).collect();
        let found = presence::check(&paths, REFRESH_TIMEOUT);
        snapshot
            .into_iter()
            .zip(found)
            .map(|((id, path, was_available), presence)| {
                // Back online: read its details again (size may have changed).
                let fresh = (presence == Presence::Present && !was_available)
                    .then(|| FileMetadata::from_path(path.clone()))
                    .flatten();
                (id, path, presence, fresh)
            })
            .collect::<Vec<_>>()
    })
    .await
    .map_err(|e| e.to_string())?;

    let mut changed = false;
    let mut deleted = 0usize;
    {
        let mut list = file_list
            .lock()
            .map_err(|_| "Failed to acquire lock".to_string())?;
        for (id, path, presence, fresh) in checked {
            // The item may have been removed, or renamed, while the check
            // ran: what was checked is then no longer its file.
            let Some(index) = list.iter().position(|f| f.id == id && f.path == path) else {
                continue;
            };
            match presence {
                Presence::Gone => {
                    list.remove(index);
                    deleted += 1;
                    changed = true;
                }
                Presence::Unreachable if list[index].available => {
                    list[index].available = false;
                    changed = true;
                }
                Presence::Present if !list[index].available => {
                    if let Some(fresh) = fresh {
                        let file = &mut list[index];
                        file.name = fresh.name;
                        file.size = fresh.size;
                        file.file_type = fresh.file_type;
                        file.kind = fresh.kind;
                        file.available = true;
                        changed = true;
                    }
                }
                _ => {}
            }
        }
    }
    if changed {
        app_handle
            .emit("files_updated", ())
            .map_err(|e| e.to_string())?;
    }
    if !removals_expected() {
        notify_missing(app_handle, deleted);
    }
    Ok(())
}

#[tauri::command]
pub async fn refresh_file_list(app_handle: AppHandle) -> Result<(), String> {
    refresh_shelf(app_handle).await
}

/// Path of a shelf item that can be used right now. Offline items are
/// refused here: shell calls on an unreachable share can hang for seconds.
fn shelf_path(file_list: &FileList, id: u64) -> Result<PathBuf, String> {
    let list = file_list.lock().map_err(|_| "Failed to acquire lock".to_string())?;
    let file = list
        .iter()
        .find(|f| f.id == id)
        .ok_or_else(|| format!("File with ID {} is not on the shelf", id))?;
    if !file.available {
        return Err("The item's drive or network location is offline".to_string());
    }
    Ok(file.path.clone())
}

/// Opens a shelf item with its default app, as a double-click in Explorer
/// would. Takes an id, not a path, so the page can only open shelf items.
#[tauri::command]
pub fn open_shelf_item(file_list: State<'_, FileList>, id: u64) -> Result<(), String> {
    let path = shelf_path(&file_list, id)?;
    crate::utils::shell_open(&windows::core::HSTRING::from(path.as_os_str()))
        .map_err(|code| format!("Windows couldn't open the file (code {})", code))
}

/// Opens Explorer with the shelf item selected.
#[tauri::command]
pub fn reveal_shelf_item(file_list: State<'_, FileList>, id: u64) -> Result<(), String> {
    use std::os::windows::process::CommandExt;
    let path = shelf_path(&file_list, id)?;
    // `/select,"path"` must reach Explorer unsplit, so pass it verbatim.
    std::process::Command::new("explorer.exe")
        .raw_arg(format!("/select,\"{}\"", path.display()))
        .spawn()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// Async so it runs off the main thread: a sync command would block every
/// window (and any drag in progress) while Windows renders thumbnails for
/// videos or PDFs. The shell call itself is blocking, so it gets a blocking
/// worker; `generate_thumbnail` initialises COM per call on that thread.
#[tauri::command]
pub async fn get_file_icon_base64(
    file_list: State<'_, FileList>,
    file_path: String,
) -> Result<String, String> {
    // Only for items on the shelf: the page never gets the shell to touch
    // an arbitrary path.
    let on_shelf = file_list
        .lock()
        .map_err(|_| "Failed to acquire lock".to_string())?
        .iter()
        .any(|f| f.available && f.path.as_os_str() == std::ffi::OsStr::new(&file_path));
    if !on_shelf {
        return Err("Not a shelf item".to_string());
    }
    tauri::async_runtime::spawn_blocking(move || {
        get_thumbnail_base64(std::path::Path::new(&file_path))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notes_are_named_after_their_first_line() {
        assert_eq!(note_title("\n  Meeting notes: Q3 plan  \nmore"), "Meeting notes_ Q3 plan");
        assert_eq!(note_title("   \n\t"), "Note");
        assert_eq!(note_title("***"), "Note");
        assert_eq!(note_title(&"a".repeat(100)).chars().count(), 40);
    }

    #[test]
    fn saved_names_stay_clean() {
        assert_eq!(clean_file_name("report", "pdf"), "report.pdf");
        assert_eq!(clean_file_name("a/b:c", "txt"), "a_b_c.txt");
        assert_eq!(clean_file_name("   ", "png"), "unnamed.png");
        assert_eq!(clean_file_name("trailing.", "txt"), "trailing.txt");
    }

    #[test]
    fn shortcut_targets_are_sanitised() {
        assert_eq!(
            shortcut_target("https://example.com/a?b=1").unwrap(),
            "https://example.com/a?b=1"
        );
        // URL parsing strips embedded newlines, so no extra .url lines.
        let injected = shortcut_target("https://example.com/\r\nIconFile=\\\\host\\s\\i.ico").unwrap();
        assert!(!injected.contains('\n') && !injected.contains('\r'));
        assert!(shortcut_target("file:///C:/Windows/System32/calc.exe").is_err());
        assert!(shortcut_target("javascript:alert(1)").is_err());
    }

    #[test]
    fn url_names_are_decoded() {
        assert_eq!(percent_decode("My%20Photo.jpg"), "My Photo.jpg");
        assert_eq!(percent_decode("%E5%86%99%E7%9C%9F.png"), "写真.png");
        assert_eq!(percent_decode("100%.png"), "100%.png");
        assert_eq!(percent_decode("a%2"), "a%2");
    }

    #[test]
    fn device_names_never_become_file_names() {
        assert_eq!(clean_file_name("nul", "pdf"), "_nul.pdf");
        assert_eq!(clean_file_name("Com1", "txt"), "_Com1.txt");
        assert_eq!(clean_file_name("aux.tar", "gz"), "_aux.tar.gz");
        assert_eq!(clean_file_name("console", "log"), "console.log");
        assert_eq!(clean_file_name("COM0", "txt"), "COM0.txt");
    }

    #[test]
    fn link_files_are_named_after_the_site() {
        assert_eq!(placeholder_name("https://www.github.com/a/b?q=1"), "github.com.url");
        assert_eq!(placeholder_name("http://localhost:3000/"), "localhost.url");
        assert_eq!(placeholder_name("blob:https://discord.com/1234"), "link.url");
        assert_eq!(placeholder_name("not a url"), "link.url");
    }
}

/// Adds the clipboard to the shelf (Ctrl+V in Holdem's windows). Sync, so it
/// runs on the main thread, which has OLE initialised for the clipboard.
#[tauri::command]
pub fn paste_clipboard(app_handle: AppHandle) -> bool {
    crate::drop_target::paste_clipboard(&app_handle)
}

/// Puts shelf items on the clipboard as files (Ctrl+C), so Ctrl+V in
/// Explorer, mail or chat pastes copies of them. Returns how many were put.
#[tauri::command]
pub fn copy_items_to_clipboard(file_list: State<'_, FileList>, ids: Vec<u64>) -> Result<usize, String> {
    let paths: Vec<PathBuf> = file_list
        .lock()
        .map_err(|_| "Failed to acquire lock".to_string())?
        .iter()
        // Offline items would paste as broken paths.
        .filter(|f| ids.contains(&f.id) && f.available)
        .map(|f| f.path.clone())
        .collect();
    if paths.is_empty() {
        return Ok(0);
    }
    clipboard::set_files(&paths).map_err(|e| e.to_string())?;
    Ok(paths.len())
}

mod clipboard {
    use std::os::windows::ffi::OsStrExt;
    use std::path::PathBuf;
    use windows::core::w;
    use windows::Win32::Foundation::{GlobalFree, HANDLE, HGLOBAL};
    use windows::Win32::System::DataExchange::{
        CloseClipboard, EmptyClipboard, OpenClipboard, RegisterClipboardFormatW, SetClipboardData,
    };
    use windows::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};
    use windows::Win32::System::Ole::{CF_HDROP, DROPEFFECT_COPY};
    use windows::Win32::UI::Shell::DROPFILES;

    /// A movable global block holding `bytes`; the clipboard owns it once set.
    fn global_with(bytes: &[u8]) -> windows::core::Result<HGLOBAL> {
        unsafe {
            let handle = GlobalAlloc(GMEM_MOVEABLE, bytes.len())?;
            let ptr = GlobalLock(handle) as *mut u8;
            if ptr.is_null() {
                let _ = GlobalFree(Some(handle));
                return Err(windows::core::Error::from_thread());
            }
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, bytes.len());
            let _ = GlobalUnlock(handle);
            Ok(handle)
        }
    }

    /// CF_HDROP: a DROPFILES header, then wide paths each ending in NUL,
    /// then a final NUL.
    fn hdrop_bytes(paths: &[PathBuf]) -> Vec<u8> {
        let header = std::mem::size_of::<DROPFILES>();
        let mut wide: Vec<u16> = Vec::new();
        for p in paths {
            wide.extend(p.as_os_str().encode_wide());
            wide.push(0);
        }
        wide.push(0);
        let mut bytes = vec![0u8; header];
        let dropfiles = DROPFILES {
            pFiles: header as u32,
            fWide: true.into(),
            ..Default::default()
        };
        unsafe {
            std::ptr::copy_nonoverlapping(
                &dropfiles as *const DROPFILES as *const u8,
                bytes.as_mut_ptr(),
                header,
            );
        }
        bytes.extend(wide.iter().flat_map(|u| u.to_le_bytes()));
        bytes
    }

    pub fn set_files(paths: &[PathBuf]) -> windows::core::Result<()> {
        unsafe {
            OpenClipboard(None)?;
            let result = (|| {
                EmptyClipboard()?;
                // The clipboard owns the memory only once it accepted it.
                let set = |format: u32, data: HGLOBAL| {
                    SetClipboardData(format, Some(HANDLE(data.0))).inspect_err(|_| {
                        let _ = GlobalFree(Some(data));
                    })
                };
                set(CF_HDROP.0 as u32, global_with(&hdrop_bytes(paths))?)?;
                // Paste copies, never moves (as with drag-out).
                let preferred = RegisterClipboardFormatW(w!("Preferred DropEffect"));
                if preferred != 0 {
                    set(preferred, global_with(&DROPEFFECT_COPY.0.to_ne_bytes())?)?;
                }
                Ok(())
            })();
            let _ = CloseClipboard();
            result
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn hdrop_layout_is_header_then_double_nul_terminated_paths() {
            let bytes = hdrop_bytes(&[PathBuf::from(r"C:\a.txt"), PathBuf::from(r"C:\b")]);
            let header = std::mem::size_of::<DROPFILES>();
            let tail: Vec<u16> = bytes[header..]
                .chunks(2)
                .map(|c| u16::from_le_bytes([c[0], c[1]]))
                .collect();
            let text = String::from_utf16_lossy(&tail);
            assert_eq!(text, "C:\\a.txt\0C:\\b\0\0");
        }
    }
}

/// Pins or unpins shelf items. Pinned items survive Clear.
#[tauri::command]
pub fn set_pinned(
    app_handle: AppHandle,
    file_list: State<'_, FileList>,
    ids: Vec<u64>,
    pinned: bool,
) -> Result<(), String> {
    let mut changed = false;
    for file in file_list
        .lock()
        .map_err(|_| "Failed to acquire lock".to_string())?
        .iter_mut()
        .filter(|f| ids.contains(&f.id) && f.pinned != pinned)
    {
        file.pinned = pinned;
        changed = true;
    }
    if changed {
        app_handle.emit("files_updated", ()).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Why a rename failed. Sent to the page as a short code it can turn into a
/// message.
#[derive(Debug, thiserror::Error)]
pub enum RenameError {
    #[error("no such item on the shelf")]
    NotFound,
    #[error("only items Holdem saved itself can be renamed")]
    NotOwned,
    #[error("the name is empty")]
    Empty,
    #[error("an item with that name already exists")]
    Exists,
    #[error("the file is open in another app")]
    InUse,
    #[error("{0}")]
    Failed(String),
}

impl serde::Serialize for RenameError {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(match self {
            Self::NotFound => "notFound",
            Self::NotOwned => "notOwned",
            Self::Empty => "empty",
            Self::Exists => "exists",
            Self::InUse => "inUse",
            Self::Failed(_) => "failed",
        })
    }
}

impl From<std::io::Error> for RenameError {
    fn from(error: std::io::Error) -> Self {
        // ERROR_SHARING_VIOLATION, ERROR_LOCK_VIOLATION
        match error.raw_os_error() {
            Some(32 | 33) => Self::InUse,
            _ => Self::Failed(error.to_string()),
        }
    }
}

/// Renames an item Holdem saved itself (an attachment, image, text or
/// link), on disk, so it is dragged out under the new name. `name` replaces
/// the name without its extension; the extension stays. The user's own
/// files are never renamed here. Returns the new file name.
#[tauri::command]
pub fn rename_item(
    app_handle: AppHandle,
    file_list: State<'_, FileList>,
    stash: State<'_, RemovedStash>,
    shelves: State<'_, crate::shelves::Shelves>,
    id: u64,
    name: String,
) -> Result<String, RenameError> {
    let lock_failed = || RenameError::Failed("Failed to acquire lock".to_string());
    // Read what's needed, then let go of the shelf: the rename touches the
    // disk (and whatever antivirus hooks it), and every window waits on
    // this lock.
    let entry = file_list
        .lock()
        .map_err(|_| lock_failed())?
        .iter()
        .find(|f| f.id == id)
        .cloned()
        .ok_or(RenameError::NotFound)?;
    if !entry.owned || !entry.available {
        return Err(RenameError::NotOwned);
    }
    let stem = name.trim();
    if stem.is_empty() {
        return Err(RenameError::Empty);
    }
    let ext = entry
        .path
        .extension()
        .map(|e| e.to_string_lossy().into_owned())
        .unwrap_or_default();
    let new_name = clean_file_name(stem, &ext);
    let new_path = entry.path.with_file_name(&new_name);
    if new_path == entry.path {
        return Ok(new_name);
    }
    // Each saved item has a folder of its own, so only a case-only change
    // can meet an existing name, and that is the same file.
    let same_file = new_name.to_lowercase() == entry.name.to_lowercase();
    if new_path.exists() && !same_file {
        return Err(RenameError::Exists);
    }
    let old_path = entry.path.clone();
    std::fs::rename(&old_path, &new_path)?;
    let renamed = FileMetadata::from_path(new_path.clone())
        .ok_or_else(|| RenameError::Failed("The renamed file can't be read".to_string()))?;
    // The same file can be on other shelves too (copied and pasted), and in
    // Undo or a closed shelf: all of them follow the rename.
    let mut list = file_list.lock().map_err(|_| lock_failed())?;
    for file in list.iter_mut().filter(|f| f.path == old_path) {
        *file = FileMetadata {
            id: file.id,
            pinned: file.pinned,
            shelf: file.shelf,
            ..renamed.clone()
        };
    }
    drop(list);
    if let Ok(mut stash) = stash.lock() {
        for file in stash.iter_mut().filter(|f| f.path == old_path) {
            file.path = new_path.clone();
            file.name = renamed.name.clone();
        }
    }
    if let Ok(mut set) = shelves.lock() {
        for item in set.closed.iter_mut().flat_map(|c| c.items.iter_mut()) {
            if item.path == old_path {
                item.path = new_path.clone();
            }
        }
    }
    app_handle
        .emit("files_updated", ())
        .map_err(|e| RenameError::Failed(e.to_string()))?;
    Ok(new_name)
}

/// Existing paths among command-line arguments ("Send to > Holdem", or files
/// dropped on the app's icon). Flags and the program path are ignored.
pub fn launch_paths(args: &[String]) -> Vec<PathBuf> {
    args.iter()
        .skip(1)
        .filter(|a| !a.starts_with("--"))
        .map(PathBuf::from)
        .filter(|p| p.exists())
        .collect()
}
