use std::collections::HashSet;
use std::path::Path;

use super::{Toolchain, python};

const PROVIDERS: &[fn(&Path) -> Vec<Toolchain>] = &[python::discover];

pub(super) fn discover(project_root: &Path) -> Vec<Toolchain> {
    let mut seen = HashSet::new();
    PROVIDERS
        .iter()
        .flat_map(|provider| provider(project_root))
        .filter(|toolchain| seen.insert(toolchain.prefix.clone()))
        .collect()
}
