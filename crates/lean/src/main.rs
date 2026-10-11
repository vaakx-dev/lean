#![cfg_attr(windows, windows_subsystem = "windows")]

mod app;
mod assets;
mod editor;
mod explorer;
mod files;
mod plugins;
mod project;
mod recent_projects;
mod settings;
mod storage;
mod terminal;
mod theme;
mod toolchain;
mod ui;
mod watcher;
mod workspace;

fn main() -> anyhow::Result<()> {
    app::init_logging();
    let launch = app::launch_target_from_args()?;
    app::run(launch);
    Ok(())
}
