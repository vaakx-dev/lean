use std::path::Path;

use gpui_kit::SharedString;

use crate::project::DirectoryEntry;

const CASE_INSENSITIVE_NAMES: bool = cfg!(any(windows, target_os = "macos"));
const WINDOWS_INVALID_CHARACTERS: [char; 7] = ['<', '>', ':', '"', '|', '?', '*'];
const WINDOWS_RESERVED_NAMES: [&str; 4] = ["CON", "PRN", "AUX", "NUL"];

pub(super) fn name_error(
    name: &str,
    siblings: &[DirectoryEntry],
    renamed: Option<&Path>,
) -> Option<SharedString> {
    if name.is_empty() {
        return Some("A file or folder name must be provided.".into());
    }
    if name.contains(['/', '\\']) {
        return Some("A file or folder name cannot contain / or \\.".into());
    }
    if name == "." || name == ".." {
        return Some(format!("\"{name}\" is not a valid file or folder name.").into());
    }
    if name.chars().any(char::is_control) {
        return Some("A file or folder name cannot contain control characters.".into());
    }
    if cfg!(windows)
        && let Some(error) = windows_name_error(name)
    {
        return Some(error);
    }
    let exists = siblings
        .iter()
        .any(|sibling| is_same_name(&sibling.name, name) && Some(&*sibling.path) != renamed);
    exists.then(|| {
        format!(
            "A file or folder \"{name}\" already exists at this location. Please choose a different name."
        )
        .into()
    })
}

fn is_same_name(left: &str, right: &str) -> bool {
    if CASE_INSENSITIVE_NAMES {
        left.to_lowercase() == right.to_lowercase()
    } else {
        left == right
    }
}

fn windows_name_error(name: &str) -> Option<SharedString> {
    if name.contains(WINDOWS_INVALID_CHARACTERS) {
        return Some("A file or folder name cannot contain any of < > : \" | ? *".into());
    }
    if name.ends_with(['.', ' ']) {
        return Some("A file or folder name cannot end with a period or a space.".into());
    }
    if is_reserved_windows_name(name) {
        return Some(
            format!("\"{name}\" is reserved by Windows. Please choose a different name.").into(),
        );
    }
    None
}

fn is_reserved_windows_name(name: &str) -> bool {
    let stem = name
        .split('.')
        .next()
        .unwrap_or(name)
        .trim_end()
        .to_uppercase();
    if WINDOWS_RESERVED_NAMES.contains(&stem.as_str()) {
        return true;
    }
    match stem
        .strip_prefix("COM")
        .or_else(|| stem.strip_prefix("LPT"))
    {
        Some(digit) => matches!(digit, "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9"),
        None => false,
    }
}
