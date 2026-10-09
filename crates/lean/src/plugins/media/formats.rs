use std::path::Path;

use gpui_kit::ImageFormat;

const RASTER_EXTENSIONS: [(&str, ImageFormat); 9] = [
    ("png", ImageFormat::Png),
    ("jpg", ImageFormat::Jpeg),
    ("jpeg", ImageFormat::Jpeg),
    ("gif", ImageFormat::Gif),
    ("webp", ImageFormat::Webp),
    ("bmp", ImageFormat::Bmp),
    ("ico", ImageFormat::Ico),
    ("tif", ImageFormat::Tiff),
    ("tiff", ImageFormat::Tiff),
];

const SIGNATURES: [(&[u8], ImageFormat); 7] = [
    (b"\x89PNG\r\n\x1a\n".as_slice(), ImageFormat::Png),
    (b"\xff\xd8\xff".as_slice(), ImageFormat::Jpeg),
    (b"GIF8".as_slice(), ImageFormat::Gif),
    (b"BM".as_slice(), ImageFormat::Bmp),
    (b"\x00\x00\x01\x00".as_slice(), ImageFormat::Ico),
    (b"II*\x00".as_slice(), ImageFormat::Tiff),
    (b"MM\x00*".as_slice(), ImageFormat::Tiff),
];

const SVG_EXTENSION: &str = "svg";

pub(super) fn raster_format(path: &Path) -> Option<ImageFormat> {
    let extension = path.extension()?;
    RASTER_EXTENSIONS
        .into_iter()
        .find(|(name, _)| extension.eq_ignore_ascii_case(name))
        .map(|(_, format)| format)
}

pub(super) fn detect_format(path: &Path, bytes: &[u8]) -> ImageFormat {
    sniff_format(bytes)
        .or_else(|| raster_format(path))
        .unwrap_or(ImageFormat::Png)
}

pub(super) fn is_svg(path: &Path) -> bool {
    path.extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case(SVG_EXTENSION))
}

pub(super) fn format_name(format: ImageFormat) -> &'static str {
    match format {
        ImageFormat::Png => "PNG",
        ImageFormat::Jpeg => "JPEG",
        ImageFormat::Webp => "WebP",
        ImageFormat::Gif => "GIF",
        ImageFormat::Svg => "SVG",
        ImageFormat::Bmp => "BMP",
        ImageFormat::Tiff => "TIFF",
        ImageFormat::Ico => "ICO",
        ImageFormat::Pnm => "PNM",
    }
}

fn sniff_format(bytes: &[u8]) -> Option<ImageFormat> {
    if is_webp(bytes) {
        return Some(ImageFormat::Webp);
    }
    SIGNATURES
        .into_iter()
        .find(|(signature, _)| bytes.starts_with(signature))
        .map(|(_, format)| format)
}

fn is_webp(bytes: &[u8]) -> bool {
    bytes.len() >= 12 && bytes.starts_with(b"RIFF") && &bytes[8..12] == b"WEBP"
}
