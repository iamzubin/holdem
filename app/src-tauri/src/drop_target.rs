use crate::drop_report::{save_issue, DropIssue, DropReport, MAX_DROP_FILES, MAX_DROP_MB, MAX_ITEM_MB};
use crate::{DragState, FileList};
use std::cell::Cell;
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};
use tracing::{error, info, warn};
use windows::core::{implement, w, BOOL};
use windows::Win32::Foundation::{HWND, LPARAM, POINTL};
use windows::Win32::System::Com::{
    IDataObject, DVASPECT_CONTENT, FORMATETC, STGMEDIUM, TYMED_HGLOBAL, TYMED_ISTREAM,
};
use windows::Win32::System::Ole::ReleaseStgMedium;
use windows::Win32::System::DataExchange::RegisterClipboardFormatW;
use windows::Win32::System::Memory::{GlobalLock, GlobalSize, GlobalUnlock};
use windows::Win32::System::Ole::{
    IDropTarget, IDropTarget_Impl, RegisterDragDrop, RevokeDragDrop, CF_DIB, CF_DIBV5, CF_HDROP,
    CF_UNICODETEXT, DROPEFFECT, DROPEFFECT_COPY, DROPEFFECT_NONE, OleGetClipboard,
};
use windows::Win32::System::SystemServices::MODIFIERKEYS_FLAGS;
use windows::Win32::UI::Shell::{DragQueryFileW, HDROP};
use windows::Win32::UI::WindowsAndMessaging::EnumChildWindows;

/// Total bytes accepted from one virtual-file drop. Each file is already
/// capped while reading; this stops a multi-file drop from buffering
/// gigabytes on the UI thread.
const MAX_VIRTUAL_DROP_BYTES: usize = MAX_DROP_MB * 1024 * 1024;
/// A dropped virtual file: its name and contents.
type VirtualFile = (String, Vec<u8>);

/// Cap for any single dropped data block (one virtual file, one image).
const MAX_STREAM_BYTES: usize = MAX_ITEM_MB * 1024 * 1024;

#[implement(IDropTarget)]
pub struct HoldemDropTarget {
    app_handle: AppHandle,
    /// Label of the window this target serves ("main" or "popup"); it gets
    /// the hover highlight.
    window: &'static str,
    /// Why the last data read failed, if it did (too large, or the source
    /// stopped part-way). Reset for each drop.
    read_issue: Cell<Option<DropIssue>>,
}

impl HoldemDropTarget {
    pub fn new(app_handle: AppHandle, window: &'static str) -> Self {
        Self { app_handle, window, read_issue: Cell::new(None) }
    }

    unsafe fn extract_files(&self, data_obj: &IDataObject) -> Option<Vec<PathBuf>> {
        let drop_format = FORMATETC {
            cfFormat: CF_HDROP.0,
            ptd: std::ptr::null_mut(),
            dwAspect: DVASPECT_CONTENT.0,
            lindex: -1,
            tymed: TYMED_HGLOBAL.0 as u32,
        };

        if let Ok(mut medium) = data_obj.GetData(&drop_format) {
            let hglobal = medium.u.hGlobal;
            let hdrop = HDROP(hglobal.0 as _);

            let item_count = DragQueryFileW(hdrop, 0xFFFFFFFF, None);
            let mut paths = Vec::new();

            for i in 0..item_count {
                let character_count = DragQueryFileW(hdrop, i, None) as usize;
                let mut path_buf = vec![0u16; character_count + 1];
                DragQueryFileW(hdrop, i, Some(&mut path_buf));
                path_buf.truncate(character_count);
                paths.push(PathBuf::from(OsString::from_wide(&path_buf)));
            }

            // NOTE: no DragFinish here — that is only for WM_DROPFILES HDROPs.
            // This HGLOBAL comes from IDataObject::GetData and is owned by the
            // STGMEDIUM; ReleaseStgMedium frees it. Calling both double-frees.
            ReleaseStgMedium(&mut medium);

            if !paths.is_empty() {
                return Some(paths);
            }
        }
        None
    }

    unsafe fn get_hglobal_bytes(&self, medium: &STGMEDIUM) -> Option<Vec<u8>> {
        if medium.tymed != TYMED_HGLOBAL.0 as u32 {
            return None;
        }
        let hglobal = medium.u.hGlobal;
        if hglobal.is_invalid() {
            return None;
        }
        let ptr = GlobalLock(hglobal) as *const u8;
        if ptr.is_null() {
            return None;
        }
        let size = GlobalSize(hglobal);
        if size > MAX_STREAM_BYTES {
            let _ = GlobalUnlock(hglobal);
            warn!("Dropped data block exceeds {} MB cap, rejecting", MAX_ITEM_MB);
            self.read_issue.set(Some(DropIssue::TooLarge));
            return None;
        }
        let bytes = std::slice::from_raw_parts(ptr, size).to_vec();
        let _ = GlobalUnlock(hglobal);
        Some(bytes)
    }

    unsafe fn read_istream_bytes(&self, medium: &STGMEDIUM) -> Option<Vec<u8>> {
        if medium.tymed != TYMED_ISTREAM.0 as u32 {
            return None;
        }
        let stream_guard = medium.u.pstm.as_ref();
        let stream = stream_guard.as_ref()?;
        let mut out = Vec::new();
        let mut buf = vec![0u8; 65536];
        loop {
            let mut read: u32 = 0;
            let hr = stream.Read(
                buf.as_mut_ptr() as *mut _,
                buf.len() as u32,
                Some(&mut read as *mut u32),
            );
            if hr.is_err() {
                warn!("Virtual file stream read failed, rejecting partial data");
                self.read_issue.set(Some(DropIssue::ReadFailed));
                return None;
            }
            if read == 0 {
                break;
            }
            out.extend_from_slice(&buf[..read as usize]);
            if out.len() > MAX_STREAM_BYTES {
                warn!("Virtual file stream exceeds {} MB cap, rejecting", MAX_ITEM_MB);
                self.read_issue.set(Some(DropIssue::TooLarge));
                return None;
            }
        }
        if out.is_empty() {
            None
        } else {
            Some(out)
        }
    }

    unsafe fn get_data_bytes(
        &self,
        data_obj: &IDataObject,
        cf_format: u16,
        lindex: i32,
    ) -> Option<Vec<u8>> {
        // Prefer IStream (virtual files, large PNGs), fall back to HGLOBAL.
        for tymed in [TYMED_ISTREAM.0 as u32, TYMED_HGLOBAL.0 as u32] {
            let fmt = FORMATETC {
                cfFormat: cf_format,
                ptd: std::ptr::null_mut(),
                dwAspect: DVASPECT_CONTENT.0,
                lindex,
                tymed,
            };
            if let Ok(mut medium) = data_obj.GetData(&fmt) {
                let bytes = if medium.tymed == TYMED_ISTREAM.0 as u32 {
                    self.read_istream_bytes(&medium)
                } else {
                    self.get_hglobal_bytes(&medium)
                };
                ReleaseStgMedium(&mut medium);
                if let Some(b) = bytes {
                    if !b.is_empty() {
                        return Some(b);
                    }
                }
            }
        }
        None
    }

    /// FileContents offered only as a structured-storage object: classic
    /// Outlook hands over a dragged email this way. Copied into an in-memory
    /// compound file, whose bytes are the .msg file Outlook itself saves.
    unsafe fn get_storage_bytes(
        &self,
        data_obj: &IDataObject,
        cf_format: u16,
        lindex: i32,
    ) -> Option<Vec<u8>> {
        use windows::Win32::System::Com::TYMED_ISTORAGE;
        let fmt = FORMATETC {
            cfFormat: cf_format,
            ptd: std::ptr::null_mut(),
            dwAspect: DVASPECT_CONTENT.0,
            lindex,
            tymed: TYMED_ISTORAGE.0 as u32,
        };
        let mut medium = data_obj.GetData(&fmt).ok()?;
        let copied = if medium.tymed == TYMED_ISTORAGE.0 as u32 {
            medium.u.pstg.as_ref().map(|source| storage_to_bytes(source))
        } else {
            None
        };
        ReleaseStgMedium(&mut medium);
        match copied? {
            Ok(bytes) if bytes.len() > MAX_STREAM_BYTES => {
                self.read_issue.set(Some(DropIssue::TooLarge));
                None
            }
            Ok(bytes) => (!bytes.is_empty()).then_some(bytes),
            Err(e) => {
                warn!("Couldn't copy a dropped email: {}", e);
                self.read_issue.set(Some(DropIssue::ReadFailed));
                None
            }
        }
    }

    unsafe fn clipboard_format(&self, name: &str) -> Option<u16> {
        // RegisterClipboardFormatW only takes wide literals via w!; match known names.
        let id = match name {
            "HTML Format" => RegisterClipboardFormatW(w!("HTML Format")),
            "FileGroupDescriptorW" => RegisterClipboardFormatW(w!("FileGroupDescriptorW")),
            "FileGroupDescriptor" => RegisterClipboardFormatW(w!("FileGroupDescriptor")),
            "FileContents" => RegisterClipboardFormatW(w!("FileContents")),
            "UniformResourceLocatorW" => {
                RegisterClipboardFormatW(w!("UniformResourceLocatorW"))
            }
            "UniformResourceLocator" => RegisterClipboardFormatW(w!("UniformResourceLocator")),
            "PNG" => RegisterClipboardFormatW(w!("PNG")),
            _ => return None,
        };
        if id == 0 {
            None
        } else {
            Some(id as u16)
        }
    }

    unsafe fn extract_html_raw(&self, data_obj: &IDataObject) -> Option<String> {
        let cf = self.clipboard_format("HTML Format")?;
        let bytes = self.get_data_bytes(data_obj, cf, -1)?;
        let end = bytes.iter().position(|&c| c == 0).unwrap_or(bytes.len());
        Some(String::from_utf8_lossy(&bytes[..end]).into_owned())
    }

    unsafe fn extract_unicode_text_with_format(
        &self,
        data_obj: &IDataObject,
        cf: u16,
    ) -> Option<String> {
        let bytes = self.get_data_bytes(data_obj, cf, -1)?;
        if bytes.len() < 2 {
            return None;
        }
        // NOTE: no raw `*const u16` cast — `Vec<u8>` is align-1 and the
        // allocation can be odd-aligned (UB even on x86-64). Decode LE pairs.
        let wide: Vec<u16> = bytes
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();
        let end = wide.iter().position(|&c| c == 0).unwrap_or(wide.len());
        let s = String::from_utf16_lossy(&wide[..end]);
        if s.is_empty() {
            None
        } else {
            Some(s)
        }
    }

    unsafe fn extract_ansi_text_with_format(
        &self,
        data_obj: &IDataObject,
        cf: u16,
    ) -> Option<String> {
        let bytes = self.get_data_bytes(data_obj, cf, -1)?;
        let end = bytes.iter().position(|&c| c == 0).unwrap_or(bytes.len());
        let s = String::from_utf8_lossy(&bytes[..end]).trim().to_string();
        if s.is_empty() {
            None
        } else {
            Some(s)
        }
    }

    /// Explicit URL flavors (address-bar / link drags). Disambiguates real
    /// URL drags from plain text that merely happens to start with http.
    unsafe fn extract_url(&self, data_obj: &IDataObject) -> Option<String> {
        if let Some(cf) = self.clipboard_format("UniformResourceLocatorW") {
            if let Some(s) = self.extract_unicode_text_with_format(data_obj, cf) {
                let t = s.trim().to_string();
                if !t.is_empty() {
                    return Some(t);
                }
            }
        }
        if let Some(cf) = self.clipboard_format("UniformResourceLocator") {
            if let Some(s) = self.extract_ansi_text_with_format(data_obj, cf) {
                let t = s.trim().to_string();
                if !t.is_empty() {
                    return Some(t);
                }
            }
        }
        None
    }

    /// Raw PNG bytes when the source offers the "PNG" clipboard format
    /// (Snipping Tool, browsers, Office image drags).
    unsafe fn extract_png_bytes(&self, data_obj: &IDataObject) -> Option<Vec<u8>> {
        let cf = self.clipboard_format("PNG")?;
        self.get_data_bytes(data_obj, cf, -1)
    }

    /// CF_DIB / CF_DIBV5 (device-independent bitmap) -> PNG bytes via `image`.
    /// Covers screenshots and canvas drags that expose only a bitmap.
    unsafe fn extract_dib_png_bytes(&self, data_obj: &IDataObject) -> Option<Vec<u8>> {
        for cf in [CF_DIB.0, CF_DIBV5.0] {
            if let Some(bytes) = self.get_data_bytes(data_obj, cf, -1) {
                if let Some(png) = dib_bytes_to_png(&bytes) {
                    return Some(png);
                }
            }
        }
        None
    }

    /// Virtual files: Outlook attachments and messages, Discord/Slack, ZIP
    /// folders, FTP — exposed as FileGroupDescriptor(W) + FileContents
    /// (per-file lindex). Older apps only offer the ANSI descriptor.
    /// Returns the files that could be read, and why the others could not.
    unsafe fn extract_virtual_files(
        &self,
        data_obj: &IDataObject,
    ) -> Option<(Vec<VirtualFile>, DropReport)> {
        let cf_contents = self.clipboard_format("FileContents")?;
        let (entries, listed) = [("FileGroupDescriptorW", true), ("FileGroupDescriptor", false)]
            .into_iter()
            .find_map(|(format, wide)| {
                let cf = self.clipboard_format(format)?;
                let bytes = self.get_data_bytes(data_obj, cf, -1)?;
                let entries = parse_file_group_descriptor(&bytes, wide);
                (!entries.is_empty()).then(|| (entries, descriptor_count(&bytes)))
            })?;

        let mut report = DropReport::new();
        // Past the item limit: left out, and said so.
        report.add(DropIssue::DropTooLarge, listed.saturating_sub(MAX_DROP_FILES));
        let mut out = Vec::new();
        let mut total = 0usize;
        let mut remaining = entries.len();
        for entry in entries {
            remaining -= 1;
            // A declared size over the cap is rejected without reading it.
            if entry.size.is_some_and(|size| size > MAX_STREAM_BYTES as u64) {
                report.add_named(DropIssue::TooLarge, &entry.name);
                continue;
            }
            self.read_issue.set(None);
            // FileContents per lindex: IStream, HGLOBAL, then structured
            // storage (an email dragged out of classic Outlook).
            let mut name = entry.name;
            let mut from_storage = false;
            let contents = self
                .get_data_bytes(data_obj, cf_contents, entry.index as i32)
                .or_else(|| {
                    let message = self.get_storage_bytes(data_obj, cf_contents, entry.index as i32);
                    if message.is_some() {
                        from_storage = true;
                        if !name.to_ascii_lowercase().ends_with(".msg") {
                            name.push_str(".msg");
                        }
                    }
                    message
                })
                // An empty file has no contents to hand over; it is still a file.
                .or_else(|| (entry.size == Some(0)).then(Vec::new));
            let Some(mut bytes) = contents else {
                warn!("No contents for a dropped virtual file");
                let issue = self.read_issue.take().unwrap_or(DropIssue::NoContents);
                report.add_named(issue, &name);
                continue;
            };
            // HGLOBAL blocks can be larger than the file; trust the declared
            // size. Not for an email copied out of storage: its compound file
            // is laid out anew, so the declared size doesn't apply.
            if let Some(size) = entry.size.filter(|_| !from_storage) {
                if (size as usize) < bytes.len() {
                    bytes.truncate(size as usize);
                }
            }
            total += bytes.len();
            if total > MAX_VIRTUAL_DROP_BYTES {
                warn!(
                    "Virtual drop exceeds {} MB in total, skipping the remaining files",
                    MAX_DROP_MB
                );
                report.add(DropIssue::DropTooLarge, remaining + 1);
                break;
            }
            out.push((name, bytes));
        }
        Some((out, report))
    }

    /// Adds whatever `dataobj` carries to the shelf, trying the richest
    /// format first. Shared by drops and Ctrl+V. Every path ends by telling
    /// the user what was not added and why (see `drop_report`). Returns false
    /// when nothing could be kept.
    unsafe fn ingest(&self, dataobj: &IDataObject) -> bool {
        self.read_issue.set(None);
        let app = &self.app_handle;
        // What went wrong in a richer format that was tried first; sent with
        // whatever the drop ends up adding.
        let mut report = DropReport::new();

        // 1. Real filesystem files — lossless, no copy (except temporary
        //    ones, which their app deletes right after the drop).
        if let Some(paths) = self.extract_files(dataobj) {
            info!("HoldemDropTarget received {} file path(s)", paths.len());
            mark_drop(app);
            let paths = keep_temporary_files(paths, &mut report);
            add_to_shelf(paths, report, app.clone());
            return true;
        }

        // 2. Virtual files (Outlook/Discord/Slack/ZIP): FileGroupDescriptorW.
        let mut virtual_failed = false;
        if let Some((virtual_files, virtual_report)) = self.extract_virtual_files(dataobj) {
            report = virtual_report;
            virtual_failed = true;
            if !virtual_files.is_empty() {
                info!("HoldemDropTarget received {} virtual file(s)", virtual_files.len());
                mark_drop(app);
                save_virtual_files(virtual_files, report, app.clone());
                return true;
            }
            // None could be read: another format may still hold something.
        }

        // 3. Bitmaps — prefer lossless pixels over re-downloading.
        //    PNG format first (already encoded), then CF_DIB(V5).
        if let Some(png_bytes) = self.extract_png_bytes(dataobj) {
            // Validate it actually looks like a PNG before accepting.
            if png_bytes.len() > 8 && &png_bytes[..8] == b"\x89PNG\r\n\x1a\n" {
                info!("HoldemDropTarget received PNG clipboard image");
                mark_drop(app);
                handle_image_bytes(png_bytes, "screenshot.png".to_string(), report, app.clone());
                return true;
            }
        }
        if let Some(png_bytes) = self.extract_dib_png_bytes(dataobj) {
            info!("HoldemDropTarget received DIB bitmap image");
            mark_drop(app);
            handle_image_bytes(png_bytes, "screenshot.png".to_string(), report, app.clone());
            return true;
        }

        // Attachments that couldn't be read: the text the same app offers
        // alongside (an Outlook header line, say) is not what was dropped.
        if virtual_failed {
            report.send(app);
            return false;
        }

        // 4. HTML (browser <img> drags): all images + SourceURL referer.
        if let Some(html) = self.extract_html_raw(dataobj) {
            let (srcs, referer) = parse_html_images(&html);
            // A selection of mostly text that happens to include an image
            // (an emoji, avatar or tracking pixel) is text, not an image drop.
            if !srcs.is_empty() && fragment_text_chars(&html) <= MAX_IMAGE_DROP_TEXT {
                info!(
                    "HoldemDropTarget detected {} image(s) in dropped HTML",
                    srcs.len()
                );
                mark_drop(app);
                handle_image_srcs(srcs, referer, report, app.clone());
                return true;
            }
            // HTML without images (rich text) — fall through to URL/text.
        }

        // 5. Explicit URL flavors (link / address-bar drags).
        if let Some(url) = self.extract_url(dataobj) {
            let url = url.trim().to_string();
            if !url.is_empty() {
                info!("HoldemDropTarget received a dropped link");
                mark_drop(app);
                handle_link_drop(url, report, app.clone());
                return true;
            }
        }

        // 6. Plain text last resort.
        if let Some(text) =
            self.extract_unicode_text_with_format(dataobj, CF_UNICODETEXT.0)
        {
            let text = text.trim();
            if !text.is_empty() {
                mark_drop(app);
                if is_single_web_url(text) {
                    info!("HoldemDropTarget received link text");
                    handle_link_drop(text.to_string(), report, app.clone());
                } else {
                    info!("HoldemDropTarget saving dropped text snippet");
                    handle_text_drop(text.to_string(), report, app.clone());
                }
                return true;
            }
        }
        warn!("Data had no format Holdem can keep");
        if report.is_empty() {
            // An image too large to take is "too large", not "unsupported".
            let too_large = self.read_issue.get() == Some(DropIssue::TooLarge);
            report.add(
                if too_large { DropIssue::TooLarge } else { DropIssue::Unsupported },
                1,
            );
        }
        report.send(app);
        false
    }
}

#[allow(non_snake_case)]
impl IDropTarget_Impl for HoldemDropTarget_Impl {
    fn DragEnter(
        &self,
        _pdataobj: windows_core::Ref<'_, IDataObject>,
        _grfkeystate: MODIFIERKEYS_FLAGS,
        _pt: &POINTL,
        pdweffect: *mut DROPEFFECT,
    ) -> windows::core::Result<()> {
        // The drag reached Holdem, so Windows did not block it (see
        // mouse_monitor's check for drags from elevated apps).
        crate::mouse_monitor::DRAG_ENTERED.store(true, Ordering::SeqCst);
        // Items dragged out of Holdem are already on the shelf: dropping them
        // on the shelf or the list again does nothing.
        if own_drag() {
            unsafe {
                if !pdweffect.is_null() {
                    *pdweffect = DROPEFFECT_NONE;
                }
            }
            return Ok(());
        }
        if let Some(drag_state) = self.app_handle.try_state::<Arc<DragState>>() {
            drag_state.drag_started.store(true, Ordering::Relaxed);
        }
        set_drop_hover(&self.app_handle, self.window, true);
        unsafe {
            if !pdweffect.is_null() {
                *pdweffect = accepted_effect(*pdweffect);
            }
        }
        Ok(())
    }

    fn DragOver(
        &self,
        _grfkeystate: MODIFIERKEYS_FLAGS,
        _pt: &POINTL,
        pdweffect: *mut DROPEFFECT,
    ) -> windows::core::Result<()> {
        unsafe {
            if !pdweffect.is_null() {
                *pdweffect = if own_drag() { DROPEFFECT_NONE } else { accepted_effect(*pdweffect) };
            }
        }
        Ok(())
    }

    fn DragLeave(&self) -> windows::core::Result<()> {
        if let Some(drag_state) = self.app_handle.try_state::<Arc<DragState>>() {
            drag_state.drag_started.store(false, Ordering::Relaxed);
        }
        set_drop_hover(&self.app_handle, self.window, false);
        Ok(())
    }

    fn Drop(
        &self,
        pdataobj: windows_core::Ref<'_, IDataObject>,
        _grfkeystate: MODIFIERKEYS_FLAGS,
        _pt: &POINTL,
        pdweffect: *mut DROPEFFECT,
    ) -> windows::core::Result<()> {
        set_drop_hover(&self.app_handle, self.window, false);
        let handled = !own_drag()
            && unsafe { pdataobj.as_ref().is_some_and(|dataobj| self.ingest(dataobj)) };
        // Nothing Holdem could keep: tell the source this was not a copy.
        unsafe {
            if !pdweffect.is_null() {
                *pdweffect = if handled { accepted_effect(*pdweffect) } else { DROPEFFECT_NONE };
            }
        }
        Ok(())
    }
}

/// Adds the clipboard's contents to the shelf (Ctrl+V, tray menu): files,
/// email attachments, images, links or text, exactly as if they were dropped.
/// Must run on a thread with OLE initialised (the main thread).
pub fn paste_clipboard(app_handle: &AppHandle) -> bool {
    match unsafe { OleGetClipboard() } {
        Ok(dataobj) => unsafe { HoldemDropTarget::new(app_handle.clone(), "main").ingest(&dataobj) },
        Err(e) => {
            warn!("Could not read the clipboard: {}", e);
            DropReport::single(DropIssue::ClipboardBusy).send(app_handle);
            false
        }
    }
}

/// The effect to report for a drag offering `allowed`. Holdem keeps a copy
/// and never takes the original away, so it never says "move" (the source
/// would then delete its file); a source that allows neither copy nor link
/// can't be dropped here.
fn accepted_effect(allowed: DROPEFFECT) -> DROPEFFECT {
    use windows::Win32::System::Ole::DROPEFFECT_LINK;
    if allowed.0 & DROPEFFECT_COPY.0 != 0 {
        DROPEFFECT_COPY
    } else if allowed.0 & DROPEFFECT_LINK.0 != 0 {
        DROPEFFECT_LINK
    } else {
        DROPEFFECT_NONE
    }
}

fn own_drag() -> bool {
    crate::commands::drag_ops::OUTGOING_DRAG.load(Ordering::SeqCst)
}

/// Tells a window whether something is being dragged over it, so it can
/// highlight. The native drop target replaces WebView2's, so the page never
/// sees HTML drag events itself.
fn set_drop_hover(app_handle: &AppHandle, window: &str, hovering: bool) {
    if window == "popup" {
        // Keeps the popup open for a drag that ends on it.
        crate::commands::window_ops::POPUP_DROP_HOVER.store(hovering, Ordering::SeqCst);
    }
    if let Err(e) = app_handle.emit_to(window, "drop-hover", hovering) {
        warn!("Failed to emit drop-hover: {}", e);
    }
}

/// Files dragged out of an archive (7-Zip, WinRAR, Explorer's ZIP view)
/// are extracted into %TEMP%, and the archiver deletes them as soon as the
/// drag ends (7-Zip) or on its next start (WinRAR). Referencing them would
/// lose them, so they are copied into Holdem's drop folder now, before the
/// drop returns. Everything else stays a reference to the user's file.
fn keep_temporary_files(paths: Vec<PathBuf>, report: &mut DropReport) -> Vec<PathBuf> {
    // Compared as text, in both the short and long spelling of %TEMP%:
    // resolving every dropped path (some on network shares) would hold up
    // the drop.
    let temp = std::env::temp_dir();
    let prefixes: Vec<String> = [temp.to_string_lossy().to_lowercase(), crate::file_drop::path_key(&temp)]
        .into_iter()
        .map(|p| format!("{}\\", p.trim_end_matches('\\')))
        .collect();
    paths
        .into_iter()
        .map(|path| {
            let text = path.to_string_lossy().to_lowercase();
            if !prefixes.iter().any(|p| text.starts_with(p)) {
                return path;
            }
            // Copied while the drop waits, so a whole extracted archive
            // would freeze both apps: past the per-drop limit it stays a
            // reference.
            if size_within(&path, MAX_VIRTUAL_DROP_BYTES as u64).is_none() {
                warn!("A temporary dropped item is too large to copy; keeping a reference");
                return path;
            }
            match copy_into_drop_folder(&path) {
                Ok(copy) => copy,
                Err(e) => {
                    error!("Couldn't keep a copy of a temporary dropped file: {}", e);
                    let name = path.file_name().map(|n| n.to_string_lossy().into_owned());
                    report.add_named(save_issue(&e.into()), name.as_deref().unwrap_or(""));
                    path
                }
            }
        })
        .collect()
}

/// Copies a file or folder into a new drop entry folder; returns the copy.
fn copy_into_drop_folder(path: &std::path::Path) -> std::io::Result<PathBuf> {
    let name = path
        .file_name()
        .ok_or_else(|| std::io::Error::other("no file name"))?;
    let folder = crate::commands::file_ops::new_entry_folder("")
        .map_err(|e| std::io::Error::other(e.to_string()))?;
    let dest = folder.join(name);
    let copied = if path.is_dir() {
        copy_dir(path, &dest)
    } else {
        std::fs::copy(path, &dest).map(|_| ())
    };
    if let Err(e) = copied {
        // No half-filled copy left behind.
        let _ = std::fs::remove_dir_all(&folder);
        return Err(e);
    }
    Ok(dest)
}

/// The total size of a file or folder, or None once it passes `limit`.
fn size_within(path: &std::path::Path, limit: u64) -> Option<u64> {
    let meta = std::fs::symlink_metadata(path).ok()?;
    if !meta.is_dir() {
        return (meta.len() <= limit).then_some(meta.len());
    }
    let mut total = 0u64;
    for entry in std::fs::read_dir(path).ok()?.flatten() {
        total += size_within(&entry.path(), limit.checked_sub(total)?)?;
    }
    Some(total)
}

fn copy_dir(from: &std::path::Path, to: &std::path::Path) -> std::io::Result<()> {
    std::fs::create_dir_all(to)?;
    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        let target = to.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir(&entry.path(), &target)?;
        } else {
            std::fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}

/// Adds saved or dropped paths to the shelf and sends the drop's report.
fn add_to_shelf(paths: Vec<PathBuf>, report: DropReport, app_handle: AppHandle) {
    match app_handle.try_state::<FileList>() {
        Some(file_list) => {
            crate::file_drop::add_paths(paths, report, file_list.inner().clone(), app_handle.clone())
        }
        None => report.send(&app_handle),
    }
}

/// Saves virtual files (attachments) into the drop folder, off the UI
/// thread, then adds them.
fn save_virtual_files(files: Vec<VirtualFile>, mut report: DropReport, app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let mut paths = Vec::new();
        for (name, bytes) in files {
            let ext = std::path::Path::new(&name)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("bin");
            match crate::commands::file_ops::save_bytes_to_drop_folder(&bytes, &name, ext) {
                Ok(p) => {
                    // Attachments came from mail or chat: treat them as
                    // downloaded from the internet.
                    crate::commands::file_ops::mark_from_internet(
                        std::path::Path::new(&p),
                        None,
                        None,
                    );
                    paths.push(PathBuf::from(p))
                }
                Err(e) => {
                    error!("Failed to save a dropped virtual file: {}", e);
                    report.add_named(save_issue(&e), &name);
                }
            }
        }
        add_to_shelf(paths, report, app);
    });
}

fn mark_drop(app_handle: &AppHandle) {
    if let Some(drag_state) = app_handle.try_state::<Arc<DragState>>() {
        drag_state.successful_drop.store(true, Ordering::Relaxed);
        drag_state.drag_started.store(false, Ordering::Relaxed);
    }
}

unsafe extern "system" fn enum_child_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let vec_ptr = lparam.0 as *mut Vec<HWND>;
    if !vec_ptr.is_null() {
        (*vec_ptr).push(hwnd);
    }
    true.into()
}

fn register_on_hwnd(app_handle: &AppHandle, window: &'static str, hwnd: HWND) {
    unsafe {
        let _ = RevokeDragDrop(hwnd);
        let target = HoldemDropTarget::new(app_handle.clone(), window);
        let target_interface: IDropTarget = target.into();
        if let Err(e) = RegisterDragDrop(hwnd, &target_interface) {
            warn!("RegisterDragDrop failed for {:?}: {:?}", hwnd, e);
        }
    }
}

/// Makes `window` (the shelf or the item list) accept drops. Must run on the
/// main thread, where OLE is initialised.
pub fn register_drop_target(app_handle: &AppHandle, window: &'static str, hwnd: HWND) {
    // Top-level window first…
    register_on_hwnd(app_handle, window, hwnd);
    // …then every child (WebView2 hosts its own child WidgetWin which would
    // otherwise swallow drops before they reach us). Collect first, register
    // after enumeration to keep the callback trivial.
    let mut children: Vec<HWND> = Vec::new();
    unsafe {
        let _ = EnumChildWindows(
            Some(hwnd),
            Some(enum_child_proc),
            LPARAM(&mut children as *mut Vec<HWND> as isize),
        );
    }
    for child in children {
        register_on_hwnd(app_handle, window, child);
    }
}

/// The bytes of a structured-storage object as a compound file: copied into
/// one held in memory, whose contents are exactly what would be on disk.
unsafe fn storage_to_bytes(
    source: &windows::Win32::System::Com::StructuredStorage::IStorage,
) -> windows::core::Result<Vec<u8>> {
    use windows::Win32::System::Com::StructuredStorage::{
        CreateILockBytesOnHGlobal, GetHGlobalFromILockBytes, StgCreateDocfileOnILockBytes,
    };
    use windows::Win32::System::Com::{
        STATFLAG_NONAME, STATSTG, STGC_DEFAULT, STGM_CREATE, STGM_READWRITE, STGM_SHARE_EXCLUSIVE,
    };
    let memory = CreateILockBytesOnHGlobal(None, true)?;
    let copy = StgCreateDocfileOnILockBytes(
        &memory,
        STGM_CREATE | STGM_READWRITE | STGM_SHARE_EXCLUSIVE,
        0,
    )?;
    source.CopyTo(None, None, &copy)?;
    copy.Commit(STGC_DEFAULT.0 as u32)?;
    drop(copy);
    let mut stat = STATSTG::default();
    memory.Stat(&mut stat, STATFLAG_NONAME.0 as u32)?;
    let hglobal = GetHGlobalFromILockBytes(&memory)?;
    let ptr = GlobalLock(hglobal) as *const u8;
    if ptr.is_null() {
        return Err(windows::core::Error::from_thread());
    }
    let size = (stat.cbSize as usize).min(GlobalSize(hglobal));
    let bytes = std::slice::from_raw_parts(ptr, size).to_vec();
    let _ = GlobalUnlock(hglobal);
    Ok(bytes)
}

// --- FILEGROUPDESCRIPTOR ----------------------------------------------------

/// One file listed in a FILEGROUPDESCRIPTOR(W/A).
#[derive(Debug, PartialEq, Eq)]
struct DescriptorEntry {
    /// Position in the descriptor, which is the `lindex` for FileContents.
    index: usize,
    name: String,
    /// Declared size, when the source set FD_FILESIZE.
    size: Option<u64>,
}

/// Text in the system ANSI code page (old apps' file names).
fn ansi_to_string(bytes: &[u8]) -> String {
    use windows::Win32::Globalization::{MultiByteToWideChar, CP_ACP, MULTI_BYTE_TO_WIDE_CHAR_FLAGS};
    if bytes.is_empty() {
        return String::new();
    }
    let flags = MULTI_BYTE_TO_WIDE_CHAR_FLAGS(0);
    let len = unsafe { MultiByteToWideChar(CP_ACP, flags, bytes, None) };
    if len <= 0 {
        return String::from_utf8_lossy(bytes).into_owned();
    }
    let mut wide = vec![0u16; len as usize];
    let written = unsafe { MultiByteToWideChar(CP_ACP, flags, bytes, Some(&mut wide)) };
    String::from_utf16_lossy(&wide[..written.max(0) as usize])
}

/// How many entries a FILEGROUPDESCRIPTOR says it holds (files and folders).
fn descriptor_count(bytes: &[u8]) -> usize {
    bytes
        .get(..4)
        .map_or(0, |b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]) as usize)
}

/// Parses a FILEGROUPDESCRIPTORW (`wide`) or FILEGROUPDESCRIPTORA blob,
/// skipping directories and nameless entries. At most 64 files are returned.
///
/// The structs are packed(1), so fields are read by byte offset:
/// `dwFlags(4) + clsid(16) + sizel(8) + pointl(8) + dwFileAttributes(4)
/// + 3xFILETIME(24) + nFileSizeHigh(4) + nFileSizeLow(4) = 72`, then
/// `cFileName[260]`: UTF-16LE (520 bytes) or ANSI (260 bytes).
fn parse_file_group_descriptor(bytes: &[u8], wide: bool) -> Vec<DescriptorEntry> {
    const FD_FILESIZE: u32 = 0x40;
    const FILE_ATTRIBUTE_DIRECTORY: u32 = 0x10;
    const NAME_OFFSET: usize = 72;
    const NAME_CHARS: usize = 260;
    let entry_size = NAME_OFFSET + NAME_CHARS * if wide { 2 } else { 1 };
    let read_u32 = |o: usize| u32::from_le_bytes([bytes[o], bytes[o + 1], bytes[o + 2], bytes[o + 3]]);

    if bytes.len() < 4 {
        return Vec::new();
    }
    // Larger drops keep their first items; the caller reports the rest.
    let count = (read_u32(0) as usize).min(MAX_DROP_FILES);
    if count == 0 {
        return Vec::new();
    }
    if bytes.len() < 4 + count * entry_size {
        return Vec::new();
    }

    let mut entries = Vec::new();
    for index in 0..count {
        let base = 4 + index * entry_size;
        let flags = read_u32(base);
        if read_u32(base + 36) & FILE_ATTRIBUTE_DIRECTORY != 0 {
            continue;
        }
        let name_bytes = &bytes[base + NAME_OFFSET..base + entry_size];
        let name = if wide {
            let units: Vec<u16> = name_bytes
                .chunks_exact(2)
                .map(|c| u16::from_le_bytes([c[0], c[1]]))
                .take_while(|&c| c != 0)
                .collect();
            String::from_utf16_lossy(&units)
        } else {
            // ANSI names are in the system code page.
            let end = name_bytes.iter().position(|&c| c == 0).unwrap_or(name_bytes.len());
            ansi_to_string(&name_bytes[..end])
        };
        let name = name.trim().to_string();
        if name.is_empty() {
            continue;
        }
        let size = (flags & FD_FILESIZE != 0)
            .then(|| (u64::from(read_u32(base + 64)) << 32) | u64::from(read_u32(base + 68)));
        entries.push(DescriptorEntry { index, name, size });
    }
    entries
}

// --- HTML parsing -----------------------------------------------------------

/// Clamp a byte index to the nearest preceding char boundary.
/// `StartFragment`/`EndFragment` are byte offsets into the original
/// `HTML Format` bytes; after `from_utf8_lossy` the length can shift and the
/// offsets can split UTF-8. Slicing there panics (web-controlled input).
fn floor_char_boundary(s: &str, mut idx: usize) -> usize {
    idx = idx.min(s.len());
    while idx > 0 && !s.is_char_boundary(idx) {
        idx -= 1;
    }
    idx
}

/// Returns (all image srcs in order, optional SourceURL referer).
/// Honors StartFragment/EndFragment offsets, decodes entities, handles
/// //host URLs and srcset. blob: URLs are kept — the caller saves a
/// placeholder since the backend can never resolve them.
fn parse_html_images(html: &str) -> (Vec<String>, Option<String>) {
    let referer = parse_html_source_url(html);
    let (frag_start, frag_end) = parse_fragment_offsets(html);
    let s = floor_char_boundary(html, frag_start.min(html.len()));
    let e = floor_char_boundary(html, frag_end.min(html.len()));
    let frag = if s <= e { &html[s..e] } else { html };

    let mut srcs = Vec::new();
    let mut rest = frag;
    // Collect every <img ... src=...> in order (multi-image drags).
    while let Some(img_idx) = find_insensitive(rest, "<img") {
        let after_img = &rest[img_idx..];
        // End of this tag.
        let tag_end = after_img.find('>').unwrap_or(after_img.len());
        let tag = &after_img[..tag_end];
        if let Some(src) = parse_src_attr(tag) {
            let src = src.trim().to_string();
            if !src.is_empty() {
                srcs.push(normalize_img_src(&src));
            }
        } else if let Some(set_src) = parse_srcset_first(tag) {
            srcs.push(normalize_img_src(&set_src));
        }
        rest = &after_img[tag_end.min(after_img.len())..];
        if rest.is_empty() {
            break;
        }
        // Advance past '>' to avoid re-matching the same tag.
        if rest.starts_with('>') {
            rest = &rest[1..];
        }
        if srcs.len() >= 20 {
            break; // cap: don't materialize 100-image selections
        }
    }
    (srcs, referer)
}

/// Visible text an image drop may carry (captions, alt text) before the drop
/// is treated as a text selection instead.
const MAX_IMAGE_DROP_TEXT: usize = 40;

/// Non-whitespace characters of visible text in the CF_HTML fragment: tags
/// stripped, common entities counted as one character.
fn fragment_text_chars(html: &str) -> usize {
    let (frag_start, frag_end) = parse_fragment_offsets(html);
    let s = floor_char_boundary(html, frag_start.min(html.len()));
    let e = floor_char_boundary(html, frag_end.min(html.len()));
    let frag = if s <= e { &html[s..e] } else { html };
    let mut count = 0;
    let mut in_tag = false;
    let mut in_entity = false;
    for c in frag.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if in_tag => {}
            '&' => {
                in_entity = true;
                count += 1;
            }
            ';' if in_entity => in_entity = false,
            _ if in_entity => {}
            c if !c.is_whitespace() => count += 1,
            _ => {}
        }
    }
    count
}

fn parse_html_source_url(html: &str) -> Option<String> {
    // Header looks like: SourceURL:https://example.com/page
    for line in html.lines().take(12) {
        if let Some(rest) = line.strip_prefix("SourceURL:") {
            let u = rest.trim().to_string();
            if !u.is_empty() {
                return Some(u);
            }
        }
    }
    None
}

fn parse_fragment_offsets(html: &str) -> (usize, usize) {
    let mut start: Option<usize> = None;
    let mut end: Option<usize> = None;
    for line in html.lines().take(12) {
        if let Some(v) = line.strip_prefix("StartFragment:") {
            start = v.trim().parse().ok();
        } else if let Some(v) = line.strip_prefix("EndFragment:") {
            end = v.trim().parse().ok();
        }
    }
    match (start, end) {
        (Some(s), Some(e)) if s < e && e <= html.len() => (s, e),
        _ => (0, html.len()),
    }
}

fn find_insensitive(haystack: &str, needle: &str) -> Option<usize> {
    // ASCII case-insensitive search for "<img".
    let h = haystack.as_bytes();
    let n = needle.as_bytes();
    if n.len() > h.len() {
        return None;
    }
    for i in 0..=h.len() - n.len() {
        let mut ok = true;
        for j in 0..n.len() {
            if !h[i + j].eq_ignore_ascii_case(&n[j]) {
                ok = false;
                break;
            }
        }
        if ok {
            return Some(i);
        }
    }
    None
}

fn parse_src_attr(tag: &str) -> Option<String> {
    // Find src= (but not srcset= / data-src=). Scan case-insensitively.
    let lower = tag.to_ascii_lowercase();
    let bytes = lower.as_bytes();
    let mut i = 0;
    // Inside another attribute's quoted value ("alt='a src=b'"), "src="
    // is text, not an attribute.
    let mut quote: Option<u8> = None;
    while i + 4 < bytes.len() {
        match (quote, bytes[i]) {
            (None, b'"' | b'\'') => {
                quote = Some(bytes[i]);
                i += 1;
                continue;
            }
            (Some(q), b) if b == q => {
                quote = None;
                i += 1;
                continue;
            }
            (Some(_), _) => {
                i += 1;
                continue;
            }
            _ => {}
        }
        if &bytes[i..i + 4] == b"src=" || (i + 5 <= bytes.len() && &bytes[i..i + 5] == b"src =") {
            // Ensure preceding char isn't part of a longer name (e.g. data-src).
            let prev_ok = if i == 0 {
                true
            } else {
                let p = bytes[i - 1];
                !(p.is_ascii_alphanumeric() || p == b'-' || p == b'_')
            };
            if prev_ok {
                let orig = &tag[i..];
                // Skip "src", spaces, '='.
                let mut j = 3;
                while j < orig.len() && orig.as_bytes()[j].is_ascii_whitespace() {
                    j += 1;
                }
                // Handle "src =".
                if j < orig.len() && orig.as_bytes()[j] == b'=' {
                    j += 1;
                } else if i + 5 <= tag.len() && tag[i..].starts_with("src =") {
                    j = 5;
                }
                while j < orig.len() && orig.as_bytes()[j].is_ascii_whitespace() {
                    j += 1;
                }
                let rest = &orig[j..];
                let quote = rest.chars().next()?;
                if quote == '"' || quote == '\'' {
                    let inner = &rest[1..];
                    if let Some(end) = inner.find(quote) {
                        return Some(html_unescape(&inner[..end]));
                    }
                    return None;
                }
                // Unquoted src.
                let end = rest
                    .find(|c: char| c.is_whitespace() || c == '>')
                    .unwrap_or(rest.len());
                return Some(html_unescape(rest[..end].trim()));
            }
        }
        i += 1;
    }
    None
}

fn parse_srcset_first(tag: &str) -> Option<String> {
    let lower = tag.to_ascii_lowercase();
    let idx = lower.find("srcset=")?;
    let mut rest = tag[idx + 7..].trim_start().to_string();
    if rest.starts_with('=') {
        rest = rest[1..].trim_start().to_string();
    }
    let quote = rest.chars().next()?;
    let inner = if quote == '"' || quote == '\'' {
        rest[1..].split(quote).next()?.to_string()
    } else {
        rest.split_whitespace().next()?.to_string()
    };
    // srcset: "url1 1x, url2 2x" — take first URL.
    let first = inner.split(',').next()?.split_whitespace().next()?;
    if first.is_empty() {
        None
    } else {
        Some(html_unescape(first))
    }
}

fn html_unescape(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
}

fn normalize_img_src(src: &str) -> String {
    if src.starts_with("//") {
        format!("https:{}", src)
    } else {
        src.to_string()
    }
}

// --- Drop handlers ----------------------------------------------------------

fn handle_image_bytes(
    bytes: Vec<u8>,
    suggested_name: String,
    mut report: DropReport,
    app_handle: AppHandle,
) {
    tauri::async_runtime::spawn(async move {
        let mut paths = Vec::new();
        match crate::commands::file_ops::save_bytes_to_drop_folder(&bytes, &suggested_name, "png") {
            Ok(path) => paths.push(PathBuf::from(path)),
            Err(e) => {
                error!("Failed to save dropped image bytes: {}", e);
                report.add(save_issue(&e), 1);
            }
        }
        add_to_shelf(paths, report, app_handle);
    });
}

fn handle_image_srcs(
    srcs: Vec<String>,
    referer: Option<String>,
    mut report: DropReport,
    app_handle: AppHandle,
) {
    use crate::commands::file_ops::{
        download_image_to_shelf, ext_from_image_mime, mark_from_internet,
        save_bytes_to_drop_folder, save_url_placeholder, FetchError, Fetched,
    };
    tauri::async_runtime::spawn(async move {
        let mut paths = Vec::new();
        for src in srcs {
            if let Some(data) = src.strip_prefix("data:image/") {
                let Some((header, b64)) = data.split_once(',') else {
                    report.add(DropIssue::Unsupported, 1);
                    continue;
                };
                let ext = ext_from_image_mime(&format!("image/{header}")).unwrap_or("png");
                let Ok(bytes) = base64::Engine::decode(
                    &base64::engine::general_purpose::STANDARD,
                    b64.trim(),
                ) else {
                    report.add(DropIssue::Unsupported, 1);
                    continue;
                };
                match save_bytes_to_drop_folder(&bytes, "drop", ext) {
                    Ok(p) => {
                        mark_from_internet(std::path::Path::new(&p), None, referer.as_deref());
                        paths.push(PathBuf::from(p))
                    }
                    Err(e) => {
                        error!("Failed to save data: URL image: {}", e);
                        report.add(save_issue(&e), 1);
                    }
                }
            } else if src.starts_with("http://") || src.starts_with("https://") {
                match download_image_to_shelf(src.clone(), referer.clone()).await {
                    Ok(Fetched::Image(p)) => paths.push(PathBuf::from(p)),
                    Ok(Fetched::Link(p)) => {
                        report.add(DropIssue::SavedAsLink, 1);
                        paths.push(PathBuf::from(p));
                    }
                    Err(FetchError::Save(e)) => {
                        error!("Failed to save a downloaded image: {}", e);
                        report.add(save_issue(&e), 1);
                    }
                    Err(FetchError::Download(e)) => {
                        // Offline, timed out or too large: keep the link so
                        // the drop is never silently lost.
                        warn!("Failed to download dropped URL, saving a link instead: {}", e);
                        match save_url_placeholder(&src) {
                            Ok(p) => {
                                report.add(DropIssue::SavedAsLink, 1);
                                paths.push(PathBuf::from(p));
                            }
                            Err(e) => {
                                error!("Failed to save link placeholder: {}", e);
                                report.add(save_issue(&e), 1);
                            }
                        }
                    }
                }
            } else {
                // blob: and other page-local images exist only inside the
                // page they were dragged from; a link to them opens nothing.
                warn!("Ignoring an image only its page can read");
                report.add(DropIssue::Unsupported, 1);
            }
        }
        add_to_shelf(paths, report, app_handle);
    });
}

/// A dropped link. Only links that point straight at an image are fetched;
/// everything else is kept as a `.url` shortcut without contacting the site,
/// so dropping a link never visits it (one-time or sign-in links stay
/// unused, and the site never sees a request from this machine).
fn handle_link_drop(url: String, mut report: DropReport, app_handle: AppHandle) {
    if is_image_url(&url) {
        handle_image_srcs(vec![url], None, report, app_handle);
        return;
    }
    tauri::async_runtime::spawn(async move {
        let mut paths = Vec::new();
        match crate::commands::file_ops::save_url_placeholder(&url) {
            Ok(path) => paths.push(PathBuf::from(path)),
            Err(e) => {
                error!("Failed to save dropped link: {}", e);
                report.add(save_issue(&e), 1);
            }
        }
        add_to_shelf(paths, report, app_handle);
    });
}

/// True for http(s) URLs whose path ends in a common image extension.
fn is_image_url(url: &str) -> bool {
    const IMAGE_EXTS: [&str; 8] = ["png", "jpg", "jpeg", "gif", "webp", "svg", "avif", "bmp"];
    url::Url::parse(url)
        .ok()
        .filter(|u| matches!(u.scheme(), "http" | "https"))
        .and_then(|u| {
            let last = u.path_segments()?.next_back()?.to_string();
            let ext = std::path::Path::new(&last)
                .extension()?
                .to_str()?
                .to_ascii_lowercase();
            Some(IMAGE_EXTS.contains(&ext.as_str()))
        })
        .unwrap_or(false)
}

/// True when dropped text is exactly one http(s) URL. A paragraph that merely
/// starts with a link stays text, so it lands on the shelf as a note.
fn is_single_web_url(text: &str) -> bool {
    !text.contains(char::is_whitespace)
        && url::Url::parse(text)
            .map(|u| matches!(u.scheme(), "http" | "https") && u.host().is_some())
            .unwrap_or(false)
}

fn handle_text_drop(text: String, mut report: DropReport, app_handle: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let mut paths = Vec::new();
        match crate::commands::file_ops::save_pasted_text(text, "txt".to_string(), None) {
            Ok(path) => paths.push(PathBuf::from(path)),
            Err(e) => {
                error!("Failed to save dropped text: {}", e);
                report.add(save_issue(&e), 1);
            }
        }
        add_to_shelf(paths, report, app_handle);
    });
}

// --- DIB -> PNG -------------------------------------------------------------

/// Convert raw CF_DIB(V5) bytes (BITMAPINFOHEADER + pixels) to PNG bytes.
/// Supports the common screenshot cases: 32-bit and 24-bit BI_RGB/BI_BITFIELDS.
/// Returns None for paletted/compressed formats (caller falls through).
fn dib_bytes_to_png(dib: &[u8]) -> Option<Vec<u8>> {
    if dib.len() < 40 {
        return None;
    }
    let read_u32 = |o: usize| u32::from_le_bytes([dib[o], dib[o + 1], dib[o + 2], dib[o + 3]]);
    let read_i32 = |o: usize| i32::from_le_bytes([dib[o], dib[o + 1], dib[o + 2], dib[o + 3]]);
    let read_u16 = |o: usize| u16::from_le_bytes([dib[o], dib[o + 1]]);

    let header_size = read_u32(0) as usize;
    if header_size < 40 || header_size > dib.len() {
        return None;
    }
    let width = read_i32(4);
    let height_raw = read_i32(8);
    let planes = read_u16(12);
    let bit_count = read_u16(14);
    let compression = read_u32(16);

    if width <= 0 || width > 16384 || height_raw == 0 || height_raw.unsigned_abs() > 16384 {
        return None;
    }
    if planes != 1 {
        return None;
    }
    // BI_RGB=0, BI_BITFIELDS=3. Others (RLE, PNG-in-DIB, etc.) unsupported.
    if compression != 0 && compression != 3 {
        return None;
    }
    let width_u = width as u32;
    let height_u = height_raw.unsigned_abs();
    let top_down = height_raw < 0;

    // Palette size for <=8bpp — we don't support paletted, bail early.
    if bit_count <= 8 {
        return None;
    }

    // For BI_BITFIELDS with 16/32bpp there are 3 color masks after a plain
    // BITMAPINFOHEADER. V4/V5 headers (108/124 bytes) hold the masks inside
    // the header, so nothing extra follows them.
    let masks_len = if header_size == 40 && compression == 3 && (bit_count == 16 || bit_count == 32)
    {
        12
    } else {
        0
    };
    let pixels_offset = header_size + masks_len;
    if pixels_offset >= dib.len() {
        return None;
    }

    // Stride is DWORD-aligned.
    let stride = (width_u as usize * bit_count as usize).div_ceil(32) * 4;
    let needed = stride.checked_mul(height_u as usize)?;
    if dib.len() < pixels_offset + needed {
        return None;
    }

    let mut rgba = Vec::with_capacity((width_u * height_u * 4) as usize);
    match bit_count {
        32 => {
            // For 32bpp BI_RGB the 4th byte is undefined — most sources leave
            // it 0, which would make the PNG fully transparent. If every
            // alpha byte is 0, treat the image as opaque instead.
            let mut any_alpha = false;
            for row in 0..height_u as usize {
                let src_row = if top_down {
                    row
                } else {
                    height_u as usize - 1 - row
                };
                let base = pixels_offset + src_row * stride;
                for col in 0..width_u as usize {
                    let o = base + col * 4;
                    let b = dib[o];
                    let g = dib[o + 1];
                    let r = dib[o + 2];
                    let a = dib[o + 3];
                    if a != 0 {
                        any_alpha = true;
                    }
                    rgba.extend_from_slice(&[r, g, b, a]);
                }
            }
            if !any_alpha {
                for px in rgba.chunks_mut(4) {
                    px[3] = 255;
                }
            }
        }
        24 => {
            for row in 0..height_u as usize {
                let src_row = if top_down {
                    row
                } else {
                    height_u as usize - 1 - row
                };
                let base = pixels_offset + src_row * stride;
                for col in 0..width_u as usize {
                    let o = base + col * 3;
                    let b = dib[o];
                    let g = dib[o + 1];
                    let r = dib[o + 2];
                    rgba.extend_from_slice(&[r, g, b, 255]);
                }
            }
        }
        _ => return None,
    }

    let img = image::RgbaImage::from_raw(width_u, height_u, rgba)?;
    let dyn_img = image::DynamicImage::ImageRgba8(img);
    let mut png = Vec::new();
    {
        use std::io::Cursor;
        let mut cursor = Cursor::new(&mut png);
        dyn_img
            .write_to(&mut cursor, image::ImageFormat::Png)
            .ok()?;
    }
    if png.is_empty() {
        None
    } else {
        Some(png)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_single_img_src_with_entities() {
        let html = "Version:0.9\r\nStartFragment:00000080\r\nEndFragment:00000200\r\nSourceURL:https://example.com/p\r\n<html><body><!--StartFragment--><img src=\"https://example.com/a.png?x=1&amp;y=2\"><!--EndFragment--></body></html>";
        let (srcs, referer) = parse_html_images(html);
        assert_eq!(srcs.len(), 1);
        assert!(srcs[0].contains("x=1&y=2"));
        assert_eq!(referer.as_deref(), Some("https://example.com/p"));
    }

    #[test]
    fn src_inside_another_attribute_is_ignored() {
        let tag = r#"<img alt="logo src=evil.png" src="https://real.test/a.png">"#;
        assert_eq!(parse_src_attr(tag).as_deref(), Some("https://real.test/a.png"));
        let tag = "<img title='x src=y' data-src=\"z\" src=https://ok.test/b.png>";
        assert_eq!(parse_src_attr(tag).as_deref(), Some("https://ok.test/b.png"));
    }

    #[test]
    fn collects_multiple_img_srcs() {
        let html = "<img src=\"https://a.com/1.png\"><p>x</p><IMG SRC='https://b.com/2.jpg'>";
        let (srcs, _) = parse_html_images(html);
        assert_eq!(srcs.len(), 2);
    }

    #[test]
    fn fragment_slice_never_panics_on_split_utf8() {
        // `é` is 2 bytes in UTF-8; offsets splitting it must clamp, not panic.
        let html = "aaé<img src=\"https://a.com/1.png\">";
        let mid = html.find('é').unwrap() + 1; // middle of `é`
        let s = floor_char_boundary(html, mid);
        let e = floor_char_boundary(html, mid);
        assert!(html.is_char_boundary(s));
        assert!(html.is_char_boundary(e));
        let _ = &html[s..e];

        // End-to-end: bogus fragment offsets from drag source can't crash us.
        let evil = format!(
            "Version:0.9\r\nStartFragment:{:08}\r\nEndFragment:{:08}\r\n{}",
            mid, mid, html
        );
        let (srcs, _) = parse_html_images(&evil);
        assert!(srcs.len() <= 1);
    }

    /// Builds a FILEGROUPDESCRIPTOR(W/A) blob from (name, attributes, size).
    fn descriptor(wide: bool, files: &[(&str, u32, Option<u64>)]) -> Vec<u8> {
        let entry_size = 72 + 260 * if wide { 2 } else { 1 };
        let mut out = (files.len() as u32).to_le_bytes().to_vec();
        for (name, attrs, size) in files {
            let mut entry = vec![0u8; entry_size];
            let mut flags = 0x4u32; // FD_ATTRIBUTES
            if let Some(size) = size {
                flags |= 0x40; // FD_FILESIZE
                entry[64..68].copy_from_slice(&((size >> 32) as u32).to_le_bytes());
                entry[68..72].copy_from_slice(&(*size as u32).to_le_bytes());
            }
            entry[0..4].copy_from_slice(&flags.to_le_bytes());
            entry[36..40].copy_from_slice(&attrs.to_le_bytes());
            if wide {
                for (i, unit) in name.encode_utf16().enumerate() {
                    entry[72 + i * 2..74 + i * 2].copy_from_slice(&unit.to_le_bytes());
                }
            } else {
                entry[72..72 + name.len()].copy_from_slice(name.as_bytes());
            }
            out.extend_from_slice(&entry);
        }
        out
    }

    #[test]
    fn parses_wide_descriptor_skipping_directories() {
        let bytes = descriptor(
            true,
            &[
                ("Folder", 0x10, None),
                ("Folder\\Résumé.pdf", 0x20, Some(5_000_000_000)),
                ("notes.txt", 0x20, None),
            ],
        );
        let entries = parse_file_group_descriptor(&bytes, true);
        assert_eq!(
            entries,
            vec![
                DescriptorEntry {
                    index: 1,
                    name: "Folder\\Résumé.pdf".to_string(),
                    size: Some(5_000_000_000),
                },
                DescriptorEntry { index: 2, name: "notes.txt".to_string(), size: None },
            ]
        );
    }

    #[test]
    fn parses_ansi_descriptor() {
        let bytes = descriptor(false, &[("Meeting notes.msg", 0x20, Some(1234))]);
        let entries = parse_file_group_descriptor(&bytes, false);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "Meeting notes.msg");
        assert_eq!(entries[0].size, Some(1234));
        // The same bytes are too short to be a wide descriptor.
        assert!(parse_file_group_descriptor(&bytes, true).is_empty());
    }

    #[test]
    fn rejects_truncated_or_empty_descriptors() {
        assert!(parse_file_group_descriptor(&[], true).is_empty());
        assert!(parse_file_group_descriptor(&0u32.to_le_bytes(), true).is_empty());
        let mut bytes = descriptor(true, &[("a.txt", 0, None)]);
        bytes.truncate(bytes.len() - 1);
        assert!(parse_file_group_descriptor(&bytes, true).is_empty());
    }

    /// A 1x1 32bpp BI_BITFIELDS bitmap with the given header size and one
    /// BGRA pixel; a 40-byte header is followed by the 3 colour masks.
    fn bitfields_dib(header_size: usize, pixel: [u8; 4]) -> Vec<u8> {
        let mut dib = vec![0u8; header_size];
        dib[0..4].copy_from_slice(&(header_size as u32).to_le_bytes());
        dib[4..8].copy_from_slice(&1i32.to_le_bytes());
        dib[8..12].copy_from_slice(&1i32.to_le_bytes());
        dib[12..14].copy_from_slice(&1u16.to_le_bytes());
        dib[14..16].copy_from_slice(&32u16.to_le_bytes());
        dib[16..20].copy_from_slice(&3u32.to_le_bytes());
        if header_size == 40 {
            dib.extend_from_slice(&[0, 0, 0xFF, 0, 0, 0xFF, 0, 0, 0xFF, 0, 0, 0]);
        }
        dib.extend_from_slice(&pixel);
        dib
    }

    fn first_pixel(png: &[u8]) -> [u8; 4] {
        image::load_from_memory(png).unwrap().to_rgba8().get_pixel(0, 0).0
    }

    #[test]
    fn dib_bitfields_pixel_offset_depends_on_header_version() {
        let pixel = [10, 20, 30, 255]; // BGRA
        for header_size in [40, 124] {
            let png = dib_bytes_to_png(&bitfields_dib(header_size, pixel))
                .unwrap_or_else(|| panic!("{header_size}-byte header should decode"));
            assert_eq!(first_pixel(&png), [30, 20, 10, 255], "header {header_size}");
        }
    }

    #[test]
    fn text_selections_with_images_stay_text() {
        let img_only = "StartFragment:0\r\n<!--StartFragment--><img src=\"https://x.test/a.png\"><!--EndFragment-->";
        assert!(fragment_text_chars(img_only) <= MAX_IMAGE_DROP_TEXT);
        let prose = format!(
            "<!--StartFragment--><p>{} <img src=\"https://x.test/e.png\"></p><!--EndFragment-->",
            "This is a paragraph of an email that has an emoji image inside it."
        );
        assert!(fragment_text_chars(&prose) > MAX_IMAGE_DROP_TEXT);
    }

    #[test]
    fn only_links_to_images_are_fetched() {
        assert!(is_image_url("https://example.com/a/photo.JPG?w=300"));
        assert!(is_image_url("http://cdn.example.com/x.webp"));
        assert!(!is_image_url("https://example.com/login?token=abc"));
        assert!(!is_image_url("https://example.com/"));
        assert!(!is_image_url("https://example.com/page.html"));
        assert!(!is_image_url("ftp://example.com/a.png"));
    }

    #[test]
    fn only_a_lone_url_counts_as_a_link_drop() {
        assert!(is_single_web_url("https://example.com/a?b=1"));
        assert!(is_single_web_url("http://localhost:3000"));
        assert!(!is_single_web_url("https://foo.com is great\nmore text"));
        assert!(!is_single_web_url("https://a.com https://b.com"));
        assert!(!is_single_web_url("ftp://example.com/file"));
        assert!(!is_single_web_url("https://"));
        assert!(!is_single_web_url("just some notes"));
    }

    #[test]
    fn a_dropped_storage_becomes_a_compound_file() {
        use windows::Win32::System::Com::StructuredStorage::{
            CreateILockBytesOnHGlobal, StgCreateDocfileOnILockBytes,
        };
        use windows::Win32::System::Com::{
            CoInitializeEx, COINIT_APARTMENTTHREADED, STGM_CREATE, STGM_READWRITE,
            STGM_SHARE_EXCLUSIVE,
        };
        unsafe {
            let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
            let memory = CreateILockBytesOnHGlobal(None, true).expect("memory");
            let source = StgCreateDocfileOnILockBytes(
                &memory,
                STGM_CREATE | STGM_READWRITE | STGM_SHARE_EXCLUSIVE,
                0,
            )
            .expect("source storage");
            let stream = source
                .CreateStream(
                    windows::core::w!("__substg1.0_0037001F"),
                    STGM_CREATE | STGM_READWRITE | STGM_SHARE_EXCLUSIVE,
                    0,
                    0,
                )
                .expect("stream");
            let subject: Vec<u8> = "Hello".encode_utf16().flat_map(u16::to_le_bytes).collect();
            stream
                .Write(subject.as_ptr().cast(), subject.len() as u32, None)
                .ok()
                .expect("write");
            drop(stream);
            let bytes = storage_to_bytes(&source).expect("copied");
            // The compound-file signature every .msg file starts with.
            assert_eq!(&bytes[..8], &[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1]);
            assert!(bytes.len() >= 512);
        }
    }

    #[test]
    fn dib_rejects_extreme_dimensions_without_panic() {
        // i32::MIN must not panic via abs(); unsigned_abs path rejects it.
        // Minimal DIB with hostile dims returns None instead of allocating.
        let mut dib = vec![0u8; 40];
        dib[0] = 40; // header_size
        dib[4..8].copy_from_slice(&20000i32.to_le_bytes()); // width too big
        dib[8..12].copy_from_slice(&1i32.to_le_bytes());
        assert!(dib_bytes_to_png(&dib).is_none());
        dib[4..8].copy_from_slice(&1i32.to_le_bytes());
        dib[8..12].copy_from_slice(&i32::MIN.to_le_bytes());
        assert!(dib_bytes_to_png(&dib).is_none());
    }
}
