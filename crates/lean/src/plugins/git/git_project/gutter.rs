use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::Duration;

use gpui_kit::component::{ActiveTheme as _, Rope};
use gpui_kit::{App, AppContext as _, Context, EntityId, Task};

use super::GitProject;
use crate::plugins::GutterMarker;
use crate::plugins::git::diff::{DiffBase, Hunk, gutter_markers, hunks_for};
use crate::plugins::git::repo::FileStatus;
use crate::plugins::git::settings::{GUTTER_BASE, GUTTER_ENABLED, GutterBase};

const DEBOUNCE: Duration = Duration::from_millis(200);

pub(super) enum GutterScope {
    File(PathBuf),
    Opened,
    All,
    Repaint,
}

#[derive(Default)]
pub(super) struct GutterSync {
    pending: HashSet<PathBuf>,
    all: bool,
    applied: HashMap<PathBuf, (EntityId, Vec<Hunk>)>,
    task: Option<Task<()>>,
}

struct GutterJob {
    path: PathBuf,
    view: EntityId,
    text: Option<Rope>,
}

struct GutterResult {
    path: PathBuf,
    view: EntityId,
    hunks: Vec<Hunk>,
}

impl GitProject {
    pub(super) fn schedule_gutter(&mut self, scope: GutterScope, cx: &mut Context<Self>) {
        match scope {
            GutterScope::File(path) => {
                self.gutter.pending.insert(path);
            }
            GutterScope::Opened => {}
            GutterScope::All => self.gutter.all = true,
            GutterScope::Repaint => {
                self.gutter.applied.clear();
                self.gutter.all = true;
            }
        }
        self.gutter.task = Some(cx.spawn(async move |git, cx| {
            cx.background_executor().timer(DEBOUNCE).await;
            let Ok((workdir, base, jobs)) = git.update(cx, |git, cx| git.gutter_jobs(cx)) else {
                return;
            };
            let results = cx
                .background_spawn(async move { compute_hunks(workdir.as_deref(), base, jobs) })
                .await;
            if let Err(error) = git.update(cx, |git, cx| git.apply_gutter(results, cx)) {
                log::debug!("git state dropped before the change markers were ready: {error:#}");
            }
        }));
    }

    fn gutter_jobs(&self, cx: &App) -> (Option<PathBuf>, DiffBase, Vec<GutterJob>) {
        let workdir = self.state.as_ref().map(|state| state.workdir.clone());
        let enabled = GUTTER_ENABLED.get(cx);
        let base = diff_base(GUTTER_BASE.get(cx));
        let jobs = self
            .editor_area
            .read(cx)
            .open_file_views()
            .filter_map(|(path, view)| {
                let id = view.view().entity_id();
                let applied = self.gutter.applied.get(path).map(|(view, _)| *view);
                let wanted =
                    self.gutter.all || self.gutter.pending.contains(path) || applied != Some(id);
                if !wanted {
                    return None;
                }
                let text = view.text(cx)?;
                Some(GutterJob {
                    path: path.to_path_buf(),
                    view: id,
                    text: (enabled && self.is_tracked(path)).then_some(text),
                })
            })
            .collect();
        (workdir, base, jobs)
    }

    fn is_tracked(&self, path: &Path) -> bool {
        self.state.as_ref().is_some_and(|state| {
            path.starts_with(&state.workdir)
                && state.unstaged.get(path) != Some(&FileStatus::Untracked)
        })
    }

    fn apply_gutter(&mut self, results: Vec<GutterResult>, cx: &mut Context<Self>) {
        self.gutter.pending.clear();
        self.gutter.all = false;
        let changed: Vec<GutterResult> = results
            .into_iter()
            .filter(|result| {
                self.gutter.applied.get(&result.path) != Some(&(result.view, result.hunks.clone()))
            })
            .collect();
        let markers: Vec<(&Path, EntityId, Vec<GutterMarker>)> = changed
            .iter()
            .map(|result| {
                let markers = gutter_markers(&result.hunks, cx.theme());
                (result.path.as_path(), result.view, markers)
            })
            .collect();
        let open_paths = self.editor_area.update(cx, |area, cx| {
            for (path, view) in area.open_file_views() {
                let id = view.view().entity_id();
                if let Some((_, _, markers)) = markers
                    .iter()
                    .find(|(changed, view, _)| *changed == path && *view == id)
                {
                    view.set_gutter_markers(markers.clone(), cx);
                }
            }
            area.open_paths()
        });
        for result in changed {
            self.gutter
                .applied
                .insert(result.path, (result.view, result.hunks));
        }
        self.gutter
            .applied
            .retain(|path, _| open_paths.contains(path));
    }
}

fn diff_base(base: GutterBase) -> DiffBase {
    match base {
        GutterBase::Index => DiffBase::Index,
        GutterBase::Head => DiffBase::Staged,
    }
}

fn compute_hunks(
    workdir: Option<&Path>,
    base: DiffBase,
    jobs: Vec<GutterJob>,
) -> Vec<GutterResult> {
    jobs.into_iter()
        .map(|job| {
            let hunks = match (workdir, job.text) {
                (Some(workdir), Some(text)) => {
                    hunks_for(workdir, &job.path, &text.to_string(), base).unwrap_or_else(|error| {
                        log::debug!("failed to diff {}: {error:#}", job.path.display());
                        Vec::new()
                    })
                }
                _ => Vec::new(),
            };
            GutterResult {
                path: job.path,
                view: job.view,
                hunks,
            }
        })
        .collect()
}
