use gpui_kit::*;

use super::Workspace;
use crate::app::keymap::{GoToLine, RevealActiveFile};
use crate::explorer;

pub(super) fn bind_shortcuts(element: Stateful<Div>, cx: &Context<Workspace>) -> Stateful<Div> {
    element
        .on_action(cx.listener(Workspace::go_to_line))
        .on_action(cx.listener(Workspace::reveal_active_file))
}

impl Workspace {
    fn go_to_line(&mut self, _: &GoToLine, window: &mut Window, cx: &mut Context<Self>) {
        self.context
            .editor_area
            .update(cx, |area, cx| area.prompt_go_to_line(window, cx));
    }

    fn reveal_active_file(
        &mut self,
        _: &RevealActiveFile,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if explorer::reveal_active_file(&self.context, cx) {
            self.show_panel(explorer::PANEL.id, window, cx);
        }
    }
}
