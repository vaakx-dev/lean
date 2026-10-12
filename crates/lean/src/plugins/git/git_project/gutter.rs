mod compute;

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::Duration;

use gpui_kit::component::ActiveTheme as _;
use gpui_kit::{App, AppContext as _, Context, EntityId, Task};

use super::GitProject;
use crate::plugins::GutterMarker;
use crate::plugins::git::diff::{GutterBase, Hunk, gutter_markers};
use crate::plugins::git::repo::FileStatus;
use crate::plugins::git::settings::{GUTTER_BASE, GUTTER_ENABLED};
use compute::{GutterBatch, GutterJob, GutterResult};

const DEBOUNCE: Duration = Duration::from_millis(200);

#[derive(Default)]
pub(super) struct GutterSync {
    pending: HashSet<PathBuf>,
    all: bool,
    applied: HashMap<PathBuf, (EntityId, Vec<Hunk>)>,
    bases: HashMap<PathBuf, GutterBase>,
    generation: u64,
    task: Option<Task<()>>,
}

impl GitProject {
    pub(super) fn repaint_gutter(&mut self, cx: &mut Context<Self>) {
        self.gutter.applied.clear();
        self.schedule_gutter(cx);
    }

    pub(super) fn gutter_base_changed(&mut self, cx: &mut Context<Self>) {
        self.forget_gutter_bases();
        self.repaint_gutter(cx);
    }

    pub(super) fn refresh_gutter(&mut self, cx: &mut Context<Self>) {
        self.forget_gutter_bases();
        self.gutter.all = true;
        self.schedule_gutter(cx);
    }

    pub(super) fn refresh_gutter_for(&mut self, path: &Path, cx: &mut Context<Self>) {
        if GUTTER_ENABLED.get(cx) && self.is_tracked(path) {
            self.gutter.pending.insert(path.to_path_buf());
            self.schedule_gutter(cx);
        }
    }

    pub(super) fn refresh_gutter_where(
        &mut self,
        touched: impl Fn(&Path) -> bool,
        cx: &mut Context<Self>,
    ) {
        if !GUTTER_ENABLED.get(cx) {
            return;
        }
        let paths: Vec<PathBuf> = self
            .gutter
            .applied
            .keys()
            .filter(|path| touched(path.as_path()) && self.is_tracked(path.as_path()))
            .cloned()
            .collect();
        if !paths.is_empty() {
            self.gutter.pending.extend(paths);
            self.schedule_gutter(cx);
        }
    }

    pub(super) fn schedule_gutter(&mut self, cx: &mut Context<Self>) {
        self.gutter.task = Some(cx.spawn(async move |git, cx| {
            cx.background_executor().timer(DEBOUNCE).await;
            let Ok(batch) = git.update(cx, |git, cx| git.gutter_batch(cx)) else {
                return;
            };
            let generation = batch.generation;
            let results = cx.background_spawn(async move { batch.compute() }).await;
            git.update(cx, |git, cx| git.apply_gutter(generation, results, cx))
                .ok();
        }));
    }

    fn forget_gutter_bases(&mut self) {
        self.gutter.bases.clear();
        self.gutter.generation += 1;
    }

    fn gutter_batch(&self, cx: &App) -> GutterBatch {
        let enabled = GUTTER_ENABLED.get(cx);
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
                let text = (enabled && self.is_tracked(path))
                    .then(|| view.text(cx))
                    .flatten();
                Some(GutterJob {
                    path: path.to_path_buf(),
                    view: id,
                    base: text
                        .as_ref()
                        .and_then(|_| self.gutter.bases.get(path).cloned()),
                    text,
                })
            })
            .collect();
        GutterBatch {
            workdir: self.state.as_ref().map(|state| state.workdir.clone()),
            base: GUTTER_BASE.get(cx),
            generation: self.gutter.generation,
            jobs,
        }
    }

    fn is_tracked(&self, path: &Path) -> bool {
        self.state.as_ref().is_some_and(|state| {
            path.starts_with(&state.workdir)
                && state.unstaged.get(path) != Some(&FileStatus::Untracked)
        })
    }

    fn apply_gutter(
        &mut self,
        generation: u64,
        results: Vec<GutterResult>,
        cx: &mut Context<Self>,
    ) {
        self.gutter.pending.clear();
        self.gutter.all = false;
        let current = generation == self.gutter.generation;
        let mut changed = Vec::new();
        for GutterResult {
            path,
            view,
            hunks,
            loaded_base,
        } in results
        {
            if current && let Some(base) = loaded_base {
                self.gutter.bases.insert(path.clone(), base);
            }
            let unchanged = self
                .gutter
                .applied
                .get(&path)
                .is_some_and(|(applied_view, applied)| *applied_view == view && *applied == hunks);
            if !unchanged {
                changed.push((path, view, hunks));
            }
        }
        let theme = cx.theme();
        let mut markers: HashMap<EntityId, (&Path, Vec<GutterMarker>)> = changed
            .iter()
            .map(|(path, view, hunks)| (*view, (path.as_path(), gutter_markers(hunks, theme))))
            .collect();
        let open_paths = self.editor_area.update(cx, move |area, cx| {
            for (path, view) in area.open_file_views() {
                if let Some((_, view_markers)) = markers
                    .remove(&view.view().entity_id())
                    .filter(|(changed, _)| *changed == path)
                {
                    view.set_gutter_markers(view_markers, cx);
                }
            }
            area.open_paths()
        });
        for (path, view, hunks) in changed {
            self.gutter.applied.insert(path, (view, hunks));
        }
        self.gutter
            .applied
            .retain(|path, _| open_paths.contains(path));
        self.gutter
            .bases
            .retain(|path, _| open_paths.contains(path));
    }
}
