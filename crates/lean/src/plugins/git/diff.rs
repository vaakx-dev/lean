mod base;
mod blobs;
mod hunks;
mod view;

pub(crate) use base::DiffBase;
pub(crate) use hunks::{Hunk, gutter_markers, hunks_for};
pub(crate) use view::DiffView;
