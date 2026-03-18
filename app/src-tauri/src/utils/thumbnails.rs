use base64::{engine::general_purpose, Engine as _};
use image::codecs::png::PngEncoder;
use image::ColorType;
use image::ImageEncoder;
use std::path::Path;

use crate::utils::native_thumbnailer;

fn crop_transparent_bounds(p: &[u8], w: usize, h: usize) -> (Vec<u8>, u32, u32) {
    let stride = w * 4;
    let (mut top, mut bottom, mut left, mut right) = (h, 0, w, 0);
    for y in 0..h {
        for x in 0..w {
            let i = y * stride + x * 4;
            if p[i + 3] > 10 {
                if y < top {
                    top = y
                }
                if y > bottom {
                    bottom = y
                }
                if x < left {
                    left = x
                }
                if x > right {
                    right = x
                }
            }
        }
    }
    if top > bottom || left > right {
        return (p.to_vec(), w as u32, h as u32);
    }
    let (nw, nh) = (right - left + 1, bottom - top + 1);
    let mut out = vec![0u8; nw * nh * 4];
    for y in 0..nh {
        let src = ((top + y) * w + left) * 4;
        let dst = y * nw * 4;
        out[dst..dst + nw * 4].copy_from_slice(&p[src..src + nw * 4]);
    }
    (out, nw as u32, nh as u32)
}

fn encode_png(pixels: &[u8], width: u32, height: u32) -> Result<String, String> {
    let (cropped, w, h) = crop_transparent_bounds(pixels, width as usize, height as usize);

    let mut png_data = Vec::new();
    {
        let encoder = PngEncoder::new(&mut png_data);
        encoder
            .write_image(&cropped, w, h, ColorType::Rgba8.into())
            .map_err(|e| format!("Failed to encode PNG: {}", e))?;
    }

    Ok(general_purpose::STANDARD.encode(&png_data))
}

/// Try to get an OS-level thumbnail for any file type.
/// On macOS this uses QuickLook (qlmanage), on Windows it uses IShellItemImageFactory.
/// These produce the same thumbnails shown in Finder / Explorer — real content
/// previews for images, videos, PDFs, documents, etc.
fn get_os_thumbnail_base64(path: &str) -> Result<String, String> {
    let path_obj = Path::new(path);
    if !path_obj.exists() {
        return Err(format!("File does not exist: {}", path));
    }

    let (pixels, width, height) = native_thumbnailer::get_native_thumbnail_rgba(path, 256)?;

    let (cropped, w, h) = crop_transparent_bounds(&pixels, width, height);

    let mut png_data = Vec::new();
    {
        let encoder = PngEncoder::new(&mut png_data);
        encoder
            .write_image(&cropped, w, h, ColorType::Rgba8.into())
            .map_err(|e| format!("Failed to encode PNG: {}", e))?;
    }

    Ok(general_purpose::STANDARD.encode(&png_data))
}

/// Get a file preview: first try an OS-level thumbnail (Finder/Explorer style),
/// then fall back to the system file-type icon.
pub fn get_file_preview_or_icon(path: &str) -> Result<String, String> {
    // First, try to get a real OS thumbnail (works for images, videos, PDFs, docs, etc.)
    match get_os_thumbnail_base64(path) {
        Ok(thumb) => return Ok(thumb),
        Err(e) => {
            println!("OS thumbnail failed for '{}': {}", path, e);
        }
    }

    // Fall back to the system file-type icon
    let icon = file_icon_provider::get_file_icon(path, 256)
        .map_err(|e| format!("Failed to get icon: {:?}", e))?;

    encode_png(&icon.pixels, icon.width, icon.height)
}
