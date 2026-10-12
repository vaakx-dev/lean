use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Result;

use super::base::DiffBase;
use super::blobs::{Content, blob_content, head_blob, index_blob, index_entry};
use crate::plugins::git::repo::{open_repository, workdir_relative};

#[derive(Clone)]
pub(crate) enum GutterBase {
    Unmarked,
    Text(Arc<str>),
}

pub(crate) struct BaseReader {
    repository: git2::Repository,
    workdir: PathBuf,
    index: git2::Index,
    base: DiffBase,
}

impl BaseReader {
    pub(crate) fn open(root: &Path, base: DiffBase) -> Result<Self> {
        let (repository, workdir) = open_repository(root)?;
        let index = repository.index()?;
        Ok(Self {
            repository,
            workdir,
            index,
            base,
        })
    }

    pub(crate) fn read(&self, path: &Path) -> Result<GutterBase> {
        let relative_path = workdir_relative(path, &self.workdir)?;
        let blob = match self.base {
            DiffBase::Head => head_blob(&self.repository, &relative_path)?,
            DiffBase::Index => index_blob(&self.repository, &self.index, &relative_path)?,
        };
        Ok(match blob_content(blob) {
            Content::Text(text) => GutterBase::Text(text.into()),
            Content::Missing if index_entry(&self.index, &relative_path).is_some() => {
                GutterBase::Text("".into())
            }
            Content::Missing | Content::Binary => GutterBase::Unmarked,
        })
    }
}
