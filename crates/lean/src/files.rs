mod file_type;
mod icon;
mod language;
pub(crate) mod settings;

use std::path::Path;

use gpui_kit::SharedString;

pub(crate) use icon::file_icon;
pub(crate) use language::Language;

const SIZE_UNITS: [&str; 4] = ["KB", "MB", "GB", "TB"];
const BYTES_PER_UNIT: f64 = 1024.;

pub(crate) fn file_name(path: &Path) -> SharedString {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
        .into()
}

pub(crate) fn format_size(bytes: usize) -> String {
    if bytes < 1024 {
        return format!("{bytes} bytes");
    }
    let mut size = bytes as f64 / BYTES_PER_UNIT;
    let mut unit = 0;
    while size >= BYTES_PER_UNIT && unit + 1 < SIZE_UNITS.len() {
        size /= BYTES_PER_UNIT;
        unit += 1;
    }
    format!("{size:.1} {}", SIZE_UNITS[unit])
}
