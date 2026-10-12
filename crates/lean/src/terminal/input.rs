mod keymap;
mod keys;
mod wheel;

use alacritty_terminal::grid::Scroll;
use alacritty_terminal::term::TermMode;
use gpui_kit::{Context, KeyDownEvent, ScrollWheelEvent, Window};

use super::view::TerminalView;

pub(crate) use keymap::{CopySelection, KEY_CONTEXT, Paste, bind_keys};

const RESTART_KEY: &str = "enter";

impl TerminalView {
    pub(super) fn key_down(
        &mut self,
        event: &KeyDownEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let keystroke = &event.keystroke;
        if !self.is_running() {
            if self.can_restart() && keystroke.key == RESTART_KEY {
                self.restart(cx);
                cx.stop_propagation();
            }
            return;
        }
        let Some(session) = self.session() else {
            return;
        };
        if let Some(scroll) = keys::history_scroll(keystroke) {
            session.scroll(scroll);
        } else if let Some(bytes) = keys::keystroke_bytes(keystroke, session.mode()) {
            session.clear_selection();
            session.scroll(Scroll::Bottom);
            session.write(bytes);
            self.show_cursor(cx);
        } else {
            return;
        }
        cx.stop_propagation();
        cx.notify();
    }

    pub(super) fn scroll_wheel(
        &mut self,
        event: &ScrollWheelEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(session) = self.session() else {
            return;
        };
        let line_height = session.size().line_height;
        self.scroll_remainder += event.delta.pixel_delta(line_height).y / line_height;
        let whole_lines = self.scroll_remainder.trunc();
        self.scroll_remainder -= whole_lines;
        let lines = whole_lines as i32;
        if lines == 0 {
            return;
        }
        let mode = session.mode();
        if mode.intersects(TermMode::MOUSE_MODE) {
            let cell = self.viewport_point(event.position, &session.size()).0;
            session.write(wheel::wheel_report_bytes(
                lines,
                cell,
                &event.modifiers,
                mode,
            ));
        } else if mode.contains(TermMode::ALT_SCREEN | TermMode::ALTERNATE_SCROLL) {
            session.write(keys::alternate_scroll_bytes(lines, mode));
        } else {
            session.scroll(Scroll::Delta(lines));
        }
        cx.stop_propagation();
        cx.notify();
    }

    pub(super) fn paste(&mut self, _: &Paste, _: &mut Window, cx: &mut Context<Self>) {
        let Some(session) = self.session() else {
            return;
        };
        let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) else {
            return;
        };
        session.clear_selection();
        session.scroll(Scroll::Bottom);
        session.write(keys::paste_bytes(&text, session.mode()));
        self.show_cursor(cx);
        cx.notify();
    }
}
