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
mod terminal;
mod theme;
mod toolchain;
mod watcher;
mod workspace;

fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();
    let project_root = app::project_root_from_args()?;
    app::run(project_root);
    Ok(())
}
