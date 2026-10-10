use gpui_kit::component::button::{Button, ButtonGroup, ButtonVariants as _};
use gpui_kit::component::{ActiveTheme as _, Icon, Selectable as _, Sizable as _};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use super::SvgViewer;
use crate::plugins::media::settings::SvgMode;

const MINIMAP_CLEARANCE: Pixels = px(116.);

#[derive(Clone, Copy, PartialEq)]
pub(super) enum Mode {
    Code,
    Preview,
    Split,
}

impl From<SvgMode> for Mode {
    fn from(mode: SvgMode) -> Self {
        match mode {
            SvgMode::Code => Mode::Code,
            SvgMode::Preview => Mode::Preview,
            SvgMode::Split => Mode::Split,
        }
    }
}

impl Mode {
    const ALL: [Mode; 3] = [Mode::Code, Mode::Preview, Mode::Split];

    pub(super) fn shows_code(self) -> bool {
        matches!(self, Mode::Code | Mode::Split)
    }

    pub(super) fn shows_preview(self) -> bool {
        matches!(self, Mode::Preview | Mode::Split)
    }

    fn id(self) -> &'static str {
        match self {
            Mode::Code => "svg-mode-code",
            Mode::Preview => "svg-mode-preview",
            Mode::Split => "svg-mode-split",
        }
    }

    fn icon(self) -> &'static str {
        match self {
            Mode::Code => "icons/code.svg",
            Mode::Preview => "icons/eye.svg",
            Mode::Split => "icons/columns-2.svg",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Mode::Code => "Code",
            Mode::Preview => "Preview",
            Mode::Split => "Code and Preview",
        }
    }
}

impl SvgViewer {
    pub(super) fn render_mode_switcher(&self, cx: &Context<Self>) -> Div {
        let theme = cx.theme();
        let buttons = Mode::ALL.map(|mode| {
            Button::new(mode.id())
                .icon(Icon::empty().path(mode.icon()))
                .tooltip(mode.label())
                .selected(self.mode == mode)
        });
        div()
            .absolute()
            .top_2()
            .when(self.mode == Mode::Code, |switcher| {
                switcher.right(MINIMAP_CLEARANCE)
            })
            .when(self.mode != Mode::Code, |switcher| switcher.right_4())
            .p_0p5()
            .rounded(theme.radius)
            .border_1()
            .border_color(theme.border)
            .bg(theme.popover)
            .shadow_sm()
            .occlude()
            .child(
                ButtonGroup::new("svg-mode")
                    .ghost()
                    .xsmall()
                    .children(buttons)
                    .on_click(cx.listener(|viewer, selected: &Vec<usize>, window, cx| {
                        let mode = selected.first().and_then(|index| Mode::ALL.get(*index));
                        if let Some(mode) = mode {
                            viewer.set_mode(*mode, window, cx);
                        }
                    })),
            )
    }
}
