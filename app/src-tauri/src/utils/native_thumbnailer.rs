use std::path::Path;

#[cfg(target_os = "windows")]
pub mod native {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr;
    use windows::core::{Interface, PCWSTR};
    use windows::Win32::Foundation::{HWND, SIZE};
    use windows::Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED};
    use windows::Win32::UI::Shell::{IShellItemImageFactory, SHCreateItemFromParsingName, SIIGBF};

    fn to_wide_string(s: &str) -> Vec<u16> {
        OsStr::new(s)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect()
    }

    pub fn get_native_thumbnail(path: &str, size: u32) -> Result<(Vec<u8>, usize, usize), String> {
        unsafe {
            let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);

            let wide_path = to_wide_string(path);
            let parsing_name = PCWSTR::from_raw(wide_path.as_ptr());

            let shell_item: IShellItemImageFactory =
                SHCreateItemFromParsingName(parsing_name, None)
                    .map_err(|e| format!("Failed to create shell item: {:?}", e))?;

            let sz = SIZE {
                cx: size as i32,
                cy: size as i32,
            };

            let flags = SIIGBF::SIIGBF_THUMBNAILONLY;

            let hbitmap = shell_item
                .GetImage(sz, flags)
                .map_err(|e| format!("Failed to get thumbnail image: {:?}", e))?;

            let (pixels, width, height, bpp) = {
                use windows::Win32::Graphics::Gdi::{
                    GetObjectW, BITMAP, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
                };

                let mut bmp = std::mem::zeroed::<BITMAP>();
                let dc =
                    windows::Win32::Graphics::Gdi::CreateCompatibleDC(HWND(ptr::null_mut()));
                let old_obj = windows::Win32::Graphics::Gdi::SelectObject(
                    dc,
                    windows::Win32::Graphics::Gdi::HGDIOBJ(hbitmap.0),
                );

                GetObjectW(windows::Win32::Graphics::Gdi::HGDIOBJ(hbitmap.0), &mut bmp)
                    .map_err(|e| format!("Failed to get bitmap info: {:?}", e))?;

                let width = bmp.bmWidth as u32;
                let height = bmp.bmHeight.abs() as u32;
                let bpp_val = bmp.bmBitsPixel as u32;
                let stride = ((width * bpp_val + 31) / 32) * 4;
                let pixel_count = (stride * height) as usize;

                let mut bmi: BITMAPINFO = std::mem::zeroed();
                bmi.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
                bmi.bmiHeader.biWidth = width as i32;
                bmi.bmiHeader.biHeight = -(height as i32);
                bmi.bmiHeader.biPlanes = 1;
                bmi.bmiHeader.biBitCount = bpp_val as u16;
                bmi.bmiHeader.biCompression = BI_RGB.0 as u32;

                let mut pixels: Vec<u8> = vec![0; pixel_count];

                windows::Win32::Graphics::Gdi::GetDIBits(
                    dc,
                    hbitmap,
                    0,
                    height,
                    Some(pixels.as_mut_ptr() as *mut _),
                    &mut bmi,
                    DIB_RGB_COLORS,
                );

                windows::Win32::Graphics::Gdi::SelectObject(dc, old_obj);
                windows::Win32::Graphics::Gdi::DeleteDC(dc);
                windows::Win32::Foundation::DeleteObject(windows::Win32::Foundation::HGDIOBJ(
                    hbitmap.0,
                ));

                (pixels, width as usize, height as usize, bpp_val as usize)
            };

            let rgba_pixels = if bpp == 32 {
                let stride = width * 4;
                let mut rgba = Vec::with_capacity(width * height * 4);
                for y in 0..height {
                    let row_start = y * stride;
                    let row = &pixels[row_start..row_start + stride];
                    for chunk in row.chunks(4) {
                        rgba.push(chunk[2]);
                        rgba.push(chunk[1]);
                        rgba.push(chunk[0]);
                        rgba.push(chunk[3]);
                    }
                }
                rgba
            } else if bpp == 24 {
                let mut rgba = Vec::with_capacity(width * height * 4);
                for y in 0..height {
                    let row_start = y * width * 3;
                    for x in 0..width {
                        let idx = row_start + x * 3;
                        rgba.push(pixels[idx + 2]);
                        rgba.push(pixels[idx + 1]);
                        rgba.push(pixels[idx]);
                        rgba.push(255);
                    }
                }
                rgba
            } else {
                CoUninitialize();
                return Err(format!("Unsupported bit depth: {}", bpp));
            };

            CoUninitialize();

            Ok((rgba_pixels, width, height))
        }
    }
}

#[cfg(target_os = "macos")]
pub mod native {
    use std::process::Command;
    use std::path::Path;
    use image::{DynamicImage, GenericImageView, ImageBuffer, Rgba};

    /// Use macOS QuickLook (`qlmanage -t`) to generate Finder-style thumbnails.
    /// This works for images, videos, PDFs, documents, and any file type that
    /// has a QuickLook generator installed (e.g., VLC for certain video formats).
    pub fn get_native_thumbnail(path: &str, size: u32) -> Result<(Vec<u8>, usize, usize), String> {
        let tmp_dir = std::env::temp_dir().join("holdem_thumbnails");
        std::fs::create_dir_all(&tmp_dir)
            .map_err(|e| format!("Failed to create temp dir: {}", e))?;

        // Run qlmanage to generate a thumbnail
        let output = Command::new("qlmanage")
            .arg("-t")
            .arg(path)
            .arg("-s")
            .arg(size.to_string())
            .arg("-o")
            .arg(tmp_dir.to_str().unwrap())
            .output()
            .map_err(|e| format!("Failed to run qlmanage: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("qlmanage failed: {}", stderr));
        }

        // qlmanage outputs a file named "<filename>.png" in the output directory
        let file_name = Path::new(path)
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| "Invalid file name".to_string())?;

        let thumb_path = tmp_dir.join(format!("{}.png", file_name));

        if !thumb_path.exists() {
            // Clean up
            let _ = std::fs::remove_dir_all(&tmp_dir);
            return Err("qlmanage did not produce a thumbnail".to_string());
        }

        // Read the generated thumbnail
        let img: DynamicImage = image::open(&thumb_path)
            .map_err(|e| format!("Failed to open generated thumbnail: {}", e))?;

        // Clean up the temp file
        let _ = std::fs::remove_file(&thumb_path);

        let (width, height) = img.dimensions();
        let rgba_img: ImageBuffer<Rgba<u8>, Vec<u8>> = img.to_rgba8();
        let pixels = rgba_img.into_raw();

        Ok((pixels, width as usize, height as usize))
    }
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub mod native {
    pub fn get_native_thumbnail(_path: &str, _size: u32) -> Result<(Vec<u8>, usize, usize), String> {
        Err("Native thumbnails not supported on this platform".to_string())
    }
}

pub fn get_native_thumbnail_rgba(
    path: &str,
    size: u32,
) -> Result<(Vec<u8>, usize, usize), String> {
    let path_obj = Path::new(path);
    if !path_obj.exists() {
        return Err(format!("File does not exist: {}", path));
    }

    native::get_native_thumbnail(path, size)
}
