#![cfg_attr(windows, windows_subsystem = "windows")]

mod app;
mod assets;
mod editor;
mod explorer;
mod files;
mod git;
mod placeholder;
mod project;
mod source_control;
mod terminal;
mod theme;
mod workspace;

fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let project_root = app::project_root_from_args()?;
    app::run(project_root);
    Ok(())
}
