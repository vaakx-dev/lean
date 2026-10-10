use std::path::PathBuf;

use anyhow::Result;
use gpui_kit::component::group_box::GroupBoxVariant;
use gpui_kit::component::setting::{SelectIndex, Settings as SettingsPanel};
use gpui_kit::*;

use super::json_file::edit_json_button;
use super::pages::{keybindings_page_index, settings_pages};
use crate::editor::EditorArea;
use crate::plugins::{FileView, FileViewEvent};
use crate::settings::Settings;

const TAB_KEY: &str = "lean://settings";
const TAB_TITLE: &str = "Settings";
const TAB_ICON: &str = "icons/settings.svg";

pub(crate) fn open_settings(editor_area: &Entity<EditorArea>, window: &mut Window, cx: &mut App) {
    open_page(editor_area, None, window, cx);
}

pub(crate) fn open_keybindings(
    editor_area: &Entity<EditorArea>,
    window: &mut Window,
    cx: &mut App,
) {
    let page = keybindings_page_index(cx);
    open_page(editor_area, Some(page), window, cx);
}

fn open_page(
    editor_area: &Entity<EditorArea>,
    page: Option<usize>,
    window: &mut Window,
    cx: &mut App,
) {
    editor_area.update(cx, |area, cx| {
        let key = PathBuf::from(TAB_KEY);
        if let Some(view) = area.view_entity::<SettingsView>(&key) {
            if let Some(page) = page {
                view.update(cx, |view, cx| view.show_page(page, cx));
            }
            area.show_view(&key, window, cx);
            return;
        }
        let area_handle = cx.weak_entity();
        let view = cx.new(|cx| SettingsView::new(area_handle, page.unwrap_or_default(), cx));
        area.open_view(
            key,
            TAB_TITLE.into(),
            TAB_ICON.into(),
            view.into(),
            window,
            cx,
        );
    });
}

struct SettingsView {
    editor_area: WeakEntity<EditorArea>,
    focus_handle: FocusHandle,
    start_page: usize,
    navigations: usize,
    _settings_changed: Subscription,
}

impl SettingsView {
    fn new(editor_area: WeakEntity<EditorArea>, start_page: usize, cx: &mut Context<Self>) -> Self {
        Self {
            editor_area,
            focus_handle: cx.focus_handle(),
            start_page,
            navigations: 0,
            _settings_changed: cx.observe_global::<Settings>(|_, cx| cx.notify()),
        }
    }

    fn show_page(&mut self, page: usize, cx: &mut Context<Self>) {
        self.start_page = page;
        self.navigations += 1;
        cx.notify();
    }
}

impl EventEmitter<FileViewEvent> for SettingsView {}

impl Focusable for SettingsView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl FileView for SettingsView {
    fn reload(&mut self, _: Vec<u8>, _: &mut Window, _: &mut Context<Self>) -> Result<()> {
        Ok(())
    }

    fn differs_from_saved(&self, _: &App) -> bool {
        false
    }
}

impl Render for SettingsView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let pages = settings_pages(cx).into_iter().map(|page| {
            let editor_area = self.editor_area.clone();
            page.title_suffix(move |_, _| div().ml_2().child(edit_json_button(editor_area.clone())))
        });
        div().track_focus(&self.focus_handle).size_full().child(
            SettingsPanel::new(("lean-settings", self.navigations))
                .default_selected_index(SelectIndex {
                    page_ix: self.start_page,
                    group_ix: None,
                })
                .with_group_variant(GroupBoxVariant::Outline)
                .pages(pages),
        )
    }
}
