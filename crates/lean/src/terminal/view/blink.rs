use std::time::Duration;

use gpui_kit::{Context, Task};

use super::TerminalView;

const BLINK_INTERVAL: Duration = Duration::from_millis(530);

#[derive(Default)]
pub(super) struct CursorBlink {
    hidden: bool,
    rendered: bool,
    running: bool,
    _task: Option<Task<()>>,
}

impl TerminalView {
    pub(super) fn blink_phase_visible(&mut self, focused: bool, cx: &mut Context<Self>) -> bool {
        let blinks = focused
            && self.is_running()
            && self
                .session()
                .is_some_and(|session| session.cursor_blinks());
        if !blinks {
            self.blink = CursorBlink::default();
            return true;
        }
        self.blink.rendered = true;
        if !self.blink.running {
            self.blink.running = true;
            self.blink._task = Some(Self::blink_loop(cx));
        }
        !self.blink.hidden
    }

    pub(crate) fn show_cursor(&mut self, cx: &mut Context<Self>) {
        if !self.blink.running {
            return;
        }
        self.blink.hidden = false;
        self.blink._task = Some(Self::blink_loop(cx));
    }

    fn blink_loop(cx: &mut Context<Self>) -> Task<()> {
        cx.spawn(async move |view, cx| {
            loop {
                cx.background_executor().timer(BLINK_INTERVAL).await;
                let blinking = view.update(cx, |view, cx| view.toggle_blink(cx));
                if !blinking.unwrap_or(false) {
                    break;
                }
            }
        })
    }

    fn toggle_blink(&mut self, cx: &mut Context<Self>) -> bool {
        if !self.blink.rendered {
            self.blink.hidden = false;
            self.blink.running = false;
            return false;
        }
        self.blink.rendered = false;
        self.blink.hidden = !self.blink.hidden;
        cx.notify();
        true
    }
}
