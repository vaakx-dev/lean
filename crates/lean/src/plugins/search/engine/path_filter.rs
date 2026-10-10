use std::path::Path;

use gpui_kit::SharedString;
use ignore::overrides::{Override, OverrideBuilder};

pub(super) fn build(
    root: &Path,
    include: &str,
    exclude: &str,
    always_exclude: &[String],
) -> Result<Override, SharedString> {
    let mut builder = OverrideBuilder::new(root);
    for glob in globs(include) {
        for pattern in expand(glob) {
            builder.add(&pattern).map_err(invalid_pattern)?;
        }
    }
    let excluded = globs(exclude).chain(always_exclude.iter().filter_map(|glob| clean(glob)));
    for glob in excluded {
        for pattern in expand(glob) {
            builder
                .add(&format!("!{pattern}"))
                .map_err(invalid_pattern)?;
        }
    }
    builder.build().map_err(invalid_pattern)
}

fn globs(list: &str) -> impl Iterator<Item = &str> {
    list.split(',').filter_map(clean)
}

fn clean(glob: &str) -> Option<&str> {
    let glob = glob.trim().trim_end_matches('/');
    (!glob.is_empty()).then_some(glob)
}

fn expand(glob: &str) -> [String; 2] {
    [glob.to_string(), format!("{glob}/**")]
}

fn invalid_pattern(error: ignore::Error) -> SharedString {
    format!("Invalid file pattern: {error}").into()
}
