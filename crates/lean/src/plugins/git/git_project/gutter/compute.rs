use std::path::PathBuf;

use gpui_kit::EntityId;
use gpui_kit::component::Rope;

use crate::plugins::git::diff::{BaseReader, DiffBase, GutterBase, Hunk, hunks_against};

pub(super) struct GutterBatch {
    pub(super) workdir: Option<PathBuf>,
    pub(super) base: DiffBase,
    pub(super) generation: u64,
    pub(super) jobs: Vec<GutterJob>,
}

pub(super) struct GutterJob {
    pub(super) path: PathBuf,
    pub(super) view: EntityId,
    pub(super) base: Option<GutterBase>,
    pub(super) text: Option<Rope>,
}

pub(super) struct GutterResult {
    pub(super) path: PathBuf,
    pub(super) view: EntityId,
    pub(super) hunks: Vec<Hunk>,
    pub(super) loaded_base: Option<GutterBase>,
}

impl GutterBatch {
    pub(super) fn compute(self) -> Vec<GutterResult> {
        let reader = self.open_reader();
        self.jobs
            .into_iter()
            .map(|job| job.compute(reader.as_ref()))
            .collect()
    }

    fn open_reader(&self) -> Option<BaseReader> {
        let needs_reader = self
            .jobs
            .iter()
            .any(|job| job.text.is_some() && job.base.is_none());
        let workdir = self.workdir.as_deref().filter(|_| needs_reader)?;
        BaseReader::open(workdir, self.base)
            .inspect_err(|error| log::debug!("failed to open the git repository: {error:#}"))
            .ok()
    }
}

impl GutterJob {
    fn compute(self, reader: Option<&BaseReader>) -> GutterResult {
        let loaded_base = match (&self.text, &self.base, reader) {
            (Some(_), None, Some(reader)) => reader
                .read(&self.path)
                .inspect_err(|error| {
                    log::debug!(
                        "failed to read the git base of {}: {error:#}",
                        self.path.display()
                    )
                })
                .ok(),
            _ => None,
        };
        let hunks = match (loaded_base.as_ref().or(self.base.as_ref()), &self.text) {
            (Some(base), Some(text)) => hunks_against(base, text).unwrap_or_else(|error| {
                log::debug!("failed to diff {}: {error:#}", self.path.display());
                Vec::new()
            }),
            _ => Vec::new(),
        };
        GutterResult {
            path: self.path,
            view: self.view,
            hunks,
            loaded_base,
        }
    }
}
