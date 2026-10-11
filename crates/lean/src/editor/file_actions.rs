use gpui_kit::component::h_flex;
use gpui_kit::*;

use super::EditorArea;
use crate::plugins::file_actions;
use crate::ui::icon_button;

impl EditorArea {
    pub(super) fn render_file_actions(&self, cx: &App) -> Option<Div> {
        let file = self.active_file().filter(|file| file.is_file())?;
        let actions = file_actions(&file.path, cx);
        if actions.is_empty() {
            return None;
        }
        Some(
            h_flex()
                .flex_shrink_0()
                .h_full()
                .px_1()
                .gap_0p5()
                .children(actions.into_iter().map(|action| {
                    let path = file.path.clone();
                    icon_button(action.id, action.icon, action.label)
                        .on_click(move |_, window, cx| (action.run)(&path, window, cx))
                })),
        )
    }
}
