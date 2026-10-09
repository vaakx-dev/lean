use gpui_kit::component::Theme;

use super::hunks::{Hunk, HunkKind};
use crate::plugins::GutterMarker;

pub(crate) fn gutter_markers(hunks: &[Hunk], theme: &Theme) -> Vec<GutterMarker> {
    hunks
        .iter()
        .map(|hunk| GutterMarker {
            lines: hunk.new_lines.clone(),
            color: match hunk.kind {
                HunkKind::Added => theme.success,
                HunkKind::Modified => theme.info,
                HunkKind::Deleted => theme.danger,
            },
        })
        .collect()
}
