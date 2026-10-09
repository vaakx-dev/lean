use std::collections::HashMap;
use std::path::{Path, PathBuf};

use gpui_kit::{Context, Hsla};

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum FileDecoration {
    Letter { letter: &'static str, color: Hsla },
    Dot(Hsla),
}

#[derive(Default)]
pub(crate) struct FileDecorations {
    sources: Vec<(&'static str, HashMap<PathBuf, FileDecoration>)>,
}

impl FileDecorations {
    pub(crate) fn get(&self, path: &Path) -> Option<FileDecoration> {
        self.sources
            .iter()
            .find_map(|(_, decorations)| decorations.get(path).copied())
    }

    pub(crate) fn set(
        &mut self,
        source: &'static str,
        decorations: HashMap<PathBuf, FileDecoration>,
        cx: &mut Context<Self>,
    ) {
        match self.sources.iter_mut().find(|(id, _)| *id == source) {
            Some((_, current)) if *current == decorations => return,
            Some((_, current)) => *current = decorations,
            None => self.sources.push((source, decorations)),
        }
        cx.notify();
    }
}
