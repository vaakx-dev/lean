mod base;
mod blobs;
mod file_hunks;
mod gutter;
mod hunks;
mod repository;
mod view;

pub(crate) use base::DiffBase;
pub(crate) use file_hunks::hunks_for;
pub(crate) use gutter::gutter_markers;
pub(crate) use hunks::Hunk;
pub(crate) use view::DiffView;
