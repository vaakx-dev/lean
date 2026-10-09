pub(crate) mod actions;
mod cli;
mod keymap;
mod menus;
mod quit;
mod window;

use std::path::PathBuf;

use crate::assets::Assets;
use crate::{plugins, recent_projects, settings, theme};
use actions::Quit;

pub(crate) use cli::project_root_from_args;
pub(crate) use menus::set_menus;

pub(crate) fn run(project_root: PathBuf) {
    gpui_kit::application().with_assets(Assets).run(move |cx| {
        gpui_kit::init(cx);
        theme::init(cx);
        keymap::bind_keys(cx);
        settings::init(cx);
        plugins::init(cx);
        recent_projects::init(project_root.clone(), cx);
        menus::set_menus(cx);

        cx.on_action(|_: &Quit, cx| quit::quit(cx));
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
