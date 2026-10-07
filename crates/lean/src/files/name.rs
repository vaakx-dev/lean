use std::path::Path;

use gpui_kit::SharedString;

pub(crate) fn file_name(path: &Path) -> SharedString {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
        .into()
}
