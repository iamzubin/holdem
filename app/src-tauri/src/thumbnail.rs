use base64::{engine::general_purpose, Engine as _};
use image::{ImageBuffer, Rgba};
use thumb_rs::{get_thumbnail, ThumbnailScale};

pub fn get_thumbnail_base64(file_path: &str) -> Result<String, String> {
    use std::time::Instant;
    let start_total = Instant::now();

    let start_gen = Instant::now();
    let thumb = get_thumbnail(file_path, ThumbnailScale::default()).map_err(|e| e.to_string())?;
    let duration_gen = start_gen.elapsed();

    let start_buf = Instant::now();
    let img = ImageBuffer::<Rgba<u8>, _>::from_raw(thumb.width, thumb.height, thumb.rgba)
        .ok_or_else(|| "Failed to create image buffer".to_string())?;
    let duration_buf = start_buf.elapsed();

    let start_png = Instant::now();
    let mut png_data = Vec::new();
    img.write_to(
        &mut std::io::Cursor::new(&mut png_data),
        image::ImageFormat::Png,
    )
    .map_err(|e| e.to_string())?;
    let duration_png = start_png.elapsed();

    let start_b64 = Instant::now();
    let encoded = general_purpose::STANDARD.encode(&png_data);
    let duration_b64 = start_b64.elapsed();

    println!(
        "[Thumbnail] {} - total: {:?}, gen: {:?}, buf: {:?}, png: {:?}, b64: {:?}",
        file_path,
        start_total.elapsed(),
        duration_gen,
        duration_buf,
        duration_png,
        duration_b64
    );

    Ok(encoded)
}
