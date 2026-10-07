pub(crate) mod actions;
mod cli;
mod keymap;
mod menus;
mod window;

use std::path::PathBuf;

use crate::assets::Assets;
use crate::theme;
use actions::Quit;

pub(crate) use cli::project_root_from_args;

pub(crate) fn run(project_root: PathBuf) {
    gpui_kit::application().with_assets(Assets).run(move |cx| {
        gpui_kit::init(cx);
        theme::init(cx);
        keymap::bind_keys(cx);
        menus::set_menus(cx);

        cx.on_action(|_: &Quit, cx| cx.quit());
        cx.on_window_closed(|cx, _| {
            if cx.windows().is_empty() {
                cx.quit();
            }
        })
        .detach();

        if let Err(error) = window::open_main_window(project_root, cx) {
            log::error!("failed to open the main window: {error:#}");
            cx.quit();
        }
        cx.activate(true);
    });
}
