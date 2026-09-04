use crate::{DragState, FileList};
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tauri::{AppHandle, Manager};
use tracing::{error, info};
use windows::core::{implement, w};
use windows::Win32::Foundation::{HWND, POINTL};
use windows::Win32::System::Com::{
    IDataObject, DVASPECT_CONTENT, FORMATETC, TYMED_HGLOBAL,
};
use windows::Win32::System::Ole::ReleaseStgMedium;
use windows::Win32::System::DataExchange::RegisterClipboardFormatW;
use windows::Win32::System::Memory::{GlobalLock, GlobalSize, GlobalUnlock};
use windows::Win32::System::Ole::{
    IDropTarget, IDropTarget_Impl, RegisterDragDrop, RevokeDragDrop, CF_HDROP, DROPEFFECT,
    DROPEFFECT_COPY,
};
use windows::Win32::System::SystemServices::MODIFIERKEYS_FLAGS;
use windows::Win32::UI::Shell::{DragFinish, DragQueryFileW, HDROP};

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

            DragFinish(hdrop);
            ReleaseStgMedium(&mut medium);

            if !paths.is_empty() {
                return Some(paths);
            }
        }
        None
    }

    unsafe fn extract_html(&self, data_obj: &IDataObject) -> Option<String> {
        let html_fmt_id = RegisterClipboardFormatW(w!("HTML Format"));
        if html_fmt_id == 0 {
            return None;
        }

        let fmt = FORMATETC {
            cfFormat: html_fmt_id as u16,
            ptd: std::ptr::null_mut(),
            dwAspect: DVASPECT_CONTENT.0,
            lindex: -1,
            tymed: TYMED_HGLOBAL.0 as u32,
        };

        if let Ok(mut medium) = data_obj.GetData(&fmt) {
            let hglobal = medium.u.hGlobal;
            let mut text = None;
            if !hglobal.is_invalid() {
                let ptr = GlobalLock(hglobal) as *const u8;
                if !ptr.is_null() {
                    let size = GlobalSize(hglobal);
                    let slice = std::slice::from_raw_parts(ptr, size);
                    let end = slice.iter().position(|&c| c == 0).unwrap_or(slice.len());
                    text = Some(String::from_utf8_lossy(&slice[..end]).into_owned());
                    let _ = GlobalUnlock(hglobal);
                }
            }
            ReleaseStgMedium(&mut medium);
            return text;
        }
        None
    }

    unsafe fn extract_text(&self, data_obj: &IDataObject) -> Option<String> {
        let fmt = FORMATETC {
            cfFormat: 13, // CF_UNICODETEXT
            ptd: std::ptr::null_mut(),
            dwAspect: DVASPECT_CONTENT.0,
            lindex: -1,
            tymed: TYMED_HGLOBAL.0 as u32,
        };

        if let Ok(mut medium) = data_obj.GetData(&fmt) {
            let hglobal = medium.u.hGlobal;
            let mut text = None;
            if !hglobal.is_invalid() {
                let ptr = GlobalLock(hglobal) as *const u16;
                if !ptr.is_null() {
                    let size = GlobalSize(hglobal);
                    let count = size / 2;
                    let slice = std::slice::from_raw_parts(ptr, count);
                    let end = slice.iter().position(|&c| c == 0).unwrap_or(slice.len());
                    text = Some(String::from_utf16_lossy(&slice[..end]));
                    let _ = GlobalUnlock(hglobal);
                }
            }
            ReleaseStgMedium(&mut medium);
            return text;
        }
        None
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
        Ok(())
    }

    fn Drop(
        &self,
        pdataobj: windows_core::Ref<'_, IDataObject>,
        _grfkeystate: MODIFIERKEYS_FLAGS,
        _pt: &POINTL,
        pdweffect: *mut DROPEFFECT,
    ) -> windows::core::Result<()> {
        unsafe {
            if !pdweffect.is_null() {
                *pdweffect = DROPEFFECT_COPY;
            }

            if let Some(dataobj) = pdataobj.as_ref() {
                // 1. First, check for native filesystem files (CF_HDROP)
                if let Some(paths) = self.extract_files(dataobj) {
                    info!("HoldemDropTarget received {} file path(s)", paths.len());
                    if let Some(drag_state) = self.app_handle.try_state::<Arc<DragState>>() {
                        drag_state.successful_drop.store(true, Ordering::Relaxed);
                        drag_state.drag_started.store(false, Ordering::Relaxed);
                    }
                    if let Some(file_list) = self.app_handle.try_state::<FileList>() {
                        crate::file_drop::handle_file_drop_from_paths(
                            paths,
                            file_list.inner().clone(),
                            self.app_handle.clone(),
                        );
                    }
                    return Ok(());
                }

                // 2. Check for HTML format (images/content dragged from web browsers)
                if let Some(html) = self.extract_html(dataobj) {
                    if let Some(src) = parse_img_src(&html) {
                        info!("HoldemDropTarget detected image src in dropped HTML: {}", src);
                        if let Some(drag_state) = self.app_handle.try_state::<Arc<DragState>>() {
                            drag_state.successful_drop.store(true, Ordering::Relaxed);
                            drag_state.drag_started.store(false, Ordering::Relaxed);
                        }
                        handle_web_image_drop(src, self.app_handle.clone());
                        return Ok(());
                    }
                }

                // 3. Check for plain text or direct URL
                if let Some(text) = self.extract_text(dataobj) {
                    let text = text.trim();
                    if !text.is_empty() {
                        if let Some(drag_state) = self.app_handle.try_state::<Arc<DragState>>() {
                            drag_state.successful_drop.store(true, Ordering::Relaxed);
                            drag_state.drag_started.store(false, Ordering::Relaxed);
                        }
                        if text.starts_with("http://") || text.starts_with("https://") {
                            info!("HoldemDropTarget detected dropped URL: {}", text);
                            handle_web_image_drop(text.to_string(), self.app_handle.clone());
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

pub fn register_main_drop_target(app_handle: &AppHandle, hwnd: HWND) {
    unsafe {
        let _ = RevokeDragDrop(hwnd);
        let target = HoldemDropTarget::new(app_handle.clone());
        let target_interface: IDropTarget = target.into();
        let _ = RegisterDragDrop(hwnd, &target_interface);
    }
}

fn parse_img_src(html: &str) -> Option<String> {
    if let Some(img_idx) = html.find("<img") {
        let after_img = &html[img_idx..];
        if let Some(src_idx) = after_img.find("src=") {
            let after_src = &after_img[src_idx + 4..];
            let quote = after_src.chars().next()?;
            if quote == '"' || quote == '\'' {
                let rest = &after_src[1..];
                if let Some(end_quote) = rest.find(quote) {
                    let mut url = rest[..end_quote].replace("&amp;", "&");
                    if url.starts_with("//") {
                        url = format!("https:{}", url);
                    }
                    return Some(url);
                }
            }
        }
    }
    None
}

fn handle_web_image_drop(src: String, app_handle: AppHandle) {
    tauri::async_runtime::spawn(async move {
        if src.starts_with("data:image/") {
            if let Some(comma_pos) = src.find(',') {
                let header = &src[..comma_pos];
                let b64 = &src[comma_pos + 1..];
                let ext = if header.contains("image/png") {
                    "png"
                } else if header.contains("image/jpeg") || header.contains("image/jpg") {
                    "jpg"
                } else if header.contains("image/webp") {
                    "webp"
                } else if header.contains("image/gif") {
                    "gif"
                } else if header.contains("image/svg") {
                    "svg"
                } else {
                    "png"
                };

                if let Ok(path) = crate::commands::file_ops::save_pasted_data_base64(
                    b64.to_string(),
                    ext.to_string(),
                    None,
                ) {
                    if let Some(file_list) = app_handle.try_state::<FileList>() {
                        crate::file_drop::handle_file_drop_from_paths(
                            vec![PathBuf::from(path)],
                            file_list.inner().clone(),
                            app_handle,
                        );
                    }
                }
            }
        } else if src.starts_with("http://") || src.starts_with("https://") {
            match crate::commands::file_ops::download_image_to_shelf(src).await {
                Ok(path) => {
                    if let Some(file_list) = app_handle.try_state::<FileList>() {
                        crate::file_drop::handle_file_drop_from_paths(
                            vec![PathBuf::from(path)],
                            file_list.inner().clone(),
                            app_handle,
                        );
                    }
                }
                Err(e) => {
                    error!("Failed to download dropped web image: {}", e);
                }
            }
        }
    });
}

fn handle_text_drop(text: String, app_handle: AppHandle) {
    tauri::async_runtime::spawn(async move {
        if let Ok(path) = crate::commands::file_ops::save_pasted_text(text, "txt".to_string(), None) {
            if let Some(file_list) = app_handle.try_state::<FileList>() {
                crate::file_drop::handle_file_drop_from_paths(
                    vec![PathBuf::from(path)],
                    file_list.inner().clone(),
                    app_handle,
                );
            }
        }
    });
}
