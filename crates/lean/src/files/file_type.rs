use std::borrow::Cow;
use std::path::Path;

use super::language::Language;

const SAND: u32 = 0xd6aa72;
const WHEAT: u32 = 0xd9c48e;
const INDIGO: u32 = 0xa3a8e8;
const GREEN: u32 = 0x8fc79a;
const CORAL: u32 = 0xe08a7a;
const TEAL: u32 = 0x7fb8b0;
const MAUVE: u32 = 0xb89cc4;
const NEUTRAL: u32 = 0x8a8a90;
const DIM: u32 = 0x6a6a70;

const FILE: FileIcon = ("icons/file.svg", NEUTRAL);
const GIT: FileIcon = ("icons/git-branch.svg", DIM);
const LOCK: FileIcon = ("icons/lock.svg", DIM);
const TEXT: FileIcon = ("icons/file-text.svg", INDIGO);
const NOTEBOOK: FileIcon = ("icons/notebook-text.svg", SAND);
const BRACES: FileIcon = ("icons/braces.svg", WHEAT);
const CONFIG: FileIcon = ("icons/settings-2.svg", NEUTRAL);
const TERMINAL: FileIcon = ("icons/square-terminal.svg", GREEN);
const IMAGE: FileIcon = ("icons/file-image.svg", MAUVE);

pub(super) type FileIcon = (&'static str, u32);

pub(super) struct FileType {
    pub(super) language: Language,
    pub(super) icon: FileIcon,
}

pub(super) fn file_type(name: &str) -> FileType {
    let name = lowercase(name);
    let extension = Path::new(&*name)
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or("");
    let (language, icon) = match (&*name, extension) {
        (".gitignore" | ".gitattributes" | ".gitmodules", _) => (Language::PLAIN, GIT),
        ("cargo.lock", _) => (Language::new("toml", "TOML"), LOCK),
        ("bun.lock" | "package-lock.json" | "yarn.lock", _) => {
            (Language::new("json", "JSON"), LOCK)
        }
        ("pnpm-lock.yaml", _) => (Language::new("yaml", "YAML"), LOCK),
        ("makefile" | "gnumakefile", _) | (_, "mk") => (Language::new("make", "Makefile"), FILE),
        ("cmakelists.txt", _) | (_, "cmake") => (Language::new("cmake", "CMake"), FILE),
        (_, "rs") => (Language::new("rust", "Rust"), code(SAND)),
        (_, "ts" | "mts" | "cts") => (Language::new("typescript", "TypeScript"), code(INDIGO)),
        (_, "tsx") => (Language::new("tsx", "TypeScript JSX"), code(INDIGO)),
        (_, "js" | "mjs" | "cjs" | "jsx") => {
            (Language::new("javascript", "JavaScript"), code(WHEAT))
        }
        (_, "json" | "jsonc") => (Language::new("json", "JSON"), BRACES),
        (_, "lock") => (Language::new("json", "JSON"), FILE),
        (_, "toml") => (Language::new("toml", "TOML"), CONFIG),
        (_, "yaml" | "yml") => (Language::new("yaml", "YAML"), CONFIG),
        (_, "ini" | "conf") => (Language::PLAIN, CONFIG),
        (_, "md" | "mdx" | "markdown") => (Language::new("markdown", "Markdown"), TEXT),
        (_, "ipynb") => (Language::PLAIN, NOTEBOOK),
        (_, "py" | "pyi") => (Language::new("python", "Python"), code(INDIGO)),
        (_, "go") => (Language::new("go", "Go"), code(TEAL)),
        (_, "c" | "h") => (Language::new("c", "C"), code(MAUVE)),
        (_, "cpp" | "cc" | "cxx" | "hpp" | "hh" | "hxx") => {
            (Language::new("cpp", "C++"), code(MAUVE))
        }
        (_, "cs") => (Language::new("text", "C#"), FILE),
        (_, "java") => (Language::new("java", "Java"), FILE),
        (_, "kt" | "kts") => (Language::new("text", "Kotlin"), FILE),
        (_, "swift") => (Language::new("text", "Swift"), FILE),
        (_, "zig") => (Language::new("zig", "Zig"), code(MAUVE)),
        (_, "lua") => (Language::new("lua", "Lua"), FILE),
        (_, "rb") => (Language::new("text", "Ruby"), FILE),
        (_, "php") => (Language::new("text", "PHP"), FILE),
        (_, "scala") => (Language::new("text", "Scala"), FILE),
        (_, "ex" | "exs") => (Language::new("text", "Elixir"), FILE),
        (_, "sh" | "bash" | "zsh") => (Language::new("bash", "Shell Script"), TERMINAL),
        (_, "fish") => (Language::PLAIN, TERMINAL),
        (_, "html" | "htm") => (Language::new("html", "HTML"), code(CORAL)),
        (_, "css" | "scss") => (Language::new("css", "CSS"), code(CORAL)),
        (_, "svelte") => (Language::new("svelte", "Svelte"), code(CORAL)),
        (_, "vue") => (Language::PLAIN, code(CORAL)),
        (_, "astro") => (Language::new("astro", "Astro"), FILE),
        (_, "sql") => (Language::new("text", "SQL"), FILE),
        (_, "graphql" | "gql") => (Language::new("graphql", "GraphQL"), FILE),
        (_, "proto") => (Language::new("proto", "Protocol Buffers"), FILE),
        (_, "diff" | "patch") => (Language::new("diff", "Diff"), FILE),
        (_, "png" | "jpg" | "jpeg" | "gif" | "svg" | "webp" | "ico") => (Language::PLAIN, IMAGE),
        _ => (Language::PLAIN, FILE),
    };
    FileType { language, icon }
}

const fn code(color: u32) -> FileIcon {
    ("icons/file-code.svg", color)
}

fn lowercase(name: &str) -> Cow<'_, str> {
    if name.chars().any(char::is_uppercase) {
        Cow::Owned(name.to_lowercase())
    } else {
        Cow::Borrowed(name)
    }
}
