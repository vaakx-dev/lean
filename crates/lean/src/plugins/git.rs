mod diff;
mod git_project;
mod repo;
pub(crate) mod settings;
mod source_control;
mod status;

use gpui_kit::App;

use super::{Plugin, PluginContext};
use git_project::GitProject;

pub(super) const PLUGIN: Plugin = Plugin {
    id: "git",
    name: "Git",
    description: "Source control panel, change markers and diffs.",
    activate,
};

fn activate(context: &mut PluginContext, cx: &mut App) {
    context.register_service(GitProject::start);
    context.register_panel(source_control::PANEL);
    context.register_status_item(status::STATUS_ITEM);
    source_control::bind_keys(cx);
}
