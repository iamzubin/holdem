//! Windows thumbnail extraction using the Shell API.
//!
//! In-house replacement for the `thumb-rs` git dependency. Uses the same API
//! Explorer.exe calls (`IShellItemImageFactory::GetImage`), so output is
//! identical: real thumbnails when a provider exists (images, video, PDF,
//! Office docs), file-type icons as fallback.
//!
//! Holdem is Windows-only, so only the Windows backend is ported. Thumbnail
//! size is fixed at 256x256 — the only size any caller ever used
//! (`ThumbnailScale::default()`).

use base64::{engine::general_purpose, Engine as _};
use image::{ImageBuffer, Rgba};
use std::path::Path;
use thiserror::Error;
use windows::core::HSTRING;
use windows::Win32::Foundation::SIZE;
use windows::Win32::Graphics::Gdi::{
    CreateCompatibleDC, DeleteDC, DeleteObject, GetDIBits, GetObjectW, SelectObject, BITMAP,
    BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, HBITMAP, HDC, HGDIOBJ,
};
use windows::Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED};
use windows::Win32::UI::Shell::{IShellItemImageFactory, SHCreateItemFromParsingName, SIIGBF};

/// Maximum thumbnail dimension in pixels (square).
const THUMB_PX: i32 = 256;

/// Errors from thumbnail generation. Mapped to `String` at the Tauri command
/// boundary (`get_thumbnail_base64`), which must return `Result<_, String>`.
#[derive(Error, Debug)]
pub enum ThumbnailError {
    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Thumbnail generation failed: {0}")]
    GenerationFailed(String),

    #[error("Platform error: {0}")]
    PlatformError(String),
}

/// Generate a base64-encoded PNG thumbnail for any file type the OS can
/// preview. Returns the file-type icon (as Explorer shows) when no thumbnail
/// provider is registered.
pub fn get_thumbnail_base64(file_path: &str) -> Result<String, String> {
    let path: &Path = Path::new(file_path);
    if !path.exists() {
        return Err(ThumbnailError::FileNotFound(file_path.to_string()).to_string());
    }
    let (rgba, width, height): (Vec<u8>, u32, u32) =
        generate_thumbnail(path).map_err(|e: ThumbnailError| e.to_string())?;
    let img: ImageBuffer<Rgba<u8>, Vec<u8>> =
        ImageBuffer::<Rgba<u8>, _>::from_raw(width, height, rgba)
            .ok_or_else(|| "Failed to create image buffer".to_string())?;
    let mut png_data: Vec<u8> = Vec::new();
    img.write_to(
        &mut std::io::Cursor::new(&mut png_data),
        image::ImageFormat::Png,
    )
    .map_err(|e| e.to_string())?;
    Ok(general_purpose::STANDARD.encode(&png_data))
}

/// Generate raw RGBA8 pixels via `IShellItemImageFactory::GetImage`.
///
/// Uses default flags (`SIIGBF(0)` = `SIIGBF_RESIZETOFIT`), matching Explorer:
/// real thumbnails when available, icons as fallback. Do NOT switch to
/// `SIIGBF_THUMBNAILONLY` — most documents would start erroring.
fn generate_thumbnail(file_path: &Path) -> Result<(Vec<u8>, u32, u32), ThumbnailError> {
    let path_str: &str = file_path.to_str().ok_or_else(|| {
        ThumbnailError::PlatformError("Invalid UTF-8 in file path".to_string())
    })?;
    let wide_path: HSTRING = HSTRING::from(path_str);

    // Required for COM. Per-call init is correct: Tauri runs commands on a
    // thread pool, so this must not be hoisted to `setup`.
    let hr: windows::core::HRESULT = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
    if hr.is_err() {
        return Err(ThumbnailError::PlatformError(format!(
            "CoInitializeEx failed: {hr:?}"
        )));
    }

    let shell_item: IShellItemImageFactory =
        match unsafe { SHCreateItemFromParsingName(&wide_path, None) } {
            Ok(item) => item,
            Err(e) => {
                unsafe { CoUninitialize() };
                // 0x80070002 = ERROR_FILE_NOT_FOUND, 0x80070003 = ERROR_PATH_NOT_FOUND
                let code: u32 = e.code().0 as u32;
                if code == 0x80070002 || code == 0x80070003 {
                    return Err(ThumbnailError::FileNotFound(
                        file_path.display().to_string(),
                    ));
                }
                return Err(ThumbnailError::PlatformError(format!(
                    "SHCreateItemFromParsingName failed: {e}"
                )));
            }
        };

    let hbitmap: HBITMAP = match get_hbitmap(&shell_item) {
        Ok(h) => h,
        Err(e) => {
            unsafe { CoUninitialize() };
            return Err(e);
        }
    };

    let result: Result<(Vec<u8>, u32, u32), ThumbnailError> = hbitmap_to_rgba(hbitmap);

    unsafe {
        let _ = DeleteObject(hbitmap.into());
        CoUninitialize();
    }

    result
}

fn get_hbitmap(shell_item: &IShellItemImageFactory) -> Result<HBITMAP, ThumbnailError> {
    let dimensions: SIZE = SIZE {
        cx: THUMB_PX,
        cy: THUMB_PX,
    };
    unsafe { shell_item.GetImage(dimensions, SIIGBF(0)) }
        .map_err(|e| ThumbnailError::GenerationFailed(format!("GetImage failed: {e}")))
}

/// Extract RGBA8 from an `HBITMAP` via GDI.
fn hbitmap_to_rgba(hbitmap: HBITMAP) -> Result<(Vec<u8>, u32, u32), ThumbnailError> {
    let mut bitmap: BITMAP = BITMAP::default();
    unsafe {
        GetObjectW(
            HGDIOBJ(hbitmap.0),
            std::mem::size_of::<BITMAP>() as i32,
            Some(&mut bitmap as *mut _ as *mut _),
        );
    }

    let width: u32 = bitmap.bmWidth as u32;
    let height: u32 = bitmap.bmHeight as u32;

    let dc: HDC = unsafe { CreateCompatibleDC(None) };
    if dc == HDC::default() {
        return Err(ThumbnailError::PlatformError(
            "CreateCompatibleDC failed".to_string(),
        ));
    }

    let old_obj = unsafe { SelectObject(dc, hbitmap.into()) };

    let mut bmi: BITMAPINFO = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: width as i32,
            biHeight: height as i32, // positive = bottom-up
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..Default::default()
        },
        ..Default::default()
    };

    let pixel_count: usize = (width as usize) * (height as usize);
    let mut buffer: Vec<u8> = vec![0u8; pixel_count * 4];

    let lines: i32 = unsafe {
        GetDIBits(
            dc,
            hbitmap,
            0,
            height,
            Some(buffer.as_mut_ptr() as *mut _),
            &mut bmi,
            DIB_RGB_COLORS,
        )
    };

    unsafe {
        SelectObject(dc, old_obj);
        let _ = DeleteDC(dc);
    }

    if lines == 0 {
        return Err(ThumbnailError::GenerationFailed(
            "GetDIBits returned 0 lines".to_string(),
        ));
    }

    swap_bgra_to_rgba(&mut buffer);
    let flipped: Vec<u8> = flip_rows_bottom_up(&buffer, width, height);

    Ok((flipped, width, height))
}

/// `GetDIBits` returns BGRA; PNG/`image` wants RGBA. Pure for testability.
fn swap_bgra_to_rgba(buffer: &mut [u8]) {
    for pixel in buffer.chunks_exact_mut(4) {
        pixel.swap(0, 2);
    }
}

/// `GetDIBits` returns rows bottom-up; callers want top-down. Pure for
/// testability.
fn flip_rows_bottom_up(buffer: &[u8], width: u32, height: u32) -> Vec<u8> {
    let row_bytes: usize = (width as usize) * 4;
    let mut flipped: Vec<u8> = vec![0u8; buffer.len()];
    for row in 0..height as usize {
        let src: usize = row * row_bytes;
        let dst: usize = (height as usize - 1 - row) * row_bytes;
        flipped[dst..dst + row_bytes].copy_from_slice(&buffer[src..src + row_bytes]);
    }
    flipped
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swap_bgra_to_rgba() {
        let mut buf: Vec<u8> = vec![10, 20, 30, 255, 1, 2, 3, 128];
        swap_bgra_to_rgba(&mut buf);
        assert_eq!(buf, vec![30, 20, 10, 255, 3, 2, 1, 128]);
    }

    #[test]
    fn test_flip_rows_bottom_up() {
        // 2x2 image, one row = 8 bytes. Row 0 is bottom, row 1 is top.
        let bottom: Vec<u8> = vec![1, 1, 1, 255, 2, 2, 2, 255];
        let top: Vec<u8> = vec![3, 3, 3, 255, 4, 4, 4, 255];
        let buffer: Vec<u8> = [bottom.clone(), top.clone()].concat();
        let flipped: Vec<u8> = flip_rows_bottom_up(&buffer, 2, 2);
        assert_eq!(flipped, [top, bottom].concat());
    }

    #[test]
    fn test_thumbnail_error_display() {
        let e: ThumbnailError =
            ThumbnailError::FileNotFound("C:\\missing.txt".to_string());
        assert_eq!(e.to_string(), "File not found: C:\\missing.txt");
    }
}
