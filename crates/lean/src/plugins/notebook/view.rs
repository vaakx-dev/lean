use gpui_kit::component::alert::Alert;
use gpui_kit::component::button::Button;
use gpui_kit::component::scroll::ScrollableElement as _;
use gpui_kit::component::{ActiveTheme as _, Icon, Sizable as _, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use super::document::CellKind;
use super::keymap::{KEY_CONTEXT, bind_actions};
use super::loading::LoadState;
use super::{KernelStatus, Notebook};

const INSERTER_GROUP: &str = "notebook-inserter";

impl Notebook {
    pub(super) fn render_inserter(&self, index: usize, cx: &Context<Self>) -> Stateful<Div> {
        let theme = cx.theme();
        let always_visible = self.cells.is_empty();
        h_flex()
            .id(ElementId::named_usize("inserter", index))
            .group(INSERTER_GROUP)
            .relative()
            .h(px(24.))
            .ml(px(52.))
            .justify_center()
            .gap_1()
            .child(
                div()
                    .absolute()
                    .left_0()
                    .right_0()
                    .top(px(12.))
                    .h(px(1.))
                    .bg(theme.primary.opacity(0.4))
                    .invisible()
                    .group_hover(INSERTER_GROUP, |style| style.visible()),
            )
            .child(
                h_flex()
                    .gap_1()
                    .when(!always_visible, |buttons| {
                        buttons
                            .invisible()
                            .group_hover(INSERTER_GROUP, |style| style.visible())
                    })
                    .child(self.insert_button(index, CellKind::Code, cx))
                    .child(self.insert_button(index, CellKind::Markdown, cx)),
            )
    }

    fn insert_button(&self, index: usize, kind: CellKind, cx: &Context<Self>) -> Button {
        let (id, label) = match kind {
            CellKind::Markdown => ("insert-markdown", "Markdown"),
            CellKind::Code | CellKind::Raw => ("insert-code", "Code"),
        };
        Button::new(id)
            .outline()
            .xsmall()
            .icon(Icon::empty().path("icons/plus.svg"))
            .label(label)
            .bg(cx.theme().popover)
            .on_click(cx.listener(move |notebook, _, window, cx| {
                notebook.insert_cell(index, kind, window, cx)
            }))
    }

    fn render_row(&self, index: usize, window: &Window, cx: &Context<Self>) -> AnyElement {
        let row = match self.cells.get(index) {
            Some(cell) => self.render_cell(cell, index, window, cx),
            None => self.render_inserter(index, cx),
        };
        div().pr_4().child(row).into_any_element()
    }

    fn render_kernel_error(&self) -> Option<Alert> {
        let KernelStatus::Failed(error) = self.kernel_status() else {
            return None;
        };
        Some(
            Alert::error("kernel-error", error)
                .banner()
                .small()
                .flex_shrink_0(),
        )
    }

    fn render_cells(&self, cx: &Context<Self>) -> Div {
        let rows = list(
            self.list_state.clone(),
            cx.processor(|notebook, index: usize, window, cx| {
                notebook.render_row(index, window, cx)
            }),
        )
        .size_full()
        .pt_2()
        .pb(px(160.));
        div()
            .relative()
            .flex_1()
            .min_h_0()
            .child(rows)
            .vertical_scrollbar(&self.list_state)
    }
}

impl Render for Notebook {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let cx: &Context<Self> = cx;
        let root = v_flex()
            .id("notebook")
            .key_context(KEY_CONTEXT)
            .track_focus(&self.focus_handle)
            .size_full()
            .bg(cx.theme().background);
        let root = bind_actions(root, cx);
        match &self.load_state {
            LoadState::Loading => root,
            LoadState::Failed(error) => root.child(div().p_4().child(
                Alert::error("load-error", error.clone()).title("This notebook can't be opened"),
            )),
            LoadState::Loaded(_) => root
                .child(self.render_toolbar(cx))
                .children(self.render_kernel_error())
                .child(self.render_cells(cx)),
        }
    }
}
