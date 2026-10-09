use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use notify::RecursiveMode;

use crate::watcher::MetadataWatch;

const RELEVANT_FILES: [&str; 4] = ["HEAD", "index", "packed-refs", "MERGE_HEAD"];

pub(crate) fn metadata_watch(git_dirs: &[PathBuf]) -> MetadataWatch {
    let mut targets = Vec::new();
    for dir in git_dirs {
        targets.push((dir.clone(), RecursiveMode::NonRecursive));
        let refs = dir.join("refs");
        if refs.is_dir() {
            targets.push((refs, RecursiveMode::Recursive));
        }
        let info = dir.join("info");
        if info.is_dir() {
            targets.push((info, RecursiveMode::NonRecursive));
        }
    }
    MetadataWatch {
        dirs: git_dirs.to_vec(),
        targets,
        is_relevant,
    }
}

fn is_relevant(relative: &Path) -> bool {
    if relative.extension() == Some(OsStr::new("lock")) {
        return false;
    }
    let mut components = relative.components().map(|component| component.as_os_str());
    match (components.next(), components.next()) {
        (Some(first), None) => RELEVANT_FILES.iter().any(|name| first == *name),
        (Some(first), Some(_)) => first == "refs",
        _ => false,
    }
}
