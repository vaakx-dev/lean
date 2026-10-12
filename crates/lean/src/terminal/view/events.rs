use alacritty_terminal::event::Event;
use alacritty_terminal::vte::ansi::Rgb;
use gpui_kit::{ClipboardItem, Context};

use super::TerminalView;
use crate::terminal::render::{Palette, rgb_from_hsla};
use crate::terminal::session::TerminalSession;

impl TerminalView {
    pub(super) fn handle_event(&mut self, event: Event, cx: &mut Context<Self>) {
        match event {
            Event::Wakeup => {
                self.startup_output_arrived(cx);
                cx.notify();
            }
            Event::ChildExit(status) => self.mark_exited(status.code(), cx),
            Event::Exit => self.mark_exited(None, cx),
            Event::Title(title) => self.set_title(Some(title.into()), cx),
            Event::ResetTitle => self.set_title(None, cx),
            Event::Bell => self.ring_bell(cx),
            Event::CursorBlinkingChange => {
                self.show_cursor(cx);
                cx.notify();
            }
            Event::PtyWrite(text) => self.write(text.into_bytes()),
            Event::ClipboardStore(_, text) => {
                cx.write_to_clipboard(ClipboardItem::new_string(text))
            }
            Event::ClipboardLoad(_, format) => {
                if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
                    self.write(format(text.as_str()).into_bytes());
                }
            }
            Event::ColorRequest(index, format) => {
                if let Some(session) = self.session() {
                    let color = terminal_color(&session, index, &self.appearance.palette);
                    session.write(format(color).into_bytes());
                }
            }
            Event::TextAreaSizeRequest(format) => {
                if let Some(session) = self.session() {
                    session.write(format(session.size().window_size()).into_bytes());
                }
            }
            Event::MouseCursorDirty => {}
        }
    }

    fn write(&self, bytes: Vec<u8>) {
        if let Some(session) = self.session() {
            session.write(bytes);
        }
    }
}

fn terminal_color(session: &TerminalSession, index: usize, palette: &Palette) -> Rgb {
    let color = palette.indexed(index, session.lock().colors());
    rgb_from_hsla(color)
}
