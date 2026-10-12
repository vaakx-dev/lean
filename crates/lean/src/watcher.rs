mod classify;
mod filter;
mod folder_watch;
mod health;
mod metadata;
mod recovery;
mod shared_paths;
mod skipped;
mod tree;
mod walk;

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::{Duration, Instant};

use futures::StreamExt as _;
use futures::channel::mpsc::{self, UnboundedReceiver, UnboundedSender};
use gpui_kit::{AppContext as _, AsyncApp, Context, Task, WeakEntity};

use crate::project::{FilesChanged, IgnoreCache, Project, ProjectEvent};
use crate::settings::Setting;
use filter::{EventFilter, RawEvent};
use health::Health;
use metadata::SharedMetadata;
use recovery::Recovery;
use shared_paths::SharedPaths;
use skipped::SkippedFolders;
use tree::WatchedTree;

pub(crate) use classify::DiskEntry;
pub(crate) use filter::is_noise;
pub(crate) use folder_watch::{FolderWatch, reload_on_change};
pub(crate) use metadata::MetadataWatch;

pub(crate) const DEBOUNCE_MS: Setting<u64> = Setting::new("watcher.debounce_ms", 150);

const SHORTEST_DEBOUNCE_MS: u64 = 50;
const LONGEST_DEBOUNCE_MS: u64 = 2000;
const WATCH_EACH_FOLDER: bool = cfg!(any(target_os = "linux", target_os = "android"));

#[derive(Default)]
pub(crate) struct FsChanges {
    pub(crate) paths: HashSet<PathBuf>,
    pub(crate) structural: HashMap<PathBuf, DiskEntry>,
    pub(crate) ignored: HashSet<PathBuf>,
    pub(crate) renames: Vec<(PathBuf, PathBuf)>,
    pub(crate) rescan: bool,
    pub(crate) metadata: bool,
    pub(crate) observed_at: Option<Instant>,
}

impl FsChanges {
    pub(crate) fn rescan() -> Self {
        Self {
            rescan: true,
            metadata: true,
            ..Self::default()
        }
    }

    pub(crate) fn reconcile(observed_at: Instant) -> Self {
        Self {
            rescan: true,
            observed_at: Some(observed_at),
            ..Self::default()
        }
    }

    pub(crate) fn unwatched(folders: HashSet<PathBuf>) -> Self {
        Self {
            ignored: folders,
            ..Self::default()
        }
    }

    pub(crate) fn touches_files(&self) -> bool {
        self.rescan || !self.paths.is_empty() || !self.renames.is_empty()
    }

    pub(crate) fn is_empty(&self) -> bool {
        !self.metadata && !self.touches_files() && self.ignored.is_empty()
    }
}

pub(crate) struct FsWatcher {
    metadata: SharedMetadata,
    health: Health,
    skipped: SkippedFolders,
    watched: SharedPaths,
    open_files: SharedPaths,
    requests: UnboundedSender<RawEvent>,
    _task: Task<()>,
}

impl FsWatcher {
    pub(crate) fn start(root: PathBuf, ignore: IgnoreCache, cx: &mut Context<Project>) -> Self {
        let started = Instant::now();
        let (sender, receiver) = mpsc::unbounded();
        let health = Health::default();
        let open_files = SharedPaths::default();
        let metadata = SharedMetadata::default();
        let filter = EventFilter {
            ignore: ignore.clone(),
            metadata: metadata.clone(),
            skipped: SkippedFolders::new(&root),
            open_files: open_files.clone(),
        };
        let skipped = filter.skipped.clone();
        let tree = WatchedTree::new(
            root,
            ignore,
            skipped.clone(),
            health.clone(),
            open_files.clone(),
            metadata.clone(),
        );
        let watched = tree.watched().clone();
        let event_sender = sender.clone();
        let requests = sender.clone();
        let watch_health = health.clone();
        let task = cx.spawn(async move |project, cx| {
            let tree = cx
                .background_spawn(async move { tree.start(filter, event_sender) })
                .await;
            let reconcile = project.update(cx, |_, cx| {
                cx.emit(FilesChanged(Rc::new(FsChanges::reconcile(started))))
            });
            if reconcile.is_err() {
                return;
            }
            forward_changes(receiver, tree, watch_health, requests, project, cx).await;
        });
        Self {
            metadata,
            health,
            skipped,
            watched,
            open_files,
            requests: sender,
            _task: task,
        }
    }

    pub(crate) fn set_metadata(&self, watch: MetadataWatch) {
        if self.metadata.replace(watch) {
            self.requests.unbounded_send(RawEvent::empty()).ok();
        }
    }

    pub(crate) fn is_healthy(&self) -> bool {
        self.health.is_ok()
    }

    pub(crate) fn watches(&self, folder: &Path) -> bool {
        if !self.health.is_ok() {
            return false;
        }
        if WATCH_EACH_FOLDER {
            return self.watched.contains(folder);
        }
        let is_inside_root = self
            .watched
            .read()
            .iter()
            .any(|root| folder.starts_with(root));
        is_inside_root && !self.skipped.contains_self_or_parent_of(folder)
    }

    pub(crate) fn unwatched_folders(&self) -> HashSet<PathBuf> {
        self.skipped.snapshot()
    }

    pub(crate) fn set_open_files(&self, files: HashSet<PathBuf>) {
        if *self.open_files.read() == files {
            return;
        }
        self.open_files.replace(files);
        self.requests.unbounded_send(RawEvent::empty()).ok();
    }

    pub(crate) fn request_rescan(&self) {
        self.requests.unbounded_send(RawEvent::rescan()).ok();
    }
}

async fn forward_changes(
    mut receiver: UnboundedReceiver<RawEvent>,
    mut tree: WatchedTree,
    health: Health,
    requests: UnboundedSender<RawEvent>,
    project: WeakEntity<Project>,
    cx: &mut AsyncApp,
) {
    let mut recovery = Recovery::default();
    loop {
        if let Some(message) = recovery.check(&health, &requests, cx) {
            let reported = project.update(cx, |_, cx| {
                cx.emit(ProjectEvent::WatchFailed(message.into()))
            });
            if reported.is_err() {
                return;
            }
        }
        let Some(first) = receiver.next().await else {
            return;
        };
        let debounce_ms = cx.update(|cx| {
            DEBOUNCE_MS
                .get(cx)
                .clamp(SHORTEST_DEBOUNCE_MS, LONGEST_DEBOUNCE_MS)
        });
        cx.background_executor()
            .timer(Duration::from_millis(debounce_ms))
            .await;
        let mut events = vec![first];
        while let Ok(event) = receiver.try_recv() {
            events.push(event);
        }
        if events.iter().any(|event| event.retry) {
            recovery.retry_started();
        }
        let (returned, changes) = cx
            .background_spawn(async move {
                let changes = tree.process(events);
                (tree, changes)
            })
            .await;
        tree = returned;
        if let Some(changes) = changes
            && project
                .update(cx, |project, cx| project.apply_fs_changes(changes, cx))
                .is_err()
        {
            return;
        }
    }
}
