mod classify;
mod coverage;
mod disk_entry;
mod filter;
mod health;
mod metadata;
mod shared_paths;
mod skipped;
mod tree;
mod walk;

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use futures::StreamExt as _;
use futures::channel::mpsc::{self, UnboundedReceiver, UnboundedSender};
use gpui_kit::{AppContext as _, AsyncApp, Context, Task, WeakEntity};

use crate::project::{IgnoreCache, Project};
use coverage::covers;
use filter::{EventFilter, RawEvent};
use health::Health;
use metadata::SharedMetadata;
use shared_paths::SharedPaths;
use skipped::SkippedFolders;
use tree::WatchedTree;

pub(crate) use disk_entry::DiskEntry;
pub(crate) use metadata::MetadataWatch;

const DEBOUNCE: Duration = Duration::from_millis(150);
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

    pub(crate) fn reconcile() -> Self {
        Self {
            rescan: true,
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
    pub(crate) fn start(
        root: PathBuf,
        metadata: MetadataWatch,
        ignore: IgnoreCache,
        open_files: HashSet<PathBuf>,
        cx: &mut Context<Project>,
    ) -> Self {
        let (sender, receiver) = mpsc::unbounded();
        let health = Health::default();
        let open_files = SharedPaths::new(open_files);
        let metadata = SharedMetadata::new(metadata);
        let tree = WatchedTree::new(
            root,
            ignore,
            health.clone(),
            open_files.clone(),
            metadata.clone(),
        );
        let skipped = tree.skipped().clone();
        let watched = tree.watched().clone();
        let filter = EventFilter {
            metadata: metadata.clone(),
            skipped: skipped.clone(),
            open_files: open_files.clone(),
        };
        let event_sender = sender.clone();
        let task = cx.spawn(async move |project, cx| {
            let tree = cx
                .background_spawn(async move { tree.start(filter, event_sender) })
                .await;
            let reconcile =
                project.update(cx, |project, cx| project.reconcile_after_watch_start(cx));
            if reconcile.is_err() {
                return;
            }
            forward_changes(receiver, tree, project, cx).await;
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
        self.metadata.replace(watch);
        self.requests.unbounded_send(RawEvent::empty()).ok();
    }

    pub(crate) fn is_healthy(&self) -> bool {
        self.health.is_ok()
    }

    pub(crate) fn watches(&self, folder: &Path) -> bool {
        self.health.is_ok() && covers(&self.watched, &self.skipped, folder)
    }

    pub(crate) fn unwatched_folders(&self) -> HashSet<PathBuf> {
        self.skipped.snapshot()
    }

    pub(crate) fn set_open_files(&self, files: HashSet<PathBuf>) {
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
    project: WeakEntity<Project>,
    cx: &mut AsyncApp,
) {
    while let Some(first) = receiver.next().await {
        cx.background_executor().timer(DEBOUNCE).await;
        let mut events = vec![first];
        while let Ok(event) = receiver.try_recv() {
            events.push(event);
        }
        let (returned, changes) = cx
            .background_spawn(async move {
                let changes = tree.classify(events);
                (tree, changes)
            })
            .await;
        tree = returned;
        let Some(changes) = changes else {
            continue;
        };
        let update = project.update(cx, |project, cx| project.apply_fs_changes(changes, cx));
        if update.is_err() {
            break;
        }
    }
}
