use std::path::PathBuf;

use anyhow::Result;
use gpui_kit::component::group_box::GroupBoxVariant;
use gpui_kit::component::setting::{SelectIndex, Settings as SettingsPanel};
use gpui_kit::*;

use super::json_file::edit_json_button;
use super::pages::{Pages, settings_pages};
use crate::editor::EditorArea;
use crate::plugins::{FileView, FileViewEvent};
use crate::settings::Settings;
use crate::theme::ThemeCatalog;

const TAB_KEY: &str = "lean://settings";
const TAB_TITLE: &str = "Settings";
const TAB_ICON: &str = "icons/settings.svg";

#[derive(Clone, Copy, PartialEq)]
enum StartPage {
    First,
    Keybindings,
}

pub(crate) fn open_settings(editor_area: &Entity<EditorArea>, window: &mut Window, cx: &mut App) {
    open_page(editor_area, StartPage::First, window, cx);
}

pub(crate) fn open_keybindings(
    editor_area: &Entity<EditorArea>,
    window: &mut Window,
    cx: &mut App,
) {
    open_page(editor_area, StartPage::Keybindings, window, cx);
}

fn open_page(editor_area: &Entity<EditorArea>, page: StartPage, window: &mut Window, cx: &mut App) {
    editor_area.update(cx, |area, cx| {
        let key = PathBuf::from(TAB_KEY);
        if let Some(view) = area.view_entity::<SettingsView>(&key) {
            if page == StartPage::Keybindings {
                view.update(cx, |view, cx| view.show_keybindings(cx));
            }
            area.show_view(&key, window, cx);
            return;
        }
        let area_handle = cx.weak_entity();
        let view = cx.new(|cx| SettingsView::new(area_handle, page, cx));
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
    pages: Option<Pages>,
    start_page: usize,
    navigations: usize,
    _subscriptions: [Subscription; 2],
}

impl SettingsView {
    fn new(editor_area: WeakEntity<EditorArea>, page: StartPage, cx: &mut Context<Self>) -> Self {
        let mut view = Self {
            editor_area,
            focus_handle: cx.focus_handle(),
            pages: None,
            start_page: 0,
            navigations: 0,
            _subscriptions: [
                cx.observe_global::<Settings>(|view, cx| view.invalidate(cx)),
                cx.observe_global::<ThemeCatalog>(|view, cx| view.invalidate(cx)),
            ],
        };
        if page == StartPage::Keybindings {
            view.start_page = view.pages(cx).keybindings;
        }
        view
    }

    fn invalidate(&mut self, cx: &mut Context<Self>) {
        self.pages = None;
        cx.notify();
    }

    fn show_keybindings(&mut self, cx: &mut Context<Self>) {
        self.start_page = self.pages(cx).keybindings;
        self.navigations += 1;
        cx.notify();
    }

    fn pages(&mut self, cx: &App) -> &Pages {
        let editor_area = &self.editor_area;
        self.pages
            .get_or_insert_with(|| build_pages(editor_area, cx))
    }
}

fn build_pages(editor_area: &WeakEntity<EditorArea>, cx: &App) -> Pages {
    let mut pages = settings_pages(cx);
    pages.all = pages
        .all
        .into_iter()
        .map(|page| {
            let editor_area = editor_area.clone();
            page.title_suffix(move |_, _| div().ml_2().child(edit_json_button(editor_area.clone())))
        })
        .collect();
    pages
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
        let pages = self.pages(cx).all.clone();
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
