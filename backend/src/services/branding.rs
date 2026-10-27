//! The shop logo prepared for emails and PDF documents: decoded from the uploads folder, flattened
//! onto white (no transparency surprises in mail clients or PDF/A) and scaled down to a sensible size.
//! A 4000 px logo would otherwise be sent with every email and shown at full size by Outlook.

use crate::models::StoreSettings;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};

#[derive(Debug)]
pub struct Logo {
    pub png: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

type CacheKey = (PathBuf, Option<std::time::SystemTime>, u32);

fn cache() -> &'static Mutex<HashMap<CacheKey, Arc<Logo>>> {
    static CACHE: OnceLock<Mutex<HashMap<CacheKey, Arc<Logo>>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Local file of a logo stored in the media library (`/uploads/<name>` or a full URL to it).
fn local_logo_path(logo_url: &str) -> Option<PathBuf> {
    let url = logo_url.trim();
    let name = url.split("/uploads/").nth(1)?.split(['?', '#']).next()?;
    if name.is_empty() || name.contains(['/', '\\']) || name.contains("..") {
        return None;
    }
    Some(PathBuf::from("uploads").join(name))
}

/// The logo as PNG whose longer side is at most `max_px`, or `None` (no logo, SVG, external URL,
/// unreadable file). Results are cached until the file changes.
pub fn logo_png(settings: &StoreSettings, max_px: u32) -> Option<Arc<Logo>> {
    let path = local_logo_path(&settings.logo_url)?;
    let modified = std::fs::metadata(&path).ok()?.modified().ok();
    let key = (path.clone(), modified, max_px);
    if let Some(hit) = cache().lock().ok()?.get(&key) {
        return Some(hit.clone());
    }

    let bytes = std::fs::read(&path).ok()?;
    let img = image::load_from_memory(&bytes).ok()?.to_rgba8();
    let (w, h) = img.dimensions();
    let scale = (max_px as f32 / w.max(h) as f32).min(1.0);
    let (nw, nh) = (((w as f32 * scale).round() as u32).max(1), ((h as f32 * scale).round() as u32).max(1));
    let resized = image::imageops::resize(&img, nw, nh, image::imageops::FilterType::Lanczos3);

    let mut flat = image::RgbImage::new(nw, nh);
    for (x, y, p) in resized.enumerate_pixels() {
        let a = p[3] as u32;
        let blend = |c: u8| ((c as u32 * a + 255 * (255 - a)) / 255) as u8;
        flat.put_pixel(x, y, image::Rgb([blend(p[0]), blend(p[1]), blend(p[2])]));
    }
    let mut png = Vec::new();
    image::DynamicImage::ImageRgb8(flat)
        .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
        .ok()?;

    let logo = Arc::new(Logo { png, width: nw, height: nh });
    if let Ok(mut c) = cache().lock() {
        c.retain(|(p, m, _), _| p != &path || *m == modified);
        c.insert(key, logo.clone());
    }
    Some(logo)
}

/// Async wrapper: decoding a large image must not block the async runtime.
pub async fn logo_png_async(settings: &StoreSettings, max_px: u32) -> Option<Arc<Logo>> {
    let settings = settings.clone();
    tokio::task::spawn_blocking(move || logo_png(&settings, max_px)).await.ok().flatten()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_media_library_files_are_read() {
        assert_eq!(local_logo_path("/uploads/logo.png"), Some(PathBuf::from("uploads").join("logo.png")));
        assert_eq!(local_logo_path("https://shop.example/uploads/logo.png?v=2"), Some(PathBuf::from("uploads").join("logo.png")));
        assert_eq!(local_logo_path("/uploads/../secret"), None);
        assert_eq!(local_logo_path("/uploads/a/b.png"), None);
        assert_eq!(local_logo_path("https://cdn.example/logo.png"), None);
    }
}
