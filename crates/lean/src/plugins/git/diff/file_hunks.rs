use std::path::Path;

use anyhow::Result;

use super::base::DiffBase;
use super::blobs::{Content, is_in_index, read_base};
use super::hunks::{Hunk, line_hunks};
use super::repository::open_repo_file;

pub(crate) fn hunks_for(
    repo_root: &Path,
    path: &Path,
    text: &str,
    base: DiffBase,
) -> Result<Vec<Hunk>> {
    let file = open_repo_file(repo_root, path)?;
    if file.repository.is_path_ignored(&file.relative_path)? {
        return Ok(Vec::new());
    }
    match read_base(&file, base)? {
        Content::Text(old) => line_hunks(&old, text),
        Content::Missing if !is_in_index(&file)? => Ok(Vec::new()),
        Content::Missing => line_hunks("", text),
        Content::Binary => Ok(Vec::new()),
    }
}
