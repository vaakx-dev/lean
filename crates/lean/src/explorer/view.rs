use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::menu::ContextMenuExt as _;
use gpui_kit::component::{ActiveTheme as _, Icon, Sizable as _, h_flex, v_flex};
use gpui_kit::*;

use super::context_menu::build_context_menu;
use super::file_operations::{NewFile, NewFolder, bind_file_operations};
use super::keyboard::{KEY_CONTEXT, NOT_EDITING_CONTEXT};
use super::{Explorer, chevron_icon};
use crate::app::actions::{CollapseAllFolders, RefreshExplorer};

const HEADER_HEIGHT: Pixels = px(36.);

impl Explorer {
    fn key_context(&self) -> KeyContext {
        let mut context = KeyContext::new_with_defaults();
        context.add(KEY_CONTEXT);
        if self.editing.is_none() {
            context.add(NOT_EDITING_CONTEXT);
        }
        context
    }

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
            .h(HEADER_HEIGHT)
            .flex_shrink_0()
            .border_b_1()
            .border_color(separator)
            .pl_2()
            .pr_1p5()
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
            .child(
                header_button("new-file", "icons/file-plus.svg", "New File…").on_click(
                    cx.listener(|explorer, _, window, cx| {
                        cx.stop_propagation();
                        explorer.new_file(&NewFile, window, cx)
                    }),
                ),
            )
            .child(
                header_button("new-folder", "icons/folder-plus.svg", "New Folder…").on_click(
                    cx.listener(|explorer, _, window, cx| {
                        cx.stop_propagation();
                        explorer.new_folder(&NewFolder, window, cx)
                    }),
                ),
            )
            .child(
                header_button("refresh", "icons/refresh-cw.svg", "Refresh Explorer").on_click(
                    cx.listener(|explorer, _, window, cx| {
                        cx.stop_propagation();
                        explorer.refresh(&RefreshExplorer, window, cx)
                    }),
                ),
            )
            .child(
                header_button(
                    "collapse",
                    "icons/list-collapse.svg",
                    "Collapse Folders in Explorer",
                )
                .on_click(cx.listener(|explorer, _, window, cx| {
                    cx.stop_propagation();
                    explorer.collapse_all(&CollapseAllFolders, window, cx)
                })),
            )
    }
}

fn header_button(id: &'static str, icon: &'static str, tooltip: &'static str) -> Button {
    Button::new(id)
        .ghost()
        .xsmall()
        .icon(Icon::empty().path(icon))
        .tooltip(tooltip)
}

impl Render for Explorer {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let explorer = cx.weak_entity();
        let element = v_flex()
            .id("explorer")
            .key_context(self.key_context())
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::select_next))
            .on_action(cx.listener(Self::select_previous))
            .on_action(cx.listener(Self::expand_selected))
            .on_action(cx.listener(Self::collapse_selected))
            .on_action(cx.listener(Self::open_selected))
            .on_action(cx.listener(Self::refresh))
            .on_action(cx.listener(Self::collapse_all))
            .capture_any_mouse_down(cx.listener(Self::prepare_context_menu))
            .on_mouse_down(
                MouseButton::Right,
                cx.listener(|explorer, _, _, cx| explorer.clear_selection(cx)),
            );
        bind_file_operations(element, cx)
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
            .context_menu(move |menu, _, cx| match explorer.upgrade() {
                Some(explorer) => build_context_menu(menu, explorer.read(cx).menu_target()),
                None => menu,
            })
    }
}
