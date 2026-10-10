use std::collections::HashMap;

use gpui_kit::Context;
use gpui_kit::component::ActiveTheme as _;

use super::GitProject;
use crate::plugins::FileDecoration;
use crate::plugins::git::settings::{
    EXPLORER_DECORATIONS, EXPLORER_FOLDER_DOTS, ExplorerDecorations,
};

const SOURCE: &str = "git";

impl GitProject {
    pub(super) fn publish_decorations(&self, cx: &mut Context<Self>) {
        let style = EXPLORER_DECORATIONS.get(cx);
        let folder_dots = EXPLORER_FOLDER_DOTS.get(cx);
        let theme = cx.theme();
        let mut decorations = HashMap::new();
        if let Some(state) = &self.state {
            if folder_dots {
                for folder in &state.changed_folders {
                    decorations.insert(folder.clone(), FileDecoration::Dot(theme.muted_foreground));
                }
            }
            for (path, status) in &state.statuses {
                let decoration = match style {
                    ExplorerDecorations::Letter => FileDecoration::Letter {
                        letter: status.letter(),
                        color: status.color(theme),
                    },
                    ExplorerDecorations::Dot => FileDecoration::Dot(status.color(theme)),
                    ExplorerDecorations::Off => continue,
                };
                decorations.insert(path.clone(), decoration);
            }
        }
        self.decorations
            .update(cx, |files, cx| files.set(SOURCE, decorations, cx));
    }
}
