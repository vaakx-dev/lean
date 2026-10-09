use std::collections::HashMap;

use gpui_kit::Context;
use gpui_kit::component::ActiveTheme as _;

use super::GitProject;
use crate::plugins::FileDecoration;

const SOURCE: &str = "git";

impl GitProject {
    pub(super) fn publish_decorations(&self, cx: &mut Context<Self>) {
        let theme = cx.theme();
        let mut decorations = HashMap::new();
        if let Some(state) = &self.state {
            for folder in &state.changed_folders {
                decorations.insert(folder.clone(), FileDecoration::Dot(theme.muted_foreground));
            }
            for (path, status) in &state.statuses {
                let decoration = FileDecoration::Letter {
                    letter: status.letter(),
                    color: status.color(theme),
                };
                decorations.insert(path.clone(), decoration);
            }
        }
        self.decorations
            .update(cx, |files, cx| files.set(SOURCE, decorations, cx));
    }
}
