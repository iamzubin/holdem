use base64::{engine::general_purpose, Engine as _};
use image::codecs::png::PngEncoder;
use image::ColorType;
use image::ImageEncoder;

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

pub fn get_explorer_thumbnail_base64(input_path: &str) -> Result<String, String> {
    let icon = file_icon_provider::get_file_icon(input_path, 256)
        .map_err(|e| format!("Failed to get icon: {:?}", e))?;

    let (cropped, w, h) =
        crop_transparent_bounds(&icon.pixels, icon.width as usize, icon.height as usize);

    let mut png_data = Vec::new();
    {
        let encoder = PngEncoder::new(&mut png_data);
        encoder
            .write_image(&cropped, w, h, ColorType::Rgba8.into())
            .map_err(|e| format!("Failed to encode PNG: {}", e))?;
    }

    Ok(general_purpose::STANDARD.encode(&png_data))
}
