use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;
use std::time::Duration;

use anyhow::{bail, Context};

pub const MAX_BYTES: u64 = 40 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageKind {
    Jpeg,
    Png,
    Bmp,
    Webp,
    Gif,
}

impl ImageKind {
    pub fn ext(self) -> &'static str {
        match self {
            Self::Jpeg => "jpg",
            Self::Png => "png",
            Self::Bmp => "bmp",
            Self::Webp => "webp",
            Self::Gif => "gif",
        }
    }
}

pub fn sniff_image(header: &[u8]) -> Option<ImageKind> {
    if header.len() >= 3 && header[0] == 0xFF && header[1] == 0xD8 && header[2] == 0xFF {
        return Some(ImageKind::Jpeg);
    }
    if header.len() >= 8 && header.starts_with(b"\x89PNG\r\n\x1a\n") {
        return Some(ImageKind::Png);
    }
    if header.len() >= 2 && header.starts_with(b"BM") {
        return Some(ImageKind::Bmp);
    }
    if header.len() >= 12 && header.starts_with(b"RIFF") && &header[8..12] == b"WEBP" {
        return Some(ImageKind::Webp);
    }
    if header.len() >= 6 && (header.starts_with(b"GIF87a") || header.starts_with(b"GIF89a")) {
        return Some(ImageKind::Gif);
    }
    None
}

pub fn download_to(url: &str, dest: &Path) -> anyhow::Result<ImageKind> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    let tmp = dest.with_extension("part");
    let _ = fs::remove_file(&tmp);

    let resp = ureq::get(url)
        .timeout(Duration::from_secs(30))
        .set("User-Agent", "WallShift/0.1")
        .set("Accept", "image/*,*/*;q=0.8")
        .call()
        .with_context(|| format!("GET {url}"))?;

    let mut reader = resp.into_reader().take(MAX_BYTES + 1);
    let mut file = fs::File::create(&tmp)
        .with_context(|| format!("create {}", tmp.display()))?;
    let copied = io::copy(&mut reader, &mut file).context("stream image")?;
    file.flush()?;
    drop(file);

    if copied > MAX_BYTES {
        let _ = fs::remove_file(&tmp);
        bail!("image larger than {MAX_BYTES} bytes");
    }
    if copied < 16 {
        let _ = fs::remove_file(&tmp);
        bail!("response too small to be an image");
    }

    let mut header = [0u8; 16];
    let mut probe = fs::File::open(&tmp)?;
    probe.read_exact(&mut header)?;
    drop(probe);

    let Some(kind) = sniff_image(&header) else {
        let _ = fs::remove_file(&tmp);
        bail!("response is not jpeg/png/bmp/webp/gif");
    };

    fs::rename(&tmp, dest).with_context(|| format!("rename to {}", dest.display()))?;
    Ok(kind)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sniffs_jpeg() {
        assert_eq!(sniff_image(&[0xFF, 0xD8, 0xFF, 0xE0]), Some(ImageKind::Jpeg));
    }

    #[test]
    fn sniffs_png() {
        let mut h = [0u8; 16];
        h[..8].copy_from_slice(b"\x89PNG\r\n\x1a\n");
        assert_eq!(sniff_image(&h), Some(ImageKind::Png));
    }

    #[test]
    fn sniffs_webp() {
        let mut h = [0u8; 16];
        h[..4].copy_from_slice(b"RIFF");
        h[8..12].copy_from_slice(b"WEBP");
        assert_eq!(sniff_image(&h), Some(ImageKind::Webp));
    }

    #[test]
    fn rejects_html() {
        assert_eq!(sniff_image(b"<!DOCTYPE html>.."), None);
    }
}
