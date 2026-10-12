mod base;
mod blobs;
mod gutter_base;
mod hunks;
mod view;

pub(crate) use base::DiffBase;
pub(crate) use gutter_base::{BaseReader, GutterBase};
pub(crate) use hunks::{Hunk, gutter_markers, hunks_against};
pub(crate) use view::DiffView;
