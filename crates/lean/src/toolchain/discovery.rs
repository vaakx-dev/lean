mod options;

use std::collections::HashSet;
use std::path::Path;

use super::{Toolchain, python};

pub(super) use options::DiscoveryOptions;

const PROVIDERS: &[fn(&Path, &DiscoveryOptions) -> Vec<Toolchain>] = &[python::discover];

pub(super) fn discover(project_root: &Path, options: &DiscoveryOptions) -> Vec<Toolchain> {
    let mut seen = HashSet::new();
    PROVIDERS
        .iter()
        .flat_map(|provider| provider(project_root, options))
        .filter(|toolchain| seen.insert(toolchain.prefix.clone()))
        .collect()
}
