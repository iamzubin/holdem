use std::{ffi::c_void, os::windows::ffi::OsStrExt};
use windows::{
    core::{Error, HRESULT, PCWSTR},
    Win32::{
        Foundation::{HWND, SIZE},
        Graphics::Gdi::{BITMAPINFO, BITMAPINFOHEADER, DIB_RGB_COLORS, GetDIBits, GetDC},
        System::Com::{CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED},
        UI::Shell::{IShellItemImageFactory, SHCreateItemFromParsingName, SIIGBF},
    },
};

use base64::{engine::general_purpose, Engine as _}; // new base64 import
use image::codecs::png::PngEncoder;
use image::ColorType;
use image::ImageEncoder;


fn wide(s: &str) -> Vec<u16> {
    std::ffi::OsStr::new(s).encode_wide().chain(Some(0)).collect()
}

fn crop_transparent_bounds(p: &[u8], w: usize, h: usize) -> (Vec<u8>, u32, u32) {
    let stride = w * 4;
    let (mut top, mut bottom, mut left, mut right) = (h, 0, w, 0);
    for y in 0..h {
        for x in 0..w {
            let i = y * stride + x * 4;
            if p[i + 3] > 10 {
                if y < top { top = y }
                if y > bottom { bottom = y }
                if x < left { left = x }
                if x > right { right = x }
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

pub fn get_explorer_thumbnail_base64(input_path: &str) -> windows::core::Result<String> {
    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok();

        let shell: IShellItemImageFactory = SHCreateItemFromParsingName(
            PCWSTR(wide(input_path).as_ptr()), None)?;

        let flags = [SIIGBF(0x0), SIIGBF(0x4)];
        let mut bmp_opt = None;
        for f in flags {
            if let Ok(h) = shell.GetImage(SIZE { cx: 256, cy: 256 }, f) {
                bmp_opt = Some(h);
                break;
            }
        }
        let bmp = bmp_opt.ok_or_else(|| Error::new(HRESULT(0x80004005u32 as i32), "No image or icon".into()))?;

        let dc = GetDC(HWND(0));
        let mut info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: 256,
                biHeight: -256,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: 0,
                ..Default::default()
            },
            bmiColors: [Default::default(); 1],
        };

        let mut buf = vec![0u8; 256 * 256 * 4];
        GetDIBits(dc, bmp, 0, 256, Some(buf.as_mut_ptr() as *mut c_void), &mut info, DIB_RGB_COLORS);

        for px in buf.chunks_mut(4) {
            px.swap(0, 2);
        }

        let (cropped, w, h) = crop_transparent_bounds(&buf, 256, 256);

        let mut png_data = Vec::new();
        {
            let encoder = PngEncoder::new(&mut png_data);
            encoder.write_image(&cropped, w, h, ColorType::Rgba8.into())
                .map_err(|e| Error::new(HRESULT(0x80004005u32 as i32), e.to_string().into()))?;
        }

        CoUninitialize();

        Ok(general_purpose::STANDARD.encode(&png_data))
    }
}