pub(crate) mod actions;
mod cli;
pub(crate) mod keybindings;
pub(crate) mod keymap;
mod logging;
mod menus;
mod quit;
pub(crate) mod settings;
mod window;

use crate::assets::Assets;
use crate::{plugins, recent_projects, theme, updates};
use actions::Quit;

pub(crate) use cli::{LaunchTarget, launch_target_from_args};
pub(crate) use logging::init_logging;
pub(crate) use menus::set_menus;
pub(crate) use quit::{restart, restart_as};

pub(crate) fn run(launch: LaunchTarget) {
    gpui_kit::application().with_assets(Assets).run(move |cx| {
        gpui_kit::init(cx);
        crate::settings::init(cx);
        logging::follow_log_level(cx);
        crate::files::settings::init(cx);
        theme::init(cx);
        keymap::bind_keys(cx);
        plugins::init(cx);
        recent_projects::init(cx);
        updates::init(cx);
        menus::set_menus(cx);

        cx.on_action(|_: &Quit, cx| quit::quit(cx));
        cx.on_window_closed(|cx, _| {
            if cx.windows().is_empty() {
                cx.quit();
            }
        })
        .detach();

        let project_root = launch.project_root(cx);
        if let Err(error) = window::open_main_window(project_root, launch.file.clone(), cx) {
            log::error!("failed to open the main window: {error:#}");
            cx.quit();
        }
        cx.activate(true);
    });
}
