use std::fs::OpenOptions;
use std::io;
use std::path::{Path, PathBuf};

use gpui_kit::component::WindowExt as _;
use gpui_kit::{AppContext as _, Context, Window};

use crate::explorer::{Explorer, ExplorerEvent};
use crate::files::file_name;

impl Explorer {
    pub(super) fn create_entry(
        &mut self,
        path: PathBuf,
        is_dir: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let task = cx.background_spawn({
            let path = path.clone();
            async move { create_on_disk(&path, is_dir) }
        });
        cx.spawn_in(window, async move |explorer, cx| {
            let result = task.await;
            let update = explorer.update_in(cx, |explorer, window, cx| match result {
                Ok(()) => explorer.finish_create(path, is_dir, cx),
                Err(error) => {
                    log::warn!("failed to create {}: {error}", path.display());
                    window.push_notification(
                        format!("Can't create \"{}\": {error}", file_name(&path)),
                        cx,
                    );
                }
            });
            if let Err(error) = update {
                log::debug!("explorer dropped while creating an entry: {error:#}");
            }
        })
        .detach();
    }

    fn finish_create(&mut self, path: PathBuf, is_dir: bool, cx: &mut Context<Self>) {
        let Some(folder) = path.parent().map(Path::to_path_buf) else {
            return;
        };
        self.finish_file_operation(folder, Some(path.clone()), cx);
        if !is_dir {
            cx.emit(ExplorerEvent::OpenFile(path));
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
