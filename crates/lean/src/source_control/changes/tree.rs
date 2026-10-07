use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};

use super::{ChangeGroup, ChangedFile};
use crate::source_control::rows::{FileRow, FolderRow, Row};

#[derive(Default)]
struct FolderNode<'a> {
    folders: BTreeMap<String, FolderNode<'a>>,
    files: Vec<&'a ChangedFile>,
}

pub(in crate::source_control) fn push_tree(
    group: ChangeGroup,
    files: &[ChangedFile],
    collapsed: &HashSet<(ChangeGroup, PathBuf)>,
    rows: &mut Vec<Row>,
) {
    let mut root = FolderNode::default();
    for file in files {
        root.insert(file);
    }
    let mut walk = TreeWalk {
        group,
        collapsed,
        rows,
    };
    walk.push_children(&root, Path::new(""), 0);
}

impl<'a> FolderNode<'a> {
    fn insert(&mut self, file: &'a ChangedFile) {
        let mut node = self;
        if let Some(parent) = file.relative_path.parent() {
            for component in parent.components() {
                let name = component.as_os_str().to_string_lossy().into_owned();
                node = node.folders.entry(name).or_default();
            }
        }
        node.files.push(file);
    }

    fn compact(&self, name: &str, prefix: &Path) -> (String, PathBuf, &FolderNode<'a>) {
        let mut label = name.to_string();
        let mut relative_path = prefix.join(name);
        let mut node = self;
        while node.files.is_empty() && node.folders.len() == 1 {
            let Some((child_name, child)) = node.folders.iter().next() else {
                break;
            };
            label = format!("{label}/{child_name}");
            relative_path.push(child_name);
            node = child;
        }
        (label, relative_path, node)
    }
}

struct TreeWalk<'a> {
    group: ChangeGroup,
    collapsed: &'a HashSet<(ChangeGroup, PathBuf)>,
    rows: &'a mut Vec<Row>,
}

impl TreeWalk<'_> {
    fn push_children(&mut self, node: &FolderNode, prefix: &Path, depth: usize) {
        for (name, child) in &node.folders {
            let (label, relative_path, folder) = child.compact(name, prefix);
            let expanded = !self
                .collapsed
                .contains(&(self.group, relative_path.clone()));
            self.rows.push(Row::Folder(FolderRow {
                group: self.group,
                relative_path: relative_path.clone(),
                label: label.into(),
                depth,
                expanded,
            }));
            if expanded {
                self.push_children(folder, &relative_path, depth + 1);
            }
        }
        for file in &node.files {
            self.rows.push(Row::File(FileRow {
                group: self.group,
                file: (*file).clone(),
                depth,
                show_folder: false,
            }));
        }
    }
}
