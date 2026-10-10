use std::path::PathBuf;

use futures::channel::oneshot;
use gpui_kit::component::button::Button;
use gpui_kit::component::input::Input;
use gpui_kit::component::{Disableable as _, Icon, Sizable as _, h_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use super::field::Field;
use super::is_stacked;
use super::text::text_input;
use super::text_value::TextValue;
use crate::settings::Setting;

const PATH_WIDTH: Pixels = px(300.);

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum PathKind {
    File,
    Folder,
}

pub(crate) fn path<T: TextValue>(
    setting: &Setting<T>,
    title: impl Into<SharedString>,
    description: impl Into<SharedString>,
    kind: PathKind,
    placeholder: impl Into<SharedString>,
) -> Field {
    let placeholder = placeholder.into();
    let control_setting = setting.clone();
    Field::bound(setting, title, description, move |options, window, cx| {
        let input = text_input(&control_setting, placeholder.clone(), window, cx);
        let setting = control_setting.clone();
        h_flex()
            .gap_1()
            .map(|row| {
                if is_stacked(options) {
                    row.w_full()
                } else {
                    row.w(PATH_WIDTH)
                }
            })
            .child(
                div().flex_1().min_w_0().child(
                    Input::new(&input)
                        .disabled(options.is_disabled())
                        .with_size(options.size()),
                ),
            )
            .child(
                Button::new(SharedString::from(format!("browse-{}", setting.key)))
                    .outline()
                    .with_size(options.size())
                    .icon(Icon::empty().path(browse_icon(kind)))
                    .tooltip("Browse…")
                    .disabled(options.is_disabled())
                    .on_click(move |_, _, cx| browse(setting.clone(), kind, cx)),
            )
            .into_any_element()
    })
}

fn browse_icon(kind: PathKind) -> &'static str {
    match kind {
        PathKind::File => "icons/file.svg",
        PathKind::Folder => "icons/folder-open.svg",
    }
}

fn browse<T: TextValue>(setting: Setting<T>, kind: PathKind, cx: &mut App) {
    let paths = cx.prompt_for_paths(PathPromptOptions {
        files: kind == PathKind::File,
        directories: kind == PathKind::Folder,
        multiple: false,
        prompt: Some("Select".into()),
    });
    cx.spawn(async move |cx| {
        if let Some(path) = picked_path(paths.await) {
            let text = path.to_string_lossy().into_owned();
            cx.update(|cx| setting.set(T::from_text(&text), cx));
        }
    })
    .detach();
}

fn picked_path(
    paths: Result<anyhow::Result<Option<Vec<PathBuf>>>, oneshot::Canceled>,
) -> Option<PathBuf> {
    match paths {
        Ok(Ok(paths)) => paths?.into_iter().next(),
        Ok(Err(error)) => {
            log::warn!("failed to open the file picker: {error:#}");
            None
        }
        Err(_) => None,
    }
}
