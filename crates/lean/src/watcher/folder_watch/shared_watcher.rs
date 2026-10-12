use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard, PoisonError};

use futures::channel::mpsc::UnboundedSender;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher as _};

pub(super) type Relevance = Box<dyn Fn(&Event) -> bool + Send>;

struct Route {
    folders: Vec<PathBuf>,
    is_relevant: Relevance,
    sender: UnboundedSender<()>,
}

impl Route {
    fn covers(&self, event: &Event) -> bool {
        event.paths.iter().any(|path| {
            self.folders
                .iter()
                .any(|folder| path == folder || path.parent() == Some(folder.as_path()))
        })
    }
}

struct SharedWatcher {
    watcher: RecommendedWatcher,
    watch_counts: HashMap<PathBuf, usize>,
}

static WATCHER: Mutex<Option<SharedWatcher>> = Mutex::new(None);
static ROUTES: Mutex<Vec<(u64, Route)>> = Mutex::new(Vec::new());
static NEXT_ID: AtomicU64 = AtomicU64::new(0);

pub(super) fn subscribe(
    is_relevant: Relevance,
    sender: UnboundedSender<()>,
) -> notify::Result<u64> {
    let mut shared = lock(&WATCHER);
    if shared.is_none() {
        let watcher = RecommendedWatcher::new(dispatch, Config::default())?;
        *shared = Some(SharedWatcher {
            watcher,
            watch_counts: HashMap::new(),
        });
    }
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let route = Route {
        folders: Vec::new(),
        is_relevant,
        sender,
    };
    lock(&ROUTES).push((id, route));
    Ok(id)
}

pub(super) fn watch(id: u64, folder: &Path) -> notify::Result<()> {
    {
        let mut shared = lock(&WATCHER);
        let Some(shared) = shared.as_mut() else {
            return Err(notify::Error::generic(
                "the shared folder watcher is not running",
            ));
        };
        let count = shared.watch_counts.get(folder).copied().unwrap_or(0);
        if count == 0 {
            shared.watcher.watch(folder, RecursiveMode::NonRecursive)?;
        }
        shared.watch_counts.insert(folder.to_path_buf(), count + 1);
    }
    let canonical = dunce::canonicalize(folder)
        .ok()
        .filter(|canonical| canonical != folder);
    if let Some((_, route)) = lock(&ROUTES)
        .iter_mut()
        .find(|(route_id, _)| *route_id == id)
    {
        route.folders.push(folder.to_path_buf());
        route.folders.extend(canonical);
    }
    Ok(())
}

pub(super) fn unsubscribe(id: u64, folders: &[PathBuf]) {
    lock(&ROUTES).retain(|(route_id, _)| *route_id != id);
    let mut shared = lock(&WATCHER);
    let Some(shared) = shared.as_mut() else {
        return;
    };
    for folder in folders {
        let Some(count) = shared.watch_counts.get_mut(folder) else {
            continue;
        };
        *count -= 1;
        if *count > 0 {
            continue;
        }
        shared.watch_counts.remove(folder);
        if let Err(error) = shared.watcher.unwatch(folder) {
            log::debug!("failed to unwatch {}: {error}", folder.display());
        }
    }
}

fn dispatch(event: notify::Result<Event>) {
    let Ok(event) = event else {
        return;
    };
    for (_, route) in lock(&ROUTES).iter() {
        if route.covers(&event) && (route.is_relevant)(&event) {
            route.sender.unbounded_send(()).ok();
        }
    }
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}
