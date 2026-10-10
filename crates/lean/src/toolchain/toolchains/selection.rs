use std::collections::HashMap;
use std::path::PathBuf;

use gpui_kit::Context;

use super::{Toolchains, ToolchainsEvent};
use crate::toolchain::settings::DefaultInterpreter;
use crate::toolchain::store;

impl Toolchains {
    pub(crate) fn select(
        &mut self,
        language: &str,
        prefix: Option<PathBuf>,
        cx: &mut Context<Self>,
    ) {
        store::save(&self.project_root, language, prefix.clone(), cx);
        self.track_selection(true, cx, |toolchains| {
            toolchains.choices.insert(language.to_owned(), prefix);
        });
        cx.notify();
    }

    pub(super) fn set_default_interpreter(
        &mut self,
        default: DefaultInterpreter,
        cx: &mut Context<Self>,
    ) {
        self.track_selection(true, cx, |toolchains| {
            toolchains.default_interpreter = default;
        });
        cx.notify();
    }

    pub(super) fn track_selection(
        &mut self,
        announce: bool,
        cx: &mut Context<Self>,
        change: impl FnOnce(&mut Self),
    ) {
        let before: HashMap<&'static str, Option<PathBuf>> = self
            .languages()
            .into_iter()
            .map(|language| (language, self.active_prefix(language)))
            .collect();
        change(self);
        if !announce {
            return;
        }
        for language in self.languages() {
            let previous = before.get(language).cloned().flatten();
            if self.active_prefix(language) != previous {
                cx.emit(ToolchainsEvent::Selected(self.active_label(language)));
            }
        }
    }
}
