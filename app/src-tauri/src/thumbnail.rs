use base64::{engine::general_purpose, Engine as _};
use image::{ImageBuffer, Rgba};
use thumb_rs::{get_thumbnail, ThumbnailScale};

pub fn get_thumbnail_base64(file_path: &str) -> Result<String, String> {
    let thumb = get_thumbnail(file_path, ThumbnailScale::default()).map_err(|e| e.to_string())?;
    let img = ImageBuffer::<Rgba<u8>, _>::from_raw(thumb.width, thumb.height, thumb.rgba)
        .ok_or_else(|| "Failed to create image buffer".to_string())?;
    let mut png_data = Vec::new();
    img.write_to(
        &mut std::io::Cursor::new(&mut png_data),
        image::ImageFormat::Png,
    )
    .map_err(|e| e.to_string())?;
    let encoded = general_purpose::STANDARD.encode(&png_data);

    Ok(encoded)
}
