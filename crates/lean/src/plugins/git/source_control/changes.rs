mod file_row;
mod folder_row;
mod tree;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use gpui_kit::SharedString;

use crate::files::file_name;
use crate::plugins::git::repo::{FileStatus, GitState};

pub(super) use tree::push_tree;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum ChangeGroup {
    Staged,
    Unstaged,
}

impl ChangeGroup {
    pub(super) fn title(self) -> &'static str {
        match self {
            ChangeGroup::Staged => "Staged Changes",
            ChangeGroup::Unstaged => "Changes",
        }
    }
}

pub(super) struct ChangedFile {
    pub(super) path: PathBuf,
    pub(super) relative_path: PathBuf,
    pub(super) name: SharedString,
    pub(super) folder: SharedString,
    pub(super) status: FileStatus,
}

impl ChangedFile {
    fn new(path: &Path, status: FileStatus, workdir: &Path) -> Self {
        let relative_path = path
            .strip_prefix(workdir)
            .map(Path::to_path_buf)
            .unwrap_or_else(|_| path.to_path_buf());
        let folder = relative_path
            .parent()
            .map(|parent| parent.to_string_lossy().into_owned())
            .unwrap_or_default();
        Self {
            path: path.to_path_buf(),
            name: file_name(path),
            folder: folder.into(),
            relative_path,
            status,
        }
    }
}

#[derive(Default)]
pub(super) struct Changes {
    staged: Vec<Rc<ChangedFile>>,
    unstaged: Vec<Rc<ChangedFile>>,
}

impl Changes {
    pub(super) fn collect(git: &GitState) -> Self {
        Self {
            staged: collect_files(&git.staged, &git.workdir),
            unstaged: collect_files(&git.unstaged, &git.workdir),
        }
    }

    pub(super) fn files(&self, group: ChangeGroup) -> &[Rc<ChangedFile>] {
        match group {
            ChangeGroup::Staged => &self.staged,
            ChangeGroup::Unstaged => &self.unstaged,
        }
    }

    pub(super) fn files_in_folder(
        &self,
        group: ChangeGroup,
        folder: &Path,
    ) -> Vec<Rc<ChangedFile>> {
        self.files(group)
            .iter()
            .filter(|file| file.relative_path.starts_with(folder))
            .cloned()
            .collect()
    }
}

pub(super) fn paths_of(files: &[Rc<ChangedFile>]) -> Vec<PathBuf> {
    files.iter().map(|file| file.path.clone()).collect()
}

fn collect_files(statuses: &HashMap<PathBuf, FileStatus>, workdir: &Path) -> Vec<Rc<ChangedFile>> {
    let mut files: Vec<ChangedFile> = statuses
        .iter()
        .map(|(path, status)| ChangedFile::new(path, *status, workdir))
        .collect();
    files.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    files.into_iter().map(Rc::new).collect()
}
