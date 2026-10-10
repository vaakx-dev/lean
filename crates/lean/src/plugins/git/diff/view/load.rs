use std::path::Path;

use anyhow::Result;

use super::super::base::DiffBase;
use super::super::blobs::{read_base, read_index, read_working};
use super::super::repository::open_repo_file;
use super::document::DiffDocument;
use crate::files::Language;

pub(super) fn load_document(
    repo_root: &Path,
    path: &Path,
    base: DiffBase,
    context_lines: u32,
) -> Result<DiffDocument> {
    let file = open_repo_file(repo_root, path)?;
    let old = read_base(&file, base)?;
    let new = match base {
        DiffBase::Index => read_working(path)?,
        DiffBase::Staged => read_index(&file)?,
    };
    DiffDocument::build(old, new, Language::for_path(path).id(), context_lines)
}
