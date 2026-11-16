//! Convert and resize images on the shelf. The result is always a new item
//! next to the original, which is never changed.

use crate::commands::file_ops::{clean_file_name, new_entry_folder};
use crate::drop_report::DropReport;
use crate::FileList;
use image::codecs::gif::GifDecoder;
use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::{PngDecoder, PngEncoder};
use image::codecs::webp::WebPDecoder;
use image::{
    AnimationDecoder, DynamicImage, ImageDecoder, ImageEncoder, ImageFormat, ImageReader, Rgba,
    RgbaImage,
};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager, State};
use tracing::warn;

/// JPEG quality: visually lossless for photos and screenshots, a fraction of
/// the PNG size.
const JPEG_QUALITY: u8 = 88;

/// Above this many pixels, downscaling uses a lighter filter: Lanczos on a
/// 100-megapixel photo needs well over a gigabyte of memory.
const LIGHT_FILTER_PIXELS: u64 = 24_000_000;

/// Extensions the `image` crate can read here.
const READABLE: &[&str] = &["png", "jpg", "jpeg", "gif", "bmp", "webp", "tif", "tiff", "ico"];

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OutputFormat {
    /// Keep the format when it can be written, else PNG.
    Same,
    Png,
    Jpeg,
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum Resize {
    None,
    /// A share of the original size, in percent (1..=100).
    Percent { value: u32 },
    /// Fit within a square of this many pixels, never enlarging.
    Fit { value: u32 },
}

#[derive(Debug, thiserror::Error)]
enum ImageToolError {
    #[error("animated images can't be converted without losing their animation")]
    Animated,
    #[error("{0}")]
    Image(#[from] image::ImageError),
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Save(#[from] crate::commands::file_ops::SaveError),
}

#[derive(Debug, Serialize)]
pub struct ImageResult {
    /// New items added to the shelf.
    pub added: usize,
    /// Images that could not be converted.
    pub failed: usize,
    /// Animated images, left alone: converting would keep one frame.
    pub animated: usize,
}

/// True for files this module can read.
pub fn is_readable_image(path: impl AsRef<Path>) -> bool {
    path.as_ref()
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| READABLE.contains(&e.to_ascii_lowercase().as_str()))
}

/// Converts and/or resizes the given items. Each result is added to the
/// active shelf as a new item; the originals stay as they are.
#[tauri::command]
pub async fn transform_images(
    app: AppHandle,
    file_list: State<'_, FileList>,
    ids: Vec<u64>,
    format: OutputFormat,
    resize: Resize,
) -> Result<ImageResult, String> {
    // Each result goes on the shelf its original is on.
    let sources: Vec<(PathBuf, u64)> = {
        let list = file_list.lock().map_err(|_| "Failed to acquire lock".to_string())?;
        list.iter()
            .filter(|f| ids.contains(&f.id) && f.available && is_readable_image(&f.path))
            .map(|f| (f.path.clone(), f.shelf))
            .collect()
    };
    let skipped = ids.len() - sources.len();
    let (done, failed, animated) = tauri::async_runtime::spawn_blocking(move || {
        let mut done: Vec<(PathBuf, u64)> = Vec::new();
        let (mut failed, mut animated) = (0, 0);
        for (source, shelf) in sources {
            match transform(&source, format, resize) {
                Ok(path) => done.push((path, shelf)),
                Err(ImageToolError::Animated) => animated += 1,
                Err(e) => {
                    warn!("Couldn't convert an image: {}", e);
                    failed += 1;
                }
            }
        }
        (done, failed, animated)
    })
    .await
    .map_err(|e| e.to_string())?;
    let added = done.len();
    let list = app.state::<FileList>().inner().clone();
    let mut shelves: Vec<u64> = done.iter().map(|(_, shelf)| *shelf).collect();
    shelves.sort_unstable();
    shelves.dedup();
    for shelf in shelves {
        let paths = done.iter().filter(|(_, s)| *s == shelf).map(|(p, _)| p.clone()).collect();
        crate::file_drop::add_paths_to(paths, Some(shelf), DropReport::new(), list.clone(), app.clone());
    }
    Ok(ImageResult { added, failed: failed + skipped, animated })
}

/// True for a GIF, WebP or PNG (APNG) with more than one frame.
fn is_animated(source: &Path, format: Option<ImageFormat>) -> Result<bool, ImageToolError> {
    let reader = || -> Result<BufReader<File>, ImageToolError> { Ok(BufReader::new(File::open(source)?)) };
    Ok(match format {
        Some(ImageFormat::Gif) => GifDecoder::new(reader()?)?.into_frames().take(2).count() > 1,
        Some(ImageFormat::WebP) => WebPDecoder::new(reader()?)?.has_animation(),
        Some(ImageFormat::Png) => PngDecoder::new(reader()?)?.is_apng()?,
        _ => false,
    })
}

/// Writes the converted image into a new drop folder and returns its path.
/// A failed write leaves nothing behind.
fn transform(source: &Path, format: OutputFormat, resize: Resize) -> Result<PathBuf, ImageToolError> {
    let reader = ImageReader::open(source)?.with_guessed_format()?;
    let source_format = reader.format();
    if is_animated(source, source_format)? {
        return Err(ImageToolError::Animated);
    }
    let mut decoder = reader.into_decoder()?;
    // Phone photos are stored sideways with a rotation tag; apply it, since
    // the result carries no tags. The colour profile is kept, so wide-gamut
    // photos keep their colours.
    let orientation = decoder.orientation()?;
    let icc_profile = decoder.icc_profile().ok().flatten();
    let mut image = DynamicImage::from_decoder(decoder)?;
    image.apply_orientation(orientation);

    let (width, height) = (image.width(), image.height());
    let (new_width, new_height) = target_size(width, height, resize);
    if (new_width, new_height) != (width, height) {
        image = if u64::from(width) * u64::from(height) > LIGHT_FILTER_PIXELS {
            image.thumbnail_exact(new_width, new_height)
        } else {
            image.resize_exact(new_width, new_height, image::imageops::FilterType::Lanczos3)
        };
    }

    let out_format = match format {
        OutputFormat::Png => ImageFormat::Png,
        OutputFormat::Jpeg => ImageFormat::Jpeg,
        OutputFormat::Same => match source_format {
            Some(f @ (ImageFormat::Png | ImageFormat::Jpeg | ImageFormat::Bmp)) => f,
            _ => ImageFormat::Png,
        },
    };
    let ext = if out_format == ImageFormat::Jpeg { "jpg" } else { out_format.extensions_str()[0] };
    let stem = source.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
    let stem = if resize == Resize::None { stem } else { format!("{stem} ({new_width}×{new_height})") };
    let folder = new_entry_folder("img-")?;
    let path = folder.join(clean_file_name(&stem, ext));
    let icc_profile = icc_profile.filter(|p| profile_fits(p, &image, out_format));
    let written = write_image(&image, &path, out_format, icc_profile);
    if written.is_err() {
        let _ = std::fs::remove_dir_all(&folder);
    }
    written.map(|()| path)
}

fn write_image(
    image: &DynamicImage,
    path: &Path,
    format: ImageFormat,
    icc_profile: Option<Vec<u8>>,
) -> Result<(), ImageToolError> {
    let mut out = BufWriter::new(File::create(path)?);
    match format {
        ImageFormat::Jpeg => {
            let mut encoder = JpegEncoder::new_with_quality(&mut out, JPEG_QUALITY);
            if let Some(profile) = icc_profile {
                let _ = encoder.set_icc_profile(profile);
            }
            // JPEG has no transparency: put it on white, as most apps do.
            flatten_on_white(image).write_with_encoder(encoder)?;
        }
        ImageFormat::Png => {
            let mut encoder = PngEncoder::new(&mut out);
            if let Some(profile) = icc_profile {
                let _ = encoder.set_icc_profile(profile);
            }
            image.write_with_encoder(encoder)?;
        }
        other => image.write_to(&mut out, other)?,
    }
    std::io::Write::flush(&mut out)?;
    Ok(())
}

/// True when an ICC profile describes the colours the output will hold: an
/// RGB profile for colour output, a grey one for greyscale PNG. (A CMYK
/// JPEG is decoded to RGB, so its CMYK profile would misdescribe it.)
fn profile_fits(profile: &[u8], image: &DynamicImage, format: ImageFormat) -> bool {
    let Some(space) = profile.get(16..20) else {
        return false;
    };
    let grey_output = format == ImageFormat::Png && !image.color().has_color();
    space == if grey_output { b"GRAY" } else { b"RGB " }
}

/// The size after `resize`, never enlarging and never below 1 px.
fn target_size(width: u32, height: u32, resize: Resize) -> (u32, u32) {
    let factor = match resize {
        Resize::None => 1.0,
        Resize::Percent { value } => f64::from(value.clamp(1, 100)) / 100.0,
        Resize::Fit { value } => {
            let longest = width.max(height).max(1);
            (f64::from(value.max(1)) / f64::from(longest)).min(1.0)
        }
    };
    let scale = |v: u32| ((f64::from(v) * factor).round() as u32).max(1);
    (scale(width), scale(height))
}

fn flatten_on_white(image: &DynamicImage) -> DynamicImage {
    if !image.color().has_alpha() {
        return DynamicImage::ImageRgb8(image.to_rgb8());
    }
    let rgba = image.to_rgba8();
    let mut out = RgbaImage::from_pixel(rgba.width(), rgba.height(), Rgba([255, 255, 255, 255]));
    image::imageops::overlay(&mut out, &rgba, 0, 0);
    DynamicImage::ImageRgb8(DynamicImage::ImageRgba8(out).to_rgb8())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fitting_never_enlarges_and_keeps_the_aspect() {
        assert_eq!(target_size(4000, 3000, Resize::Fit { value: 1920 }), (1920, 1440));
        assert_eq!(target_size(800, 600, Resize::Fit { value: 1920 }), (800, 600));
        assert_eq!(target_size(4000, 3000, Resize::Percent { value: 25 }), (1000, 750));
        assert_eq!(target_size(1, 1, Resize::Percent { value: 1 }), (1, 1));
    }

    #[test]
    fn transparency_turns_white_for_jpeg() {
        let clear = DynamicImage::ImageRgba8(RgbaImage::from_pixel(2, 2, Rgba([0, 0, 0, 0])));
        let flat = flatten_on_white(&clear).to_rgb8();
        assert_eq!(flat.get_pixel(0, 0).0, [255, 255, 255]);
    }

    #[test]
    fn profiles_only_travel_when_their_colour_space_fits() {
        let mut rgb = vec![0u8; 128];
        rgb[16..20].copy_from_slice(b"RGB ");
        let mut cmyk = rgb.clone();
        cmyk[16..20].copy_from_slice(b"CMYK");
        let colour = DynamicImage::ImageRgb8(image::RgbImage::new(1, 1));
        let grey = DynamicImage::ImageLuma8(image::GrayImage::new(1, 1));
        assert!(profile_fits(&rgb, &colour, ImageFormat::Jpeg));
        assert!(!profile_fits(&cmyk, &colour, ImageFormat::Jpeg));
        assert!(!profile_fits(&rgb, &grey, ImageFormat::Png));
        assert!(!profile_fits(&rgb[..10], &colour, ImageFormat::Png));
    }

    #[test]
    fn only_known_image_types_are_offered() {
        assert!(is_readable_image(r"C:\a\photo.JPG"));
        assert!(!is_readable_image(r"C:\a\photo.heic"));
        assert!(!is_readable_image(r"C:\a\notes.txt"));
    }
}
