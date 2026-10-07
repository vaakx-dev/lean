use std::path::Path;

use gpui_kit::SharedString;

use crate::project::DirectoryEntry;

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
    let exists = siblings
        .iter()
        .any(|sibling| sibling.name == name && Some(sibling.path.as_path()) != renamed);
    exists.then(|| {
        format!(
            "A file or folder \"{name}\" already exists at this location. Please choose a different name."
        )
        .into()
    })
}
