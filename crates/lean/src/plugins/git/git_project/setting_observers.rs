use gpui_kit::{Context, Subscription};

use super::GitProject;
use super::gutter::GutterScope;
use crate::plugins::git::settings::{
    EXPLORER_DECORATIONS, EXPLORER_FOLDER_DOTS, GUTTER_BASE, GUTTER_ENABLED, HISTORY_LIMIT,
    INCLUDE_UNTRACKED, SHOW_HISTORY, history_page_size,
};

impl GitProject {
    pub(super) fn observe_settings(cx: &mut Context<Self>) -> Vec<Subscription> {
        vec![
            INCLUDE_UNTRACKED.observe(cx, |git, _, cx| git.refresh(cx)),
            SHOW_HISTORY.observe(cx, |git, _, cx| git.refresh(cx)),
            HISTORY_LIMIT.observe(cx, |git, _, cx| {
                git.history_limit = history_page_size(cx);
                git.refresh(cx);
            }),
            GUTTER_ENABLED.observe(cx, |git, _, cx| {
                git.schedule_gutter(GutterScope::Repaint, cx)
            }),
            GUTTER_BASE.observe(cx, |git, _, cx| {
                git.schedule_gutter(GutterScope::Repaint, cx)
            }),
            EXPLORER_DECORATIONS.observe(cx, |git, _, cx| git.publish_decorations(cx)),
            EXPLORER_FOLDER_DOTS.observe(cx, |git, _, cx| git.publish_decorations(cx)),
        ]
    }
}
