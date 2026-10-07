use gpui_kit::{ClipboardItem, Context, Window};

use super::{CopyPath, CopyRelativePath};
use crate::explorer::Explorer;

impl Explorer {
    pub(super) fn copy_path(&mut self, _: &CopyPath, _: &mut Window, cx: &mut Context<Self>) {
        let path = self.target_path();
        cx.write_to_clipboard(ClipboardItem::new_string(
            path.to_string_lossy().into_owned(),
        ));
    }

    pub(super) fn copy_relative_path(
        &mut self,
        _: &CopyRelativePath,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let path = self.target_path();
        let Ok(relative) = path.strip_prefix(&self.root) else {
            return;
        };
        if relative.as_os_str().is_empty() {
            return;
        }
        cx.write_to_clipboard(ClipboardItem::new_string(
            relative.to_string_lossy().into_owned(),
        ));
    }
}
