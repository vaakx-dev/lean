use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::component::{RopeExt as _, WindowExt as _};
use gpui_kit::*;

use super::EditorArea;
use super::opening::GoTo;

const PROMPT_WIDTH: Pixels = px(380.);

impl EditorArea {
    pub(crate) fn prompt_go_to_line(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(file) = self.active_file() else {
            return;
        };
        let Some(text) = file.view.text(cx) else {
            return;
        };
        let file_id = file.id;
        let line_count = text.lines_len().max(1);
        let input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder(format!("Line 1 to {line_count}, or line:column"))
        });
        let area = cx.weak_entity();
        window
            .subscribe(&input, cx, move |input, event: &InputEvent, window, cx| {
                if !matches!(event, InputEvent::PressEnter { .. }) {
                    return;
                }
                let Some(target) = parse_target(&input.read(cx).value(), line_count) else {
                    return;
                };
                window.close_dialog(cx);
                area.update(cx, |area, cx| area.go_to_line(file_id, target, window, cx))
                    .ok();
            })
            .detach();
        let dialog_input = input.clone();
        window.open_dialog(cx, move |dialog, _, _| {
            dialog
                .width(PROMPT_WIDTH)
                .title("Go to Line")
                .child(Input::new(&dialog_input))
        });
        input.update(cx, |input, cx| input.focus(window, cx));
    }

    fn go_to_line(&mut self, id: u64, target: GoTo, window: &mut Window, cx: &mut Context<Self>) {
        let Some(index) = self.index_of_id(id) else {
            return;
        };
        self.go_to(index, target, window, cx);
        self.focus_active(window, cx);
    }
}

fn parse_target(text: &str, line_count: usize) -> Option<GoTo> {
    let mut parts = text.trim().splitn(2, [':', ',']);
    let line: usize = parts.next()?.trim().parse().ok()?;
    let column: usize = match parts.next() {
        Some(column) => column.trim().parse().ok()?,
        None => 1,
    };
    Some(GoTo {
        line: line.clamp(1, line_count) - 1,
        column: column.max(1) - 1,
    })
}
