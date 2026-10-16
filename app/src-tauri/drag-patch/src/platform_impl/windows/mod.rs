// Copyright 2023-2023 CrabNebula Ltd.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use raw_window_handle::{HasWindowHandle, RawWindowHandle};

use crate::{CursorPosition, DragItem, DragMode, DragResult, Image, Options};

use std::{
    ffi::c_void,
    iter::once,
    os::windows::ffi::OsStrExt,
    path::{Path, PathBuf},
};
use windows::{
    core::{implement, Interface, BOOL, GUID, HRESULT, PCWSTR},
    Win32::{
        Foundation::{
            GlobalFree, COLORREF, DRAGDROP_S_CANCEL, DRAGDROP_S_DROP,
            DRAGDROP_S_USEDEFAULTCURSORS, HGLOBAL, POINT, SIZE, S_OK,
        },
        Globalization::{WideCharToMultiByte, CP_ACP},
        Graphics::Gdi::{DeleteObject, GetObjectW, BITMAP, HBITMAP},
        System::Com::{
            CoCreateInstance, IDataObject, CLSCTX_ALL, DVASPECT_CONTENT, FORMATETC, STGMEDIUM,
            STGMEDIUM_0, TYMED_HGLOBAL,
        },
        System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GHND},
        System::Ole::{
            DoDragDrop, IDropSource, IDropSource_Impl, OleInitialize, CF_HDROP, CF_TEXT,
            CF_UNICODETEXT, CLIPBOARD_FORMAT, DROPEFFECT, DROPEFFECT_COPY, DROPEFFECT_MOVE,
            DROPEFFECT_NONE,
        },
        System::SystemServices::{MK_LBUTTON, MODIFIERKEYS_FLAGS},
        UI::{
            Shell::{
                BHID_DataObject, CLSID_DragDropHelper, Common::ITEMIDLIST, IDragSourceHelper,
                ILCreateFromPathW, ILFree, SHCreateDataObject, SHCreateShellItemArrayFromIDLists,
                DROPFILES, SHDRAGIMAGE,
            },
            WindowsAndMessaging::GetCursorPos,
        },
    },
};

mod image;

thread_local! {
    /// OLE must be initialized on every thread that calls `DoDragDrop` (and the
    /// thread must be STA). Initialize once per thread and keep it for the
    /// lifetime of the thread.
    static OLE_INIT: windows::core::Result<()> = unsafe { OleInitialize(None) };
}

fn ensure_ole() -> crate::Result<()> {
    OLE_INIT.with(|r| r.clone().map_err(Into::into))
}

#[implement(IDropSource)]
struct DropSource;

#[allow(non_snake_case)]
impl IDropSource_Impl for DropSource_Impl {
    fn QueryContinueDrag(&self, fescapepressed: BOOL, grfkeystate: MODIFIERKEYS_FLAGS) -> HRESULT {
        if fescapepressed.as_bool() {
            DRAGDROP_S_CANCEL
        } else if (grfkeystate & MK_LBUTTON) == MODIFIERKEYS_FLAGS(0) {
            DRAGDROP_S_DROP
        } else {
            S_OK
        }
    }

    fn GiveFeedback(&self, _dweffect: DROPEFFECT) -> HRESULT {
        DRAGDROP_S_USEDEFAULTCURSORS
    }
}

pub fn start_drag<W: HasWindowHandle, F: Fn(DragResult, CursorPosition) + Send + 'static>(
    handle: &W,
    item: DragItem,
    image: Image,
    on_drop_callback: F,
    options: Options,
) -> crate::Result<()> {
    let Ok(RawWindowHandle::Win32(_)) = handle.window_handle().map(|h| h.as_raw()) else {
        return Err(crate::Error::UnsupportedWindowHandle);
    };

    ensure_ole()?;

    let (data_object, allowed_effects, is_files) = match item {
        DragItem::Files(files) => {
            if files.is_empty() {
                return Err(crate::Error::FailedToStartDrag);
            }
            let mut paths = Vec::with_capacity(files.len());
            for f in files {
                // Fails (instead of panicking later) when the file no longer exists.
                paths.push(dunce::canonicalize(f)?);
            }
            let effects = match options.mode {
                DragMode::Copy => DROPEFFECT_COPY,
                DragMode::Move => DROPEFFECT_MOVE,
                DragMode::CopyOrMove => DROPEFFECT_COPY | DROPEFFECT_MOVE,
            };
            (get_file_data_object(&paths)?, effects, true)
        }
        DragItem::Data { provider, types } => {
            let text = collect_text(&provider, &types).ok_or(crate::Error::FailedToStartDrag)?;
            (get_text_data_object(&text)?, DROPEFFECT_COPY, false)
        }
    };

    let drop_source: IDropSource = DropSource.into();
    set_drag_image(image, &data_object);

    let mut out_effect = DROPEFFECT_NONE;
    let drop_result =
        unsafe { DoDragDrop(&data_object, &drop_source, allowed_effects, &mut out_effect) };
    // Failure HRESULTs (E_OUTOFMEMORY, E_UNSPEC, ...) become an error;
    // DRAGDROP_S_DROP / DRAGDROP_S_CANCEL are success codes.
    drop_result.ok()?;

    let mut pt = POINT { x: 0, y: 0 };
    // Not fatal: the drag already finished, so still report the outcome.
    let _ = unsafe { GetCursorPos(&mut pt) };

    // For file drags an "optimized move" (e.g. Explorer, same volume) legitimately
    // reports DROPEFFECT_NONE even though the drop happened, so only the
    // HRESULT is trusted there. For text, a DROPEFFECT_NONE means the target
    // refused the data.
    let dropped = drop_result == DRAGDROP_S_DROP && (is_files || out_effect != DROPEFFECT_NONE);
    let result = if dropped {
        DragResult::Dropped
    } else {
        DragResult::Cancel
    };
    on_drop_callback(result, CursorPosition { x: pt.x, y: pt.y });

    Ok(())
}

/// Pulls a UTF-8 text payload out of the data provider.
fn collect_text(provider: &crate::DataProvider, types: &[String]) -> Option<String> {
    let is_text = |t: &str| {
        let t = t.to_ascii_lowercase();
        t.starts_with("text/plain") || t == "text" || t == "string" || t == "utf8_string"
    };
    types
        .iter()
        .filter(|t| is_text(t))
        .find_map(|t| provider(t))
        .or_else(|| provider("text/plain"))
        .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
}

/// Attaches the drag image to the data object. Failures are non-fatal (the
/// drag simply has no custom image) and never leak the HBITMAP.
fn set_drag_image(image: Image, data_object: &IDataObject) {
    let Some(drag_image) = get_drag_image(image) else {
        return;
    };
    let attached = unsafe {
        create_instance::<IDragSourceHelper>(&CLSID_DragDropHelper)
            .and_then(|helper| helper.InitializeFromBitmap(&drag_image, data_object))
    };
    if attached.is_err() {
        // On success the drag-image manager owns the bitmap; otherwise it is ours.
        unsafe {
            let _ = DeleteObject(drag_image.hbmpDragImage.into());
        }
    }
}

fn get_drag_image(image: Image) -> Option<SHDRAGIMAGE> {
    let hbitmap: HBITMAP = match image {
        Image::Raw(bytes) => image::read_bytes_to_hbitmap(&bytes).ok(),
        Image::File(path) => image::read_path_to_hbitmap(&path).ok(),
    }?;
    if hbitmap.is_invalid() {
        return None;
    }

    let mut bitmap = BITMAP::default();
    let (width, height) = if unsafe {
        GetObjectW(
            hbitmap.into(),
            std::mem::size_of::<BITMAP>() as i32,
            Some(&mut bitmap as *mut BITMAP as *mut c_void),
        )
    } == 0
    {
        (128, 128)
    } else {
        (bitmap.bmWidth, bitmap.bmHeight)
    };

    Some(SHDRAGIMAGE {
        sizeDragImage: SIZE {
            cx: width,
            cy: height,
        },
        ptOffset: POINT { x: 0, y: 0 },
        hbmpDragImage: hbitmap,
        crColorKey: COLORREF(0x00000000),
    })
}

pub fn create_instance<T: Interface>(clsid: &GUID) -> windows::core::Result<T> {
    unsafe { CoCreateInstance(clsid, None, CLSCTX_ALL) }
}

/// Allocates a zero-initialized movable HGLOBAL of `size` bytes and lets
/// `fill` write into it. The handle is freed if anything goes wrong.
fn alloc_hglobal(size: usize, fill: impl FnOnce(*mut u8)) -> windows::core::Result<HGLOBAL> {
    unsafe {
        let handle = GlobalAlloc(GHND, size)?;
        let ptr = GlobalLock(handle) as *mut u8;
        if ptr.is_null() {
            let err = windows::core::Error::from_thread();
            let _ = GlobalFree(Some(handle));
            return Err(err);
        }
        fill(ptr);
        // GlobalUnlock returns FALSE with NO_ERROR once the lock count hits
        // zero, which windows-rs reports as an Err; that is the expected case.
        let _ = GlobalUnlock(handle);
        Ok(handle)
    }
}

/// Builds a CF_HDROP payload (DROPFILES header + double-NUL-terminated list
/// of wide paths) directly, so it is not limited to MAX_PATH.
fn build_hdrop(paths: &[PathBuf]) -> windows::core::Result<HGLOBAL> {
    let mut list: Vec<u16> = Vec::new();
    for path in paths {
        list.extend(path.as_os_str().encode_wide().chain(once(0)));
    }
    list.push(0);

    let header = std::mem::size_of::<DROPFILES>();
    let size = header + list.len() * std::mem::size_of::<u16>();
    alloc_hglobal(size, |ptr| unsafe {
        // Memory is zeroed (GHND), so pt and fNC are already 0.
        let dropfiles = ptr as *mut DROPFILES;
        (*dropfiles).pFiles = header as u32;
        (*dropfiles).fWide = BOOL(1);
        std::ptr::copy_nonoverlapping(
            list.as_ptr() as *const u8,
            ptr.add(header),
            list.len() * std::mem::size_of::<u16>(),
        );
    })
}

fn build_unicode_text(text: &str) -> windows::core::Result<HGLOBAL> {
    let wide: Vec<u16> = text.encode_utf16().chain(once(0)).collect();
    let bytes = wide.len() * std::mem::size_of::<u16>();
    alloc_hglobal(bytes, |ptr| unsafe {
        std::ptr::copy_nonoverlapping(wide.as_ptr() as *const u8, ptr, bytes);
    })
}

fn build_ansi_text(text: &str) -> windows::core::Result<HGLOBAL> {
    let wide: Vec<u16> = text.encode_utf16().chain(once(0)).collect();
    // WideCharToMultiByte takes i32 lengths; skip CF_TEXT for absurd sizes.
    if wide.len() > (i32::MAX / 4) as usize {
        return Err(windows::core::Error::from(
            windows::Win32::Foundation::E_OUTOFMEMORY,
        ));
    }
    unsafe {
        let needed =
            WideCharToMultiByte(CP_ACP, 0, &wide, None, windows::core::PCSTR::null(), None);
        if needed <= 0 {
            return Err(windows::core::Error::from_thread());
        }
        let mut ansi = vec![0u8; needed as usize];
        let written = WideCharToMultiByte(
            CP_ACP,
            0,
            &wide,
            Some(&mut ansi),
            windows::core::PCSTR::null(),
            None,
        );
        if written <= 0 {
            return Err(windows::core::Error::from_thread());
        }
        ansi.truncate(written as usize);
        if ansi.last() != Some(&0) {
            ansi.push(0);
        }
        alloc_hglobal(ansi.len(), |ptr| {
            std::ptr::copy_nonoverlapping(ansi.as_ptr(), ptr, ansi.len());
        })
    }
}

/// Hands `hglobal` to the data object under `format`. On success the data
/// object owns it (fRelease = TRUE); on failure it is freed here.
fn set_hglobal(
    data_object: &IDataObject,
    format: CLIPBOARD_FORMAT,
    hglobal: HGLOBAL,
) -> windows::core::Result<()> {
    let format_etc = FORMATETC {
        cfFormat: format.0,
        ptd: std::ptr::null_mut(),
        dwAspect: DVASPECT_CONTENT.0,
        lindex: -1,
        tymed: TYMED_HGLOBAL.0 as u32,
    };
    let medium = STGMEDIUM {
        tymed: TYMED_HGLOBAL.0 as u32,
        u: STGMEDIUM_0 { hGlobal: hglobal },
        pUnkForRelease: std::mem::ManuallyDrop::new(None),
    };
    let result = unsafe { data_object.SetData(&format_etc, &medium, true) };
    if result.is_err() {
        unsafe {
            let _ = GlobalFree(Some(hglobal));
        }
    }
    result
}

/// An empty shell data object: accepts arbitrary formats via SetData,
/// implements EnumFormatEtc, and supports the drag-image helper formats.
fn new_empty_data_object() -> windows::core::Result<IDataObject> {
    unsafe { SHCreateDataObject(None, None, None) }
}

fn get_text_data_object(text: &str) -> crate::Result<IDataObject> {
    let data_object = new_empty_data_object()?;
    set_hglobal(&data_object, CF_UNICODETEXT, build_unicode_text(text)?)?;
    // CF_TEXT is a best-effort extra for legacy targets.
    if let Ok(ansi) = build_ansi_text(text) {
        let _ = set_hglobal(&data_object, CF_TEXT, ansi);
    }
    Ok(data_object)
}

fn get_file_data_object(paths: &[PathBuf]) -> crate::Result<IDataObject> {
    // Preferred: a real shell data object (CFSTR_SHELLIDLIST, CF_HDROP, ...),
    // which Explorer and other shell targets understand best.
    if let Some(data_object) = get_shell_data_object(paths) {
        return Ok(data_object);
    }
    // Fallback (long paths, items the shell can't resolve, items with
    // different parents, ...): plain CF_HDROP built by hand.
    let data_object = new_empty_data_object()?;
    set_hglobal(&data_object, CF_HDROP, build_hdrop(paths)?)?;
    Ok(data_object)
}

/// Owns a PIDL returned by `ILCreateFromPathW` and frees it on drop.
struct Pidl(*mut ITEMIDLIST);

impl Drop for Pidl {
    fn drop(&mut self) {
        unsafe { ILFree(Some(self.0.cast_const())) };
    }
}

fn get_shell_data_object(paths: &[PathBuf]) -> Option<IDataObject> {
    let pidls = paths
        .iter()
        .map(|p| get_file_item_id(p))
        .collect::<Option<Vec<Pidl>>>()?;
    let list: Vec<*const ITEMIDLIST> = pidls.iter().map(|p| p.0.cast_const()).collect();
    unsafe {
        // The shell item array copies the PIDLs; `pidls` is freed on return.
        let shell_item_array = SHCreateShellItemArrayFromIDLists(&list).ok()?;
        shell_item_array
            .BindToHandler::<_, IDataObject>(None, &BHID_DataObject)
            .ok()
    }
}

fn get_file_item_id(path: &Path) -> Option<Pidl> {
    let wide_path: Vec<u16> = path.as_os_str().encode_wide().chain(once(0)).collect();
    let pidl = unsafe { ILCreateFromPathW(PCWSTR::from_raw(wide_path.as_ptr())) };
    if pidl.is_null() {
        None
    } else {
        Some(Pidl(pidl))
    }
}
