use std::time::Duration;

use gpui_kit::component::ActiveTheme as _;
use gpui_kit::*;

use super::TerminalView;
use crate::terminal::settings::{BELL, BellMode};

const FLASH_DURATION: Duration = Duration::from_millis(150);
const FLASH_OPACITY: f32 = 0.08;

#[derive(Default)]
pub(super) struct Bell {
    flashing: bool,
    sound_pending: bool,
    _flash_task: Option<Task<()>>,
}

impl TerminalView {
    pub(crate) fn ring_bell(&mut self, cx: &mut Context<Self>) {
        match BELL.get(cx) {
            BellMode::None => return,
            BellMode::Visual => {
                self.bell.flashing = true;
                self.bell._flash_task = Some(Self::end_flash_later(cx));
            }
            BellMode::System => self.bell.sound_pending = true,
        }
        cx.notify();
    }

    pub(super) fn render_bell(&mut self, window: &Window, cx: &App) -> Option<Div> {
        if std::mem::take(&mut self.bell.sound_pending) {
            window.play_system_bell();
        }
        self.bell.flashing.then(|| {
            div()
                .absolute()
                .inset_0()
                .bg(cx.theme().foreground.opacity(FLASH_OPACITY))
        })
    }

    fn end_flash_later(cx: &mut Context<Self>) -> Task<()> {
        cx.spawn(async move |view, cx| {
            cx.background_executor().timer(FLASH_DURATION).await;
            let ended = view.update(cx, |view, cx| {
                view.bell.flashing = false;
                cx.notify();
            });
            if let Err(error) = ended {
                log::debug!("terminal view dropped during a visual bell: {error:#}");
            }
        })
    }
}
