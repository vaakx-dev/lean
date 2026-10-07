use gpui_kit::{Context, Window};

use super::RevealInFileManager;
use crate::explorer::Explorer;

impl Explorer {
    pub(super) fn reveal_in_file_manager(
        &mut self,
        _: &RevealInFileManager,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        cx.reveal_path(&self.target_path());
    }
}
