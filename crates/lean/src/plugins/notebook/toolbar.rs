use gpui_kit::component::button::Button;
use gpui_kit::component::tooltip::Tooltip;
use gpui_kit::component::{ActiveTheme as _, Disableable as _, h_flex};
use gpui_kit::*;

use super::Notebook;
use super::document::CellKind;
use super::status::KernelStatus;
use crate::toolchain::toolchain_picker;
use crate::ui::icon_button;

const TOOLBAR_HEIGHT: Pixels = px(34.);
const PYTHON: &str = "python";

impl Notebook {
    pub(super) fn render_toolbar(&self, cx: &Context<Self>) -> Div {
        let theme = cx.theme();
        let status = self.kernel_status();
        let busy = matches!(status, KernelStatus::Busy);
        h_flex()
            .h(TOOLBAR_HEIGHT)
            .flex_shrink_0()
            .px_2()
            .gap_0p5()
            .border_b_1()
            .border_color(theme.border)
            .bg(theme.background)
            .child(
                icon_button("add-code", "icons/plus.svg", "Insert Code Cell")
                    .label("Code")
                    .on_click(cx.listener(|notebook, _, window, cx| {
                        notebook.insert_below_selected(CellKind::Code, window, cx)
                    })),
            )
            .child(
                icon_button("add-markdown", "icons/plus.svg", "Insert Markdown Cell")
                    .label("Markdown")
                    .on_click(cx.listener(|notebook, _, window, cx| {
                        notebook.insert_below_selected(CellKind::Markdown, window, cx)
                    })),
            )
            .child(div().w(px(1.)).h(px(16.)).mx_1().bg(theme.border))
            .child(
                icon_button("run-all", "icons/fast-forward.svg", "Run All Cells")
                    .label("Run All")
                    .on_click(cx.listener(|notebook, _, _, cx| notebook.run_all(cx))),
            )
            .child(
                icon_button("restart", "icons/rotate-ccw.svg", "Restart Kernel")
                    .label("Restart")
                    .on_click(cx.listener(|notebook, _, _, cx| notebook.restart_kernel(cx))),
            )
            .child(
                icon_button("interrupt", "icons/square.svg", "Interrupt Kernel")
                    .label("Interrupt")
                    .disabled(!busy)
                    .on_click(cx.listener(|notebook, _, _, cx| notebook.interrupt_kernel(cx))),
            )
            .child(
                icon_button("clear-all", "icons/eraser.svg", "Clear All Outputs")
                    .label("Clear Outputs")
                    .on_click(cx.listener(|notebook, _, _, cx| notebook.clear_all_outputs(cx))),
            )
            .child(div().flex_1())
            .children(self.render_restart_notice(cx))
            .child(kernel_indicator(&status, cx))
            .children(toolchain_picker(&self.toolchains, PYTHON, cx))
    }

    fn render_restart_notice(&self, cx: &Context<Self>) -> Option<Button> {
        let outdated = self
            .session
            .as_ref()
            .is_some_and(|session| session.outdated);
        outdated.then(|| {
            icon_button(
                "restart-to-apply",
                "icons/triangle-alert.svg",
                "The Python environment or interpreter changed since this kernel started",
            )
            .label("Restart kernel to apply")
            .text_color(cx.theme().warning)
            .on_click(cx.listener(|notebook, _, _, cx| notebook.restart_kernel(cx)))
        })
    }
}

fn kernel_indicator(status: &KernelStatus, cx: &App) -> Stateful<Div> {
    let theme = cx.theme();
    let color = match status {
        KernelStatus::Stopped => theme.muted_foreground,
        KernelStatus::Starting | KernelStatus::Busy => theme.warning,
        KernelStatus::Idle => theme.success,
        KernelStatus::Failed(_) => theme.danger,
    };
    let tooltip = match status {
        KernelStatus::Failed(error) => error.clone(),
        _ => status.label(),
    };
    h_flex()
        .id("kernel-status")
        .gap_1p5()
        .px_1p5()
        .text_xs()
        .text_color(theme.muted_foreground)
        .child(div().size(px(7.)).rounded_full().bg(color))
        .child(status.label())
        .tooltip(move |window, cx| Tooltip::new(tooltip.clone()).build(window, cx))
}
