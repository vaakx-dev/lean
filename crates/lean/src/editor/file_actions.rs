use std::path::PathBuf;

use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::{Icon, Sizable as _, h_flex};
use gpui_kit::*;

use super::EditorArea;
use crate::plugins::{FileAction, file_actions};

impl EditorArea {
    pub(super) fn render_file_actions(&self, cx: &App) -> Option<Div> {
        let file = self.active_file().filter(|file| file.is_file())?;
        let path = file.path.clone();
        let actions = file_actions(&path, cx);
        if actions.is_empty() {
            return None;
        }
        Some(
            h_flex().flex_shrink_0().h_full().px_1().gap_0p5().children(
                actions
                    .into_iter()
                    .map(|action| action_button(action, path.clone())),
            ),
        )
    }
}

fn action_button(action: FileAction, path: PathBuf) -> Button {
    Button::new(action.id)
        .ghost()
        .xsmall()
        .icon(Icon::empty().path(action.icon))
        .tooltip(action.label)
        .on_click(move |_, window, cx| (action.run)(path.as_path(), window, cx))
}
