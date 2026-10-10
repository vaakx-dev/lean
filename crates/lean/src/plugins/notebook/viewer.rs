use std::path::{Path, PathBuf};

use anyhow::Result;
use gpui_kit::component::ActiveTheme as _;
use gpui_kit::*;

use super::document::Document;
use super::{KernelStatus, Notebook};
use crate::editor::file_text::decode_text;
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
    let (text, line_ending) = decode_text(request.bytes)?;
    let document = Document::parse(&text)?;
    let folder = folder_of(&request.path);
    let toolchains = request.toolchains;
    let notebook =
        cx.new(|cx| Notebook::new(folder, document, line_ending, toolchains, window, cx));
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
        let (text, line_ending) = decode_text(bytes)?;
        let document = Document::parse(&text)?;
        let focus_handle = self.focus_handle.clone();
        let folder = self.folder.clone();
        let toolchains = self.toolchains.clone();
        *self = Notebook::new(folder, document, line_ending, toolchains, window, cx);
        self.focus_handle = focus_handle;
        cx.notify();
        Ok(())
    }

    fn differs_from_saved(&self, cx: &App) -> bool {
        self.has_unsaved_changes(cx)
    }

    fn snapshot(&mut self, cx: &mut Context<Self>) -> Option<Result<Snapshot>> {
        let line_ending = self.line_ending.for_save(LINE_ENDINGS.get(cx));
        let saved = self.saved_cells(cx);
        let notebook = cx.weak_entity();
        Some(self.content(cx).map(|content| {
            Snapshot::new(move || line_ending.encode(content)).on_saved(move |cx| {
                if let Err(error) = notebook.update(cx, |notebook, _| notebook.saved = saved) {
                    log::debug!("notebook dropped after saving: {error:#}");
                }
            })
        }))
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
