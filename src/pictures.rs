//! Profile photos and business logos: shrink a picture before uploading it, and decode
//! downloaded pictures into textures.

use eframe::egui::ColorImage;
use image::imageops::FilterType;

fn decode(bytes: &[u8]) -> Result<image::DynamicImage, String> {
    image::load_from_memory(bytes).map_err(|_| "That file is not a picture QuickBiz can read. Use a JPG or PNG.".to_string())
}

/// Crops to a centred square and shrinks to at most `max` pixels, like the phone's square crop.
fn square(img: image::DynamicImage, max: u32) -> image::DynamicImage {
    let (w, h) = (img.width(), img.height());
    let side = w.min(h);
    let cropped = img.crop_imm((w - side) / 2, (h - side) / 2, side, side);
    if side > max { cropped.resize_exact(max, max, FilterType::Triangle) } else { cropped }
}

/// Turns any JPG or PNG into a small square JPEG ready to upload.
pub fn to_jpeg(bytes: &[u8], max: u32) -> Result<Vec<u8>, String> {
    let img = square(decode(bytes)?, max).to_rgb8();
    let mut out = Vec::new();
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, 82).encode_image(&img).map_err(|e| e.to_string())?;
    Ok(out)
}

/// Decodes a downloaded picture into something egui can draw.
pub fn thumbnail(bytes: &[u8], max: u32) -> Result<ColorImage, String> {
    let img = square(decode(bytes)?, max).to_rgba8();
    let size = [img.width() as usize, img.height() as usize];
    Ok(ColorImage::from_rgba_unmultiplied(size, img.as_raw()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shrinks_to_a_square_jpeg() {
        let img = image::RgbImage::from_pixel(900, 600, image::Rgb([45, 98, 237]));
        let mut png = Vec::new();
        image::DynamicImage::ImageRgb8(img).write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png).unwrap();
        let jpeg = to_jpeg(&png, 512).unwrap();
        let back = image::load_from_memory(&jpeg).unwrap();
        assert_eq!((back.width(), back.height()), (512, 512));
        let t = thumbnail(&jpeg, 128).unwrap();
        assert_eq!(t.size, [128, 128]);
        assert!(to_jpeg(b"not a picture", 512).is_err());
    }
}
