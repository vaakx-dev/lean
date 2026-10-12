mod diff;
mod git_project;
mod repo;
pub(crate) mod settings;
mod source_control;
mod status;

use gpui_kit::App;

use super::{Plugin, Registry};
use git_project::GitProject;

pub(super) const PLUGIN: Plugin = Plugin {
    id: "git",
    name: "Git",
    description: "Source control panel, change markers and diffs.",
    activate,
};

fn activate(registry: &mut Registry, cx: &mut App) {
    registry.register_service(GitProject::start);
    registry.register_panel(source_control::PANEL);
    registry.register_status_item(status::STATUS_ITEM);
    source_control::bind_keys(cx);
}
