use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::{ActiveTheme as _, Icon, Sizable as _, h_flex, v_flex};
use gpui_kit::*;

use super::keyboard::KEY_CONTEXT;
use super::{Explorer, chevron_icon};
use crate::app::actions::{CollapseAllFolders, RefreshExplorer};

impl Explorer {
    fn is_scrolled(&self) -> bool {
        self.scroll_handle.0.borrow().base_handle.offset().y < px(0.)
    }

    fn render_header(&self, cx: &Context<Self>) -> Stateful<Div> {
        let theme = cx.theme();
        let project_name = self.project.read(cx).name().clone();
        let separator = if self.is_scrolled() {
            theme.border
        } else {
            theme.transparent
        };
        h_flex()
            .id("explorer-root")
            .h(px(30.))
            .flex_shrink_0()
            .border_b_1()
            .border_color(separator)
            .pl_2()
            .pr_1()
            .gap_1()
            .cursor_pointer()
            .on_click(cx.listener(|explorer, _, _, cx| explorer.toggle_root(cx)))
            .child(
                Icon::empty()
                    .path(chevron_icon(self.root_expanded))
                    .size_4()
                    .text_color(theme.muted_foreground),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .overflow_hidden()
                    .text_ellipsis()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(project_name),
            )
            .child(header_button(
                "refresh",
                "icons/refresh-cw.svg",
                "Refresh Explorer",
                RefreshExplorer,
            ))
            .child(header_button(
                "collapse",
                "icons/list-collapse.svg",
                "Collapse Folders in Explorer",
                CollapseAllFolders,
            ))
    }
}

fn header_button(
    id: &'static str,
    icon: &'static str,
    tooltip: &'static str,
    action: impl Action,
) -> Button {
    Button::new(id)
        .ghost()
        .xsmall()
        .icon(Icon::empty().path(icon))
        .tooltip(tooltip)
        .on_click(move |_, window, cx| window.dispatch_action(action.boxed_clone(), cx))
}

impl Render for Explorer {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .id("explorer")
            .key_context(KEY_CONTEXT)
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::select_next))
            .on_action(cx.listener(Self::select_previous))
            .on_action(cx.listener(Self::expand_selected))
            .on_action(cx.listener(Self::collapse_selected))
            .on_action(cx.listener(Self::open_selected))
            .on_action(cx.listener(Self::refresh))
            .on_action(cx.listener(Self::collapse_all))
            .size_full()
            .child(self.render_header(cx))
            .child(
                div().flex_1().min_h_0().overflow_hidden().child(
                    uniform_list(
                        "explorer-entries",
                        self.visible.len(),
                        cx.processor(Self::render_rows),
                    )
                    .track_scroll(&self.scroll_handle)
                    .size_full()
                    .px_1()
                    .pb_2(),
                ),
            )
    }
}
