use gpui_kit::component::ActiveTheme as _;
use gpui_kit::component::button::Button;
use gpui_kit::*;

use super::SourceControl;
use super::changes::ChangeGroup;
use super::rows::{Section, clickable_row, hover_actions};
use crate::ui::{chevron, count_badge, icon_button};

impl SourceControl {
    pub(super) fn render_section_row(
        &self,
        index: usize,
        section: Section,
        cx: &Context<Self>,
    ) -> AnyElement {
        let theme = cx.theme();
        let expanded = !self.collapsed_sections.contains(&section);
        let (title, count) = match section {
            Section::Changes(group) => (group.title(), self.changes.files(group).len()),
            Section::Commits => ("Commits", self.history.entries.len()),
        };
        clickable_row(index, theme)
            .px_1()
            .gap_1()
            .child(chevron(expanded, theme))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .truncate()
                    .text_sm()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(title),
            )
            .child(hover_actions(
                self.section_actions(section, cx),
                self.is_busy(),
            ))
            .child(count_badge(count, theme))
            .on_click(cx.listener(move |source_control, _, _, cx| {
                source_control.toggle_section(section, cx)
            }))
            .into_any_element()
    }

    fn section_actions(&self, section: Section, cx: &Context<Self>) -> Vec<Button> {
        match section {
            Section::Changes(ChangeGroup::Staged) => vec![
                icon_button("unstage-all", "icons/minus.svg", "Unstage All Changes").on_click(
                    cx.listener(|source_control, _, window, cx| {
                        source_control.unstage_all(window, cx)
                    }),
                ),
            ],
            Section::Changes(ChangeGroup::Unstaged) => vec![
                icon_button("discard-all", "icons/undo-2.svg", "Discard All Changes").on_click(
                    cx.listener(|source_control, _, window, cx| {
                        let files = source_control.changes.files(ChangeGroup::Unstaged).to_vec();
                        source_control.confirm_discard(files, window, cx)
                    }),
                ),
                icon_button("stage-all", "icons/plus.svg", "Stage All Changes").on_click(
                    cx.listener(|source_control, _, window, cx| {
                        source_control.stage_all(window, cx)
                    }),
                ),
            ],
            Section::Commits => Vec::new(),
        }
    }
}
