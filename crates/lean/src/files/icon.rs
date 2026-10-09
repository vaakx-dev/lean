mod tint;

use std::borrow::Cow;
use std::path::Path;

use gpui_kit::component::Icon;
use gpui_kit::{Styled, rgb};

pub(crate) fn file_icon(name: &str) -> Icon {
    let lowercase = lowercase_name(name);
    let extension = Path::new(&*lowercase)
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or("");
    let (path, hex) = match (&*lowercase, extension) {
        (".gitignore" | ".gitattributes" | ".gitmodules", _) => ("icons/git-branch.svg", tint::DIM),
        ("cargo.lock" | "bun.lock" | "package-lock.json" | "yarn.lock" | "pnpm-lock.yaml", _) => {
            ("icons/lock.svg", tint::DIM)
        }
        (_, "md" | "mdx") => ("icons/file-text.svg", tint::INDIGO),
        (_, "ipynb") => ("icons/notebook-text.svg", tint::SAND),
        (_, "json" | "jsonc") => ("icons/braces.svg", tint::WHEAT),
        (_, "toml" | "yaml" | "yml" | "ini" | "conf") => ("icons/settings-2.svg", tint::NEUTRAL),
        (_, "rs") => ("icons/file-code.svg", tint::SAND),
        (_, "ts" | "tsx" | "mts" | "cts") => ("icons/file-code.svg", tint::INDIGO),
        (_, "js" | "jsx" | "mjs" | "cjs") => ("icons/file-code.svg", tint::WHEAT),
        (_, "py") => ("icons/file-code.svg", tint::INDIGO),
        (_, "go") => ("icons/file-code.svg", tint::TEAL),
        (_, "c" | "h" | "cpp" | "hpp" | "cc" | "zig") => ("icons/file-code.svg", tint::MAUVE),
        (_, "html" | "css" | "scss" | "svelte" | "vue") => ("icons/file-code.svg", tint::CORAL),
        (_, "sh" | "bash" | "zsh" | "fish") => ("icons/square-terminal.svg", tint::GREEN),
        (_, "png" | "jpg" | "jpeg" | "gif" | "svg" | "webp" | "ico") => {
            ("icons/file-image.svg", tint::MAUVE)
        }
        _ => ("icons/file.svg", tint::NEUTRAL),
    };
    Icon::empty().path(path).size_4().text_color(rgb(hex))
}

fn lowercase_name(name: &str) -> Cow<'_, str> {
    if name.chars().any(char::is_uppercase) {
        Cow::Owned(name.to_lowercase())
    } else {
        Cow::Borrowed(name)
    }
}
