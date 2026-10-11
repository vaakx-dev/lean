use std::fs::OpenOptions;
use std::io;
use std::path::{Path, PathBuf};

use gpui_kit::{Context, Window};

use super::report_failure;
use crate::explorer::Explorer;
use crate::plugins::PanelEvent;

impl Explorer {
    pub(super) fn create_entry(
        &mut self,
        path: PathBuf,
        is_dir: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let target = path.clone();
        self.run_disk_op(
            move || create_on_disk(&target, is_dir),
            window,
            cx,
            move |explorer, result, window, cx| match result {
                Ok(()) => explorer.finish_create(path, is_dir, window, cx),
                Err(error) => report_failure("create", &path, error, window, cx),
            },
        );
    }

    fn finish_create(
        &mut self,
        path: PathBuf,
        is_dir: bool,
        window: &Window,
        cx: &mut Context<Self>,
    ) {
        let Some(folder) = path.parent().map(Path::to_path_buf) else {
            return;
        };
        self.finish_file_operation(folder, Some(path.clone()), window, cx);
        if !is_dir {
            cx.emit(PanelEvent::OpenFile(path));
        }
    }
}

fn create_on_disk(path: &Path, is_dir: bool) -> io::Result<()> {
    if is_dir {
        std::fs::create_dir(path)
    } else {
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .map(drop)
    }
}
