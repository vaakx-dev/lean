use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow};
use gpui_kit::component::ActiveTheme as _;
use gpui_kit::*;

use super::loading::LoadState;
use super::saved::serialize;
use super::settings::SAVE_OUTPUTS;
use super::{KernelStatus, Notebook};
use crate::files::settings::LINE_ENDINGS;
use crate::plugins::{AnyFileView, FileView, FileViewer, OpenRequest, Snapshot};
use crate::workspace::status_item;

const NOTEBOOK_EXTENSION: &str = "ipynb";

pub(super) const NOTEBOOK_VIEWER: FileViewer = FileViewer {
    priority: 10,
    handles: is_notebook,
    prepare: None,
    open,
};

fn is_notebook(path: &Path) -> bool {
    path.extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case(NOTEBOOK_EXTENSION))
}

fn open(request: OpenRequest, window: &mut Window, cx: &mut App) -> Result<AnyFileView> {
    let folder = folder_of(&request.path);
    let toolchains = request.toolchains;
    let bytes = request.bytes;
    let notebook = cx.new(|cx| {
        let mut notebook = Notebook::new(folder, toolchains, window, cx);
        notebook.load_in_background(bytes, window, cx);
        notebook
    });
    Ok(notebook.into())
}

fn folder_of(path: &Path) -> PathBuf {
    path.parent().map(Path::to_path_buf).unwrap_or_default()
}

impl FileView for Notebook {
    fn reload(
        &mut self,
        bytes: Vec<u8>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Result<()> {
        self.load_now(bytes, window, cx)
    }

    fn differs_from_saved(&self, cx: &App) -> bool {
        self.has_unsaved_changes(cx)
    }

    fn snapshot(&mut self, cx: &mut Context<Self>) -> Option<Result<Snapshot>> {
        let LoadState::Loaded(line_ending) = &self.load_state else {
            return Some(Err(anyhow!("the notebook didn't load")));
        };
        let line_ending = line_ending.for_save(LINE_ENDINGS.get(cx));
        let with_outputs = SAVE_OUTPUTS.get(cx);
        let fields = self.fields.clone();
        let cells = self.saved_cells(cx);
        let saved = cells.clone();
        let notebook = cx.weak_entity();
        let encode = move || line_ending.encode(serialize(&fields, &cells, with_outputs));
        Some(Ok(Snapshot::new(encode).on_saved(move |cx| {
            notebook
                .update(cx, |notebook, cx| notebook.mark_saved(saved, cx))
                .ok();
        })))
    }

    fn set_path(&mut self, path: &Path, _: &mut Context<Self>) {
        self.folder = folder_of(path);
    }

    fn status_items(&self, cx: &App) -> Vec<AnyElement> {
        let theme = cx.theme();
        vec![
            status_item("cell", theme)
                .child(self.selection_label())
                .into_any_element(),
            status_item("kernel", theme)
                .child(self.kernel_status().label())
                .into_any_element(),
            status_item("language", theme)
                .child("Jupyter Notebook")
                .into_any_element(),
        ]
    }

    fn running_work(&self, _: &App) -> Option<&'static str> {
        match self.kernel_status() {
            KernelStatus::Starting | KernelStatus::Busy => Some("running kernel"),
            KernelStatus::Stopped | KernelStatus::Idle | KernelStatus::Failed(_) => None,
        }
    }
}
