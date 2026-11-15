//! The QuickBiz app icon (same artwork as the mobile app), used for the window icon and in-app logo.
//! The 1024 px PNG is decoded once and box-filtered down, so small sizes stay crisp.

use eframe::egui;

const PNG: &[u8] = include_bytes!("../assets/images/icon.png");
const TEXTURE_ID: &str = "quickbiz-logo-texture";

/// Decodes the icon and shrinks it to `size` x `size` (size must divide the source width).
fn rgba(size: usize) -> Option<(Vec<u8>, usize)> {
    let src = eframe::icon_data::from_png_bytes(PNG).ok()?;
    let (w, h) = (src.width as usize, src.height as usize);
    if w != h || size == 0 || w % size != 0 {
        return None;
    }
    let f = w / size;
    let mut out = vec![0u8; size * size * 4];
    for oy in 0..size {
        for ox in 0..size {
            // average in premultiplied space so transparent corners don't bleed dark edges
            let (mut r, mut g, mut b, mut a) = (0u64, 0u64, 0u64, 0u64);
            for dy in 0..f {
                for dx in 0..f {
                    let i = ((oy * f + dy) * w + (ox * f + dx)) * 4;
                    let al = src.rgba[i + 3] as u64;
                    r += src.rgba[i] as u64 * al;
                    g += src.rgba[i + 1] as u64 * al;
                    b += src.rgba[i + 2] as u64 * al;
                    a += al;
                }
            }
            let o = (oy * size + ox) * 4;
            if a > 0 {
                out[o] = (r / a) as u8;
                out[o + 1] = (g / a) as u8;
                out[o + 2] = (b / a) as u8;
            }
            out[o + 3] = (a / (f * f) as u64) as u8;
        }
    }
    Some((out, size))
}

/// Icon for the window title bar and taskbar.
pub fn window_icon() -> Option<egui::IconData> {
    let (rgba, size) = rgba(256)?;
    Some(egui::IconData { rgba, width: size as u32, height: size as u32 })
}

/// Uploads the logo once; widgets fetch it with [`texture`].
pub fn install_texture(ctx: &egui::Context) {
    if let Some((rgba, size)) = rgba(128) {
        let img = egui::ColorImage::from_rgba_unmultiplied([size, size], &rgba);
        let handle = ctx.load_texture(TEXTURE_ID, img, egui::TextureOptions::LINEAR);
        ctx.data_mut(|d| d.insert_temp(egui::Id::new(TEXTURE_ID), handle));
    }
}

pub fn texture(ctx: &egui::Context) -> Option<egui::TextureHandle> {
    ctx.data(|d| d.get_temp::<egui::TextureHandle>(egui::Id::new(TEXTURE_ID)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icon_decodes_and_shrinks() {
        let (px, size) = rgba(128).expect("icon should decode");
        assert_eq!(px.len(), size * size * 4);
        // the artwork is opaque in the middle
        let mid = (size / 2 * size + size / 2) * 4;
        assert_eq!(px[mid + 3], 255);
        assert!(window_icon().is_some());
    }
}
