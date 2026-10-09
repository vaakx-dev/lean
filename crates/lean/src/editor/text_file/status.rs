use gpui_kit::component::ActiveTheme as _;
use gpui_kit::*;

use super::TextFile;
use crate::toolchain::toolchain_picker;
use crate::workspace::status_item;

impl TextFile {
    pub(super) fn render_status_items(&self, cx: &App) -> Vec<AnyElement> {
        let theme = cx.theme();
        let cursor = format!(
            "Ln {}, Col {}",
            self.cursor.line + 1,
            self.cursor.character + 1
        );
        let mut items = vec![
            status_item("cursor", theme)
                .child(cursor)
                .into_any_element(),
            status_item("indent", theme)
                .child("Spaces: 4")
                .into_any_element(),
            status_item("encoding", theme)
                .child("UTF-8")
                .into_any_element(),
            status_item("line-ending", theme)
                .child(self.line_ending.label())
                .into_any_element(),
            status_item("language", theme)
                .child(self.language.display_name())
                .into_any_element(),
        ];
        items.extend(
            toolchain_picker(&self.toolchains, self.language.id(), cx)
                .map(IntoElement::into_any_element),
        );
        items
    }
}
