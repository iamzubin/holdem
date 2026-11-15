use crate::{DragState, FileList};
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
    CF_UNICODETEXT, DROPEFFECT, DROPEFFECT_COPY,
};
use windows::Win32::System::SystemServices::MODIFIERKEYS_FLAGS;
use windows::Win32::UI::Shell::{DragQueryFileW, HDROP};
use windows::Win32::UI::WindowsAndMessaging::EnumChildWindows;

/// Total bytes accepted from one virtual-file drop. Each file is already
/// capped at 64 MB while reading; this stops a multi-file drop (up to 64
/// files) from buffering gigabytes on the UI thread.
const MAX_VIRTUAL_DROP_BYTES: usize = 256 * 1024 * 1024;

#[implement(IDropTarget)]
pub struct HoldemDropTarget {
    app_handle: AppHandle,
}

impl HoldemDropTarget {
    pub fn new(app_handle: AppHandle) -> Self {
        Self { app_handle }
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
                return None;
            }
            if read == 0 {
                break;
            }
            out.extend_from_slice(&buf[..read as usize]);
            if out.len() > 64 * 1024 * 1024 {
                warn!("Virtual file stream exceeds 64 MB cap, rejecting");
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
    unsafe fn extract_virtual_files(
        &self,
        data_obj: &IDataObject,
    ) -> Option<Vec<(String, Vec<u8>)>> {
        let cf_contents = self.clipboard_format("FileContents")?;
        let entries = [("FileGroupDescriptorW", true), ("FileGroupDescriptor", false)]
            .into_iter()
            .find_map(|(format, wide)| {
                let cf = self.clipboard_format(format)?;
                let bytes = self.get_data_bytes(data_obj, cf, -1)?;
                let entries = parse_file_group_descriptor(&bytes, wide);
                (!entries.is_empty()).then_some(entries)
            })?;

        let mut out = Vec::new();
        let mut total = 0usize;
        for entry in entries {
            // FileContents per lindex: try IStream then HGLOBAL.
            let Some(mut bytes) = self.get_data_bytes(data_obj, cf_contents, entry.index as i32)
            else {
                warn!("No contents for dropped virtual file {}", entry.name);
                continue;
            };
            // HGLOBAL blocks can be larger than the file; trust the declared size.
            if let Some(size) = entry.size {
                if (size as usize) < bytes.len() {
                    bytes.truncate(size as usize);
                }
            }
            total += bytes.len();
            if total > MAX_VIRTUAL_DROP_BYTES {
                warn!(
                    "Virtual drop exceeds {} MB in total, skipping the remaining files",
                    MAX_VIRTUAL_DROP_BYTES / (1024 * 1024)
                );
                break;
            }
            out.push((entry.name, bytes));
        }
        if out.is_empty() {
            None
        } else {
            Some(out)
        }
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
        if let Some(drag_state) = self.app_handle.try_state::<Arc<DragState>>() {
            drag_state.drag_started.store(true, Ordering::Relaxed);
        }
        set_drop_hover(&self.app_handle, true);
        unsafe {
            if !pdweffect.is_null() {
                *pdweffect = DROPEFFECT_COPY;
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
                *pdweffect = DROPEFFECT_COPY;
            }
        }
        Ok(())
    }

    fn DragLeave(&self) -> windows::core::Result<()> {
        if let Some(drag_state) = self.app_handle.try_state::<Arc<DragState>>() {
            drag_state.drag_started.store(false, Ordering::Relaxed);
        }
        set_drop_hover(&self.app_handle, false);
        Ok(())
    }

    fn Drop(
        &self,
        pdataobj: windows_core::Ref<'_, IDataObject>,
        _grfkeystate: MODIFIERKEYS_FLAGS,
        _pt: &POINTL,
        pdweffect: *mut DROPEFFECT,
    ) -> windows::core::Result<()> {
        set_drop_hover(&self.app_handle, false);
        unsafe {
            if !pdweffect.is_null() {
                *pdweffect = DROPEFFECT_COPY;
            }

            if let Some(dataobj) = pdataobj.as_ref() {
                // 1. Real filesystem files — lossless, no copy.
                if let Some(paths) = self.extract_files(dataobj) {
                    info!("HoldemDropTarget received {} file path(s)", paths.len());
                    mark_drop(&self.app_handle);
                    if let Some(file_list) = self.app_handle.try_state::<FileList>() {
                        crate::file_drop::handle_file_drop_from_paths(
                            paths,
                            file_list.inner().clone(),
                            self.app_handle.clone(),
                        );
                    }
                    return Ok(());
                }

                // 2. Virtual files (Outlook/Discord/Slack/ZIP): FileGroupDescriptorW.
                if let Some(virtual_files) = self.extract_virtual_files(dataobj) {
                    info!(
                        "HoldemDropTarget received {} virtual file(s)",
                        virtual_files.len()
                    );
                    mark_drop(&self.app_handle);
                    let app = self.app_handle.clone();
                    tauri::async_runtime::spawn(async move {
                        let mut paths = Vec::new();
                        for (name, bytes) in virtual_files {
                            let ext = std::path::Path::new(&name)
                                .extension()
                                .and_then(|e| e.to_str())
                                .unwrap_or("bin");
                            match crate::commands::file_ops::save_bytes_to_drop_folder(
                                &bytes, &name, ext,
                            ) {
                                Ok(p) => paths.push(PathBuf::from(p)),
                                Err(e) => error!("Failed to save virtual file {}: {}", name, e),
                            }
                        }
                        if !paths.is_empty() {
                            if let Some(file_list) = app.try_state::<FileList>() {
                                crate::file_drop::handle_file_drop_from_paths(
                                    paths,
                                    file_list.inner().clone(),
                                    app,
                                );
                            }
                        }
                    });
                    return Ok(());
                }

                // 3. Bitmaps — prefer lossless pixels over re-downloading.
                //    PNG format first (already encoded), then CF_DIB(V5).
                if let Some(png_bytes) = self.extract_png_bytes(dataobj) {
                    // Validate it actually looks like a PNG before accepting.
                    if png_bytes.len() > 8 && &png_bytes[..8] == b"\x89PNG\r\n\x1a\n" {
                        info!("HoldemDropTarget received PNG clipboard image");
                        mark_drop(&self.app_handle);
                        handle_image_bytes(
                            png_bytes,
                            "screenshot.png".to_string(),
                            self.app_handle.clone(),
                        );
                        return Ok(());
                    }
                }
                if let Some(png_bytes) = self.extract_dib_png_bytes(dataobj) {
                    info!("HoldemDropTarget received DIB bitmap image");
                    mark_drop(&self.app_handle);
                    handle_image_bytes(
                        png_bytes,
                        "screenshot.png".to_string(),
                        self.app_handle.clone(),
                    );
                    return Ok(());
                }

                // 4. HTML (browser <img> drags): all images + SourceURL referer.
                if let Some(html) = self.extract_html_raw(dataobj) {
                    let (srcs, referer) = parse_html_images(&html);
                    if !srcs.is_empty() {
                        info!(
                            "HoldemDropTarget detected {} image(s) in dropped HTML",
                            srcs.len()
                        );
                        mark_drop(&self.app_handle);
                        handle_image_srcs(srcs, referer, self.app_handle.clone());
                        return Ok(());
                    }
                    // HTML without images (rich text) — fall through to URL/text.
                }

                // 5. Explicit URL flavors (link / address-bar drags).
                if let Some(url) = self.extract_url(dataobj) {
                    let url = url.trim().to_string();
                    if !url.is_empty() {
                        info!("HoldemDropTarget detected dropped URL: {}", url);
                        mark_drop(&self.app_handle);
                        handle_image_srcs(vec![url], None, self.app_handle.clone());
                        return Ok(());
                    }
                }

                // 6. Plain text last resort.
                if let Some(text) =
                    self.extract_unicode_text_with_format(dataobj, CF_UNICODETEXT.0)
                {
                    let text = text.trim();
                    if !text.is_empty() {
                        mark_drop(&self.app_handle);
                        if is_single_web_url(text) {
                            info!("HoldemDropTarget detected URL text: {}", text);
                            handle_image_srcs(
                                vec![text.to_string()],
                                None,
                                self.app_handle.clone(),
                            );
                        } else {
                            info!("HoldemDropTarget saving dropped text snippet");
                            handle_text_drop(text.to_string(), self.app_handle.clone());
                        }
                        return Ok(());
                    }
                }
            }
        }
        Ok(())
    }
}

/// Tells the shelf whether something is being dragged over it, so it can
/// highlight. The native drop target replaces WebView2's, so the page never
/// sees HTML drag events itself.
fn set_drop_hover(app_handle: &AppHandle, hovering: bool) {
    if let Err(e) = app_handle.emit_to("main", "drop-hover", hovering) {
        warn!("Failed to emit drop-hover: {}", e);
    }
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

fn register_on_hwnd(app_handle: &AppHandle, hwnd: HWND) {
    unsafe {
        let _ = RevokeDragDrop(hwnd);
        let target = HoldemDropTarget::new(app_handle.clone());
        let target_interface: IDropTarget = target.into();
        if let Err(e) = RegisterDragDrop(hwnd, &target_interface) {
            warn!("RegisterDragDrop failed for {:?}: {:?}", hwnd, e);
        }
    }
}

pub fn register_main_drop_target(app_handle: &AppHandle, hwnd: HWND) {
    // Top-level window first…
    register_on_hwnd(app_handle, hwnd);
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
        register_on_hwnd(app_handle, child);
    }
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
    let count = read_u32(0) as usize;
    if count == 0 || count > 512 {
        return Vec::new();
    }
    let count = count.min(64);
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
            // ANSI names are in the system code page; non-ASCII characters
            // may not survive, but the file contents are unaffected.
            let end = name_bytes.iter().position(|&c| c == 0).unwrap_or(name_bytes.len());
            String::from_utf8_lossy(&name_bytes[..end]).into_owned()
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
    while i + 4 < bytes.len() {
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

fn handle_image_bytes(bytes: Vec<u8>, suggested_name: String, app_handle: AppHandle) {
    tauri::async_runtime::spawn(async move {
        match crate::commands::file_ops::save_bytes_to_drop_folder(
            &bytes,
            &suggested_name,
            "png",
        )
        {
            Ok(path) => {
                if let Some(file_list) = app_handle.try_state::<FileList>() {
                    crate::file_drop::handle_file_drop_from_paths(
                        vec![PathBuf::from(path)],
                        file_list.inner().clone(),
                        app_handle,
                    );
                }
            }
            Err(e) => error!("Failed to save dropped image bytes: {}", e),
        }
    });
}

fn handle_image_srcs(srcs: Vec<String>, referer: Option<String>, app_handle: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let mut paths = Vec::new();
        for src in srcs {
            if src.starts_with("data:image/") {
                if let Some(comma) = src.find(',') {
                    let header = &src[..comma];
                    let b64 = &src[comma + 1..];
                    let ext = crate::commands::file_ops::ext_from_image_mime(header)
                        .unwrap_or("png");
                    match crate::commands::file_ops::save_pasted_data_base64(
                        b64.to_string(),
                        ext.to_string(),
                        None,
                    ) {
                        Ok(p) => paths.push(PathBuf::from(p)),
                        Err(e) => error!("Failed to save data: URL image: {}", e),
                    }
                }
            } else if src.starts_with("blob:") {
                // Backend cannot resolve blob: — save placeholder; the thin
                // frontend forwarder handles the real bytes when it can.
                match crate::commands::file_ops::save_url_placeholder(&src) {
                    Ok(p) => paths.push(PathBuf::from(p)),
                    Err(e) => error!("Failed to save blob placeholder: {}", e),
                }
            } else if src.starts_with("http://") || src.starts_with("https://") {
                match crate::commands::file_ops::download_image_to_shelf(
                    src.clone(),
                    referer.clone(),
                )
                .await
                {
                    Ok(p) => paths.push(PathBuf::from(p)),
                    Err(e) => {
                        // Offline, timed out or too large: keep the link so
                        // the drop is never silently lost.
                        warn!("Failed to download dropped URL, saving a link instead: {}", e);
                        match crate::commands::file_ops::save_url_placeholder(&src) {
                            Ok(p) => paths.push(PathBuf::from(p)),
                            Err(e) => error!("Failed to save link placeholder: {}", e),
                        }
                    }
                }
            } else {
                warn!("Ignoring unsupported dropped image src");
            }
        }
        if !paths.is_empty() {
            if let Some(file_list) = app_handle.try_state::<FileList>() {
                crate::file_drop::handle_file_drop_from_paths(
                    paths,
                    file_list.inner().clone(),
                    app_handle,
                );
            }
        }
    });
}

/// True when dropped text is exactly one http(s) URL. A paragraph that merely
/// starts with a link stays text, so it lands on the shelf as a note.
fn is_single_web_url(text: &str) -> bool {
    !text.contains(char::is_whitespace)
        && url::Url::parse(text)
            .map(|u| matches!(u.scheme(), "http" | "https") && u.host().is_some())
            .unwrap_or(false)
}

fn handle_text_drop(text: String, app_handle: AppHandle) {
    tauri::async_runtime::spawn(async move {
        match crate::commands::file_ops::save_pasted_text(text, "txt".to_string(), None) {
            Ok(path) => {
                if let Some(file_list) = app_handle.try_state::<FileList>() {
                    crate::file_drop::handle_file_drop_from_paths(
                        vec![PathBuf::from(path)],
                        file_list.inner().clone(),
                        app_handle,
                    );
                }
            }
            Err(e) => error!("Failed to save dropped text: {}", e),
        }
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
