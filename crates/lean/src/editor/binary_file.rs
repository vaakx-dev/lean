use std::path::{Path, PathBuf};

use anyhow::Result;
use gpui_kit::component::button::Button;
use gpui_kit::component::{ActiveTheme as _, Icon, v_flex};
use gpui_kit::*;

use crate::files::format_size;
use crate::plugins::{AnyFileView, FileView, FileViewEvent};
use crate::workspace::status_item;

pub(super) struct BinaryFile {
    path: PathBuf,
    size: usize,
    focus_handle: FocusHandle,
}

impl EventEmitter<FileViewEvent> for BinaryFile {}

impl BinaryFile {
    pub(super) fn open(path: PathBuf, size: usize, cx: &mut App) -> AnyFileView {
        cx.new(|cx| Self {
            path,
            size,
            focus_handle: cx.focus_handle(),
        })
        .into()
    }
}

impl FileView for BinaryFile {
    fn reload(&mut self, bytes: Vec<u8>, _: &mut Window, cx: &mut Context<Self>) -> Result<()> {
        self.size = bytes.len();
        cx.emit(FileViewEvent::StatusChanged);
        cx.notify();
        Ok(())
    }

    fn differs_from_saved(&self, _: &App) -> bool {
        false
    }

    fn set_path(&mut self, path: &Path, cx: &mut Context<Self>) {
        self.path = path.to_path_buf();
        cx.notify();
    }

    fn status_items(&self, cx: &App) -> Vec<AnyElement> {
        vec![
            status_item("file-size", cx.theme())
                .child(format_size(self.size))
                .into_any_element(),
        ]
    }
}

impl Focusable for BinaryFile {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for BinaryFile {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let path = self.path.clone();
        v_flex()
            .id("binary-file")
            .track_focus(&self.focus_handle)
            .size_full()
            .items_center()
            .justify_center()
            .gap_3()
            .p_4()
            .child(
                Icon::empty()
                    .path("icons/file-x.svg")
                    .size(px(48.))
                    .text_color(theme.muted_foreground),
            )
            .child(
                div()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("This file can't be displayed"),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(theme.muted_foreground)
                    .child(format!(
                        "It is not a text file ({}).",
                        format_size(self.size)
                    )),
            )
            .child(
                Button::new("open-with-system")
                    .outline()
                    .label("Open with Default App")
                    .on_click(move |_, _, cx| cx.open_with_system(&path)),
            )
    }
}
