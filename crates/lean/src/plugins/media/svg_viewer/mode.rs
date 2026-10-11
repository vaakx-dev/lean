use gpui_kit::component::button::{Button, ButtonGroup, ButtonVariants as _};
use gpui_kit::component::{ActiveTheme as _, Icon, Selectable as _, Sizable as _};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use super::SvgViewer;
use crate::plugins::media::settings::SvgMode;

const MINIMAP_CLEARANCE: Pixels = px(116.);

impl SvgMode {
    const ALL: [SvgMode; 3] = [SvgMode::Code, SvgMode::Preview, SvgMode::Split];

    pub(super) fn shows_code(self) -> bool {
        matches!(self, SvgMode::Code | SvgMode::Split)
    }

    pub(super) fn shows_preview(self) -> bool {
        matches!(self, SvgMode::Preview | SvgMode::Split)
    }

    fn id(self) -> &'static str {
        match self {
            SvgMode::Code => "svg-mode-code",
            SvgMode::Preview => "svg-mode-preview",
            SvgMode::Split => "svg-mode-split",
        }
    }

    fn icon(self) -> &'static str {
        match self {
            SvgMode::Code => "icons/code.svg",
            SvgMode::Preview => "icons/eye.svg",
            SvgMode::Split => "icons/columns-2.svg",
        }
    }

    fn label(self) -> &'static str {
        match self {
            SvgMode::Code => "Code",
            SvgMode::Preview => "Preview",
            SvgMode::Split => "Code and Preview",
        }
    }
}

impl SvgViewer {
    pub(super) fn render_mode_switcher(&self, cx: &Context<Self>) -> Div {
        let theme = cx.theme();
        let buttons = SvgMode::ALL.map(|mode| {
            Button::new(mode.id())
                .icon(Icon::empty().path(mode.icon()))
                .tooltip(mode.label())
                .selected(self.mode == mode)
        });
        div()
            .absolute()
            .top_2()
            .when(self.mode == SvgMode::Code, |switcher| {
                switcher.right(MINIMAP_CLEARANCE)
            })
            .when(self.mode != SvgMode::Code, |switcher| switcher.right_4())
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
                        let mode = selected.first().and_then(|index| SvgMode::ALL.get(*index));
                        if let Some(mode) = mode {
                            viewer.set_mode(*mode, window, cx);
                        }
                    })),
            )
    }
}
