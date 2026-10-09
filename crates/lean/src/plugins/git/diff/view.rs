mod document;
mod load;
mod render;
mod row;
mod side;

use std::path::PathBuf;

use anyhow::Result;
use gpui_kit::component::ActiveTheme as _;
use gpui_kit::*;

use super::base::DiffBase;
use crate::files::file_name;
use crate::plugins::{FileView, FileViewEvent};
use crate::workspace::status_item;
use document::DiffDocument;
use load::load_document;

pub(crate) struct DiffView {
    repo_root: PathBuf,
    path: PathBuf,
    base: DiffBase,
    document: Option<DiffDocument>,
    error: Option<SharedString>,
    focus_handle: FocusHandle,
    scroll_handle: UniformListScrollHandle,
    _load: Task<()>,
}

impl EventEmitter<FileViewEvent> for DiffView {}

impl DiffView {
    pub(crate) fn open(
        repo_root: PathBuf,
        path: PathBuf,
        base: DiffBase,
        _: &mut Window,
        cx: &mut App,
    ) -> Entity<Self> {
        cx.new(|cx| {
            let mut view = Self {
                repo_root,
                path,
                base,
                document: None,
                error: None,
                focus_handle: cx.focus_handle(),
                scroll_handle: UniformListScrollHandle::new(),
                _load: Task::ready(()),
            };
            view.refresh(cx);
            view
        })
    }

    pub(crate) fn title(&self) -> SharedString {
        format!("{} ({})", file_name(&self.path), self.base.title()).into()
    }

    pub(crate) fn refresh(&mut self, cx: &mut Context<Self>) {
        let repo_root = self.repo_root.clone();
        let path = self.path.clone();
        let base = self.base;
        let load = cx.background_spawn(async move { load_document(&repo_root, &path, base) });
        self._load = cx.spawn(async move |view, cx| {
            let result = load.await;
            if let Err(error) = view.update(cx, |view, cx| view.show(result, cx)) {
                log::debug!("diff view dropped while loading: {error:#}");
            }
        });
    }

    fn show(&mut self, result: Result<DiffDocument>, cx: &mut Context<Self>) {
        match result {
            Ok(document) => {
                self.document = Some(document);
                self.error = None;
            }
            Err(error) => {
                log::warn!("failed to diff {}: {error:#}", self.path.display());
                self.error = Some(format!("Can't show the diff: {error:#}").into());
            }
        }
        cx.emit(FileViewEvent::StatusChanged);
        cx.notify();
    }
}

impl FileView for DiffView {
    fn reload(&mut self, _: Vec<u8>, _: &mut Window, cx: &mut Context<Self>) -> Result<()> {
        self.refresh(cx);
        Ok(())
    }

    fn differs_from_saved(&self, _: &App) -> bool {
        false
    }

    fn status_items(&self, cx: &App) -> Vec<AnyElement> {
        let theme = cx.theme();
        let mut items = vec![
            status_item("diff-base", theme)
                .child(self.base.comparison())
                .into_any_element(),
        ];
        if let Some(document) = &self.document {
            items.push(
                status_item("diff-stats", theme)
                    .child(
                        div()
                            .text_color(theme.success)
                            .child(format!("+{}", document.added)),
                    )
                    .child(
                        div()
                            .text_color(theme.danger)
                            .child(format!("−{}", document.removed)),
                    )
                    .into_any_element(),
            );
        }
        items
    }
}

impl Focusable for DiffView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
