#![cfg(target_os = "windows")]

use serde::Serialize;
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use tauri::{AppHandle, Emitter};
use windows::core::{implement, Result as WindowsResult};
use windows::Win32::Foundation::POINTL;
use windows::Win32::System::Com::{
    IDataObject, DVASPECT_CONTENT, FORMATETC, TYMED_HGLOBAL,
};
use windows::Win32::System::Memory::{GlobalLock, GlobalSize, GlobalUnlock};
use windows::Win32::System::Ole::{
    IDropTarget, IDropTarget_Impl, DROPEFFECT, DROPEFFECT_COPY,
};
use windows::Win32::System::SystemServices::MODIFIERKEYS_FLAGS;
use windows::Win32::UI::Shell::{DragQueryFileW, HDROP};

#[derive(Clone, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum DropPayload {
    Files(Vec<String>),
    Text(String),
    Html(String),
}

#[implement(IDropTarget)]
pub struct CustomDropTarget {
    app_handle: AppHandle,
}

impl CustomDropTarget {
    pub fn new(app_handle: AppHandle) -> Self {
        Self { app_handle }
    }

    unsafe fn extract_files(&self, pdataobj: &IDataObject) -> Option<Vec<String>> {
        let mut fmt = FORMATETC {
            cfFormat: 15, // CF_HDROP
            ptd: std::ptr::null_mut(),
            dwAspect: DVASPECT_CONTENT.0,
            lindex: -1,
            tymed: TYMED_HGLOBAL.0 as u32,
        };

        if let Ok(stg) = pdataobj.GetData(&mut fmt) {
            let hglobal = stg.u.hGlobal;
            if !hglobal.is_invalid() {
                let ptr = GlobalLock(hglobal);
                if !ptr.is_null() {
                    let hdrop = HDROP(ptr as _);
                    let count = DragQueryFileW(hdrop, 0xFFFFFFFF, None);
                    let mut files = Vec::new();

                    for i in 0..count {
                        let len = DragQueryFileW(hdrop, i, None);
                        if len > 0 {
                            let mut buffer: Vec<u16> = vec![0; (len + 1) as usize];
                            DragQueryFileW(hdrop, i, Some(&mut buffer));
                            
                            if let Some(pos) = buffer.iter().position(|&c| c == 0) {
                                buffer.truncate(pos);
                            }
                            
                            let path = OsString::from_wide(&buffer).to_string_lossy().into_owned();
                            files.push(path);
                        }
                    }

                    let _ = GlobalUnlock(hglobal);
                    return Some(files);
                }
            }
        }
        None
    }

    unsafe fn extract_text(&self, pdataobj: &IDataObject) -> Option<String> {
        // Try CF_UNICODETEXT first
        let mut fmt = FORMATETC {
            cfFormat: 13, // CF_UNICODETEXT
            ptd: std::ptr::null_mut(),
            dwAspect: DVASPECT_CONTENT.0,
            lindex: -1,
            tymed: TYMED_HGLOBAL.0 as u32,
        };

        match pdataobj.GetData(&mut fmt) {
            Ok(stg) => {
                let hglobal = stg.u.hGlobal;
                if !hglobal.is_invalid() {
                    let ptr = GlobalLock(hglobal) as *const u16;
                    if !ptr.is_null() {
                        let size = GlobalSize(hglobal);
                        let count = size / 2;
                        let slice = std::slice::from_raw_parts(ptr, count as usize);
                        let end = slice.iter().position(|&c| c == 0).unwrap_or(slice.len());
                        let text = String::from_utf16_lossy(&slice[..end]);
                        let _ = GlobalUnlock(hglobal);
                        return Some(text);
                    } else {
                        println!("GlobalLock failed for CF_UNICODETEXT");
                    }
                }
            }
            Err(e) => println!("GetData failed for CF_UNICODETEXT: {}", e),
        }

        // Try CF_TEXT (1)
        fmt.cfFormat = 1; // CF_TEXT
        match pdataobj.GetData(&mut fmt) {
            Ok(stg) => {
                let hglobal = stg.u.hGlobal;
                if !hglobal.is_invalid() {
                    let ptr = GlobalLock(hglobal) as *const u8;
                    if !ptr.is_null() {
                        let size = GlobalSize(hglobal);
                        let slice = std::slice::from_raw_parts(ptr, size as usize);
                        let end = slice.iter().position(|&c| c == 0).unwrap_or(slice.len());
                        let text = String::from_utf8_lossy(&slice[..end]).into_owned();
                        let _ = GlobalUnlock(hglobal);
                        return Some(text);
                    } else {
                        println!("GlobalLock failed for CF_TEXT");
                    }
                }
            }
            Err(e) => println!("GetData failed for CF_TEXT: {}", e),
        }
        
        None
    }

    unsafe fn extract_html(&self, pdataobj: &IDataObject) -> Option<String> {
        let html_fmt_id = windows::Win32::System::DataExchange::RegisterClipboardFormatW(windows::core::w!("HTML Format"));
        if html_fmt_id == 0 {
            return None;
        }

        let mut fmt = FORMATETC {
            cfFormat: html_fmt_id as u16,
            ptd: std::ptr::null_mut(),
            dwAspect: DVASPECT_CONTENT.0,
            lindex: -1,
            tymed: TYMED_HGLOBAL.0 as u32,
        };

        if let Ok(stg) = pdataobj.GetData(&mut fmt) {
            let hglobal = stg.u.hGlobal;
            if !hglobal.is_invalid() {
                let ptr = GlobalLock(hglobal) as *const u8;
                if !ptr.is_null() {
                    let size = GlobalSize(hglobal);
                    let slice = std::slice::from_raw_parts(ptr, size as usize);
                    let end = slice.iter().position(|&c| c == 0).unwrap_or(slice.len());
                    let text = String::from_utf8_lossy(&slice[..end]).into_owned();
                    let _ = GlobalUnlock(hglobal);
                    return Some(text);
                }
            }
        }
        None
    }
}

impl IDropTarget_Impl for CustomDropTarget_Impl {
    fn DragEnter(
        &self,
        pdataobj: windows::core::Ref<'_, IDataObject>,
        _grfkeystate: MODIFIERKEYS_FLAGS,
        _pt: &POINTL,
        pdweffect: *mut DROPEFFECT,
    ) -> WindowsResult<()> {
        unsafe {
            if let Some(obj) = pdataobj.as_ref() {
                if let Ok(mut ef) = obj.EnumFormatEtc(windows::Win32::System::Com::DATADIR_GET.0 as u32) {
                    let mut fmt = [windows::Win32::System::Com::FORMATETC::default(); 1];
                    let mut fetched = 0;
                    while ef.Next(&mut fmt, Some(&mut fetched)).is_ok() && fetched > 0 {
                        println!("DRAG FORMAT AVAILABLE: {}", fmt[0].cfFormat);
                    }
                }
            }

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
    ) -> WindowsResult<()> {
        unsafe {
            if !pdweffect.is_null() {
                *pdweffect = DROPEFFECT_COPY;
            }
        }
        Ok(())
    }

    fn DragLeave(&self) -> WindowsResult<()> {
        Ok(())
    }

    fn Drop(
        &self,
        pdataobj: windows::core::Ref<'_, IDataObject>,
        _grfkeystate: MODIFIERKEYS_FLAGS,
        _pt: &POINTL,
        pdweffect: *mut DROPEFFECT,
    ) -> WindowsResult<()> {
        unsafe {
            if !pdweffect.is_null() {
                *pdweffect = DROPEFFECT_COPY;
            }

            if let Some(dataobj) = pdataobj.as_ref() {
                let mut debug_log = String::new();
                debug_log.push_str("Drop called! Attempting to extract files...\n");
                
                if let Some(files) = self.extract_files(dataobj) {
                    debug_log.push_str(&format!("extract_files succeeded: {:?}\n", files));
                    if !files.is_empty() {
                        let _ = self.app_handle.emit("native_drop", DropPayload::Files(files));
                        let _ = std::fs::write("C:\\Users\\Zubin\\Desktop\\holdem_drop_debug.txt", debug_log);
                        return Ok(());
                    }
                }
                
                debug_log.push_str("extract_files failed, attempting extract_html...\n");
                if let Some(html) = self.extract_html(dataobj) {
                    debug_log.push_str(&format!("extract_html succeeded, length: {}\nHTML HEAD: {}\n", html.len(), &html[..html.len().min(300)]));
                    if !html.is_empty() {
                        let _ = self.app_handle.emit("native_drop", DropPayload::Html(html));
                        let _ = std::fs::write("C:\\Users\\Zubin\\Desktop\\holdem_drop_debug.txt", debug_log);
                        return Ok(());
                    }
                }

                debug_log.push_str("extract_html failed, attempting extract_text...\n");
                if let Some(text) = self.extract_text(dataobj) {
                    debug_log.push_str(&format!("extract_text succeeded, length: {}\nTEXT HEAD: {}\n", text.len(), &text[..text.len().min(100)]));
                    if !text.is_empty() {
                        let _ = self.app_handle.emit("native_drop", DropPayload::Text(text));
                        let _ = std::fs::write("C:\\Users\\Zubin\\Desktop\\holdem_drop_debug.txt", debug_log);
                        return Ok(());
                    }
                }
                debug_log.push_str("All extractions failed.\n");
                let _ = std::fs::write("C:\\Users\\Zubin\\Desktop\\holdem_drop_debug.txt", debug_log);
            }
        }
        Ok(())
    }
}
