use gpui_kit::{Context, Subscription};

use super::{Toolchains, ToolchainsEvent};
use crate::toolchain::settings::{
    ACTIVATE_IN_TERMINAL, ACTIVATION_MODE, AUTO_REFRESH, DEFAULT_INTERPRETER, PYENV_ROOT,
    SEARCH_PATHS, VENV_SEARCH_DEPTH,
};

pub(super) fn observe_settings(cx: &mut Context<Toolchains>) -> Vec<Subscription> {
    vec![
        DEFAULT_INTERPRETER.observe(cx, |toolchains, default, cx| {
            toolchains.set_default_interpreter(default, cx)
        }),
        ACTIVATE_IN_TERMINAL.observe(cx, |_, _, cx| {
            cx.emit(ToolchainsEvent::TerminalActivationChanged)
        }),
        ACTIVATION_MODE.observe(cx, |_, _, cx| {
            cx.emit(ToolchainsEvent::TerminalActivationChanged)
        }),
        VENV_SEARCH_DEPTH.observe(cx, |toolchains, _, cx| toolchains.refresh(cx)),
        SEARCH_PATHS.observe(cx, |toolchains, _, cx| toolchains.refresh(cx)),
        PYENV_ROOT.observe(cx, |toolchains, _, cx| toolchains.refresh(cx)),
        AUTO_REFRESH.observe(cx, |toolchains, enabled, cx| {
            toolchains.set_auto_refresh(enabled, cx)
        }),
    ]
}
