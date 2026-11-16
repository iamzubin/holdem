// Copyright 2023-2023 CrabNebula Ltd.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::os::windows::ffi::OsStrExt;
use std::{ffi::c_void, iter::once, path::Path};
use windows::core::PCWSTR;
use windows::Win32::Foundation::*;
use windows::Win32::{
    Graphics::{
        Gdi::{CreateBitmap, HBITMAP},
        Imaging::{
            CLSID_WICImagingFactory, GUID_WICPixelFormat32bppPBGRA, IWICBitmapDecoder,
            IWICImagingFactory, WICConvertBitmapSource, WICDecodeMetadataCacheOnDemand,
        },
    },
    System::Com::{CoCreateInstance, CLSCTX_INPROC_SERVER},
};

use crate::Result;

pub(crate) fn read_bytes_to_hbitmap(bytes: &[u8]) -> Result<HBITMAP> {
    unsafe {
        let factory: IWICImagingFactory =
            CoCreateInstance(&CLSID_WICImagingFactory, None, CLSCTX_INPROC_SERVER)?;

        let stream = factory.CreateStream()?;
        stream.InitializeFromMemory(bytes)?;

        let decoder = factory.CreateDecoderFromStream(
            &stream,
            std::ptr::null(),
            WICDecodeMetadataCacheOnDemand,
        )?;

        decoder_to_hbitmap(decoder)
    }
}

pub(crate) fn read_path_to_hbitmap(path: &Path) -> Result<HBITMAP> {
    unsafe {
        let factory: IWICImagingFactory =
            CoCreateInstance(&CLSID_WICImagingFactory, None, CLSCTX_INPROC_SERVER)?;

        let path = dunce::canonicalize(path)?;
        let wide_path: Vec<u16> = path.as_os_str().encode_wide().chain(once(0)).collect();

        let decoder = factory.CreateDecoderFromFilename(
            PCWSTR::from_raw(wide_path.as_ptr()),
            None,
            GENERIC_READ,
            WICDecodeMetadataCacheOnDemand,
        )?;

        decoder_to_hbitmap(decoder)
    }
}

fn decoder_to_hbitmap(decoder: IWICBitmapDecoder) -> Result<HBITMAP> {
    unsafe {
        let frame = decoder.GetFrame(0)?;

        let mut width: u32 = 0;
        let mut height: u32 = 0;
        frame.GetSize(&mut width, &mut height)?;

        // Guard against overflow / absurd sizes from malformed images.
        let stride = width.checked_mul(4).ok_or(crate::Error::ImageNotFound)?;
        let len = (stride as usize)
            .checked_mul(height as usize)
            // CopyPixels takes a u32 buffer size (windows-rs unwraps the conversion).
            .filter(|&len| len > 0 && len <= u32::MAX as usize && height <= i32::MAX as u32)
            .ok_or(crate::Error::ImageNotFound)?;
        let mut pixel_buf: Vec<u8> = vec![0; len];
        let pixel_format = frame.GetPixelFormat()?;
        if pixel_format != GUID_WICPixelFormat32bppPBGRA {
            let bitmap_source = WICConvertBitmapSource(&GUID_WICPixelFormat32bppPBGRA, &frame)?;
            bitmap_source.CopyPixels(std::ptr::null(), stride, &mut pixel_buf)?;
        } else {
            frame.CopyPixels(std::ptr::null(), stride, &mut pixel_buf)?;
        }

        let hbitmap = CreateBitmap(
            width as i32,
            height as i32,
            1,
            32,
            Some(pixel_buf.as_ptr() as *const c_void),
        );
        if hbitmap.is_invalid() {
            return Err(windows::core::Error::from_thread().into());
        }
        Ok(hbitmap)
    }
}
