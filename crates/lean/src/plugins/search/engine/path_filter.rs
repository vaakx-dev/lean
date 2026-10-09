use std::path::Path;

use gpui_kit::SharedString;
use ignore::overrides::{Override, OverrideBuilder};

pub(super) fn build(root: &Path, include: &str, exclude: &str) -> Result<Override, SharedString> {
    let mut builder = OverrideBuilder::new(root);
    for glob in globs(include) {
        for pattern in expand(glob) {
            builder.add(&pattern).map_err(invalid_pattern)?;
        }
    }
    for glob in globs(exclude) {
        for pattern in expand(glob) {
            builder
                .add(&format!("!{pattern}"))
                .map_err(invalid_pattern)?;
        }
    }
    builder.build().map_err(invalid_pattern)
}

fn globs(list: &str) -> impl Iterator<Item = &str> {
    list.split(',')
        .map(|glob| glob.trim().trim_end_matches('/'))
        .filter(|glob| !glob.is_empty())
}

fn expand(glob: &str) -> [String; 2] {
    [glob.to_string(), format!("{glob}/**")]
}

fn invalid_pattern(error: ignore::Error) -> SharedString {
    format!("Invalid file pattern: {error}").into()
}
