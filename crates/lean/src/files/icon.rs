use std::borrow::Cow;
use std::path::Path;

use gpui_kit::component::Icon;
use gpui_kit::{Styled, rgb};

const SAND: u32 = 0xd6aa72;
const WHEAT: u32 = 0xd9c48e;
const INDIGO: u32 = 0xa3a8e8;
const GREEN: u32 = 0x8fc79a;
const CORAL: u32 = 0xe08a7a;
const TEAL: u32 = 0x7fb8b0;
const MAUVE: u32 = 0xb89cc4;
const NEUTRAL: u32 = 0x8a8a90;
const DIM: u32 = 0x6a6a70;

pub(crate) fn file_icon(name: &str) -> Icon {
    let lowercase = lowercase_name(name);
    let extension = Path::new(&*lowercase)
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or("");
    let (path, hex) = match (&*lowercase, extension) {
        (".gitignore" | ".gitattributes" | ".gitmodules", _) => ("icons/git-branch.svg", DIM),
        ("cargo.lock" | "bun.lock" | "package-lock.json" | "yarn.lock" | "pnpm-lock.yaml", _) => {
            ("icons/lock.svg", DIM)
        }
        (_, "md" | "mdx") => ("icons/file-text.svg", INDIGO),
        (_, "ipynb") => ("icons/notebook-text.svg", SAND),
        (_, "json" | "jsonc") => ("icons/braces.svg", WHEAT),
        (_, "toml" | "yaml" | "yml" | "ini" | "conf") => ("icons/settings-2.svg", NEUTRAL),
        (_, "rs") => ("icons/file-code.svg", SAND),
        (_, "ts" | "tsx" | "mts" | "cts") => ("icons/file-code.svg", INDIGO),
        (_, "js" | "jsx" | "mjs" | "cjs") => ("icons/file-code.svg", WHEAT),
        (_, "py") => ("icons/file-code.svg", INDIGO),
        (_, "go") => ("icons/file-code.svg", TEAL),
        (_, "c" | "h" | "cpp" | "hpp" | "cc" | "zig") => ("icons/file-code.svg", MAUVE),
        (_, "html" | "css" | "scss" | "svelte" | "vue") => ("icons/file-code.svg", CORAL),
        (_, "sh" | "bash" | "zsh" | "fish") => ("icons/square-terminal.svg", GREEN),
        (_, "png" | "jpg" | "jpeg" | "gif" | "svg" | "webp" | "ico") => {
            ("icons/file-image.svg", MAUVE)
        }
        _ => ("icons/file.svg", NEUTRAL),
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
