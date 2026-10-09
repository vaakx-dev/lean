use std::path::PathBuf;

use gpui_kit::*;

use super::EditorArea;
use super::open_file::OpenFile;
use crate::plugins::AnyFileView;

impl EditorArea {
    pub(crate) fn open_view(
        &mut self,
        key: PathBuf,
        title: SharedString,
        icon: SharedString,
        view: AnyFileView,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(index) = self.index_of_view(&key) {
            self.activate(index, window, cx);
            return;
        }
        let file = OpenFile::detached(key, title, icon, view, cx);
        let index = self.insert(file, cx);
        self.activate(index, window, cx);
    }
}
