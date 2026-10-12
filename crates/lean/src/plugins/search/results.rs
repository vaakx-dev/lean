use std::ops::Range;
use std::path::{Path, PathBuf};

use gpui_kit::SharedString;

pub(super) struct FileResult {
    pub(super) path: PathBuf,
    pub(super) name: SharedString,
    pub(super) folder: SharedString,
    pub(super) match_count: usize,
    pub(super) lines: Vec<LineResult>,
    pub(super) collapsed: bool,
}

pub(super) struct LineResult {
    pub(super) line: usize,
    pub(super) column: usize,
    pub(super) match_count: usize,
    pub(super) preview: SharedString,
    pub(super) highlights: Vec<Range<usize>>,
}

pub(super) struct SearchBatch {
    pub(super) files: Vec<FileResult>,
    pub(super) truncated: bool,
}

#[derive(Default)]
pub(super) struct SearchResults {
    pub(super) files: Vec<FileResult>,
    pub(super) match_count: usize,
    pub(super) truncated: bool,
}

impl FileResult {
    pub(super) fn new(root: &Path, path: PathBuf, lines: Vec<LineResult>) -> Self {
        let name = path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default()
            .into();
        let folder = path
            .parent()
            .and_then(|parent| parent.strip_prefix(root).ok())
            .map(|folder| folder.to_string_lossy().into_owned())
            .unwrap_or_default()
            .into();
        let match_count = lines.iter().map(|line| line.match_count).sum();
        Self {
            path,
            name,
            folder,
            match_count,
            lines,
            collapsed: false,
        }
    }
}

impl SearchResults {
    pub(super) fn append(&mut self, batch: SearchBatch) {
        self.match_count += batch
            .files
            .iter()
            .map(|file| file.match_count)
            .sum::<usize>();
        self.files.extend(batch.files);
        self.files.sort_by(|left, right| left.path.cmp(&right.path));
        self.truncated |= batch.truncated;
    }
}
