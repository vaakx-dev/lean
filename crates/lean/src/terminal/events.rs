use alacritty_terminal::event::Event;
use alacritty_terminal::vte::ansi::Rgb;
use gpui_kit::component::ActiveTheme as _;
use gpui_kit::{ClipboardItem, Context};

use super::render::{Palette, rgb_from_hsla};
use super::session::TerminalSession;
use super::view::TerminalView;

impl TerminalView {
    pub(super) fn handle_event(&mut self, event: Event, cx: &mut Context<Self>) {
        match event {
            Event::Wakeup => cx.notify(),
            Event::Exit | Event::ChildExit(_) => self.mark_exited(cx),
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
                    let color = terminal_color(&session, index, cx);
                    session.write(format(color).into_bytes());
                }
            }
            Event::TextAreaSizeRequest(format) => {
                if let Some(session) = self.session() {
                    session.write(format(session.size().window_size()).into_bytes());
                }
            }
            Event::MouseCursorDirty
            | Event::Title(_)
            | Event::ResetTitle
            | Event::CursorBlinkingChange
            | Event::Bell => {}
        }
    }

    fn write(&self, bytes: Vec<u8>) {
        if let Some(session) = self.session() {
            session.write(bytes);
        }
    }
}

fn terminal_color(session: &TerminalSession, index: usize, cx: &Context<TerminalView>) -> Rgb {
    let palette = Palette::new(cx.theme());
    let color = palette.indexed(index, session.lock().colors());
    rgb_from_hsla(color)
}
