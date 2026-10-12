use std::path::Path;

use super::file_type::file_type;

#[derive(Clone, Copy)]
pub(crate) struct Language {
    id: &'static str,
    display_name: &'static str,
}

impl Language {
    pub(crate) const PLAIN: Language = Language::new("text", "Plain Text");

    pub(super) const fn new(id: &'static str, display_name: &'static str) -> Self {
        Self { id, display_name }
    }

    pub(crate) fn id(self) -> &'static str {
        self.id
    }

    pub(crate) fn display_name(self) -> &'static str {
        self.display_name
    }

    pub(crate) fn for_path(path: &Path) -> Language {
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        file_type(&name).language
    }
}
