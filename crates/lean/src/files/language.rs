use std::path::Path;

#[derive(Clone, Copy)]
pub(crate) struct Language {
    id: &'static str,
    display_name: &'static str,
}

impl Language {
    pub(crate) const PLAIN: Language = Language {
        id: "text",
        display_name: "Plain Text",
    };

    pub(crate) fn id(self) -> &'static str {
        self.id
    }

    pub(crate) fn display_name(self) -> &'static str {
        self.display_name
    }

    pub(crate) fn for_path(path: &Path) -> Language {
        let file_name = path
            .file_name()
            .map(|name| name.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        let extension = path
            .extension()
            .map(|extension| extension.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        let (id, display_name) = match (file_name.as_str(), extension.as_str()) {
            ("makefile" | "gnumakefile", _) | (_, "mk") => ("make", "Makefile"),
            ("cmakelists.txt", _) | (_, "cmake") => ("cmake", "CMake"),
            (_, "rs") => ("rust", "Rust"),
            (_, "ts" | "mts" | "cts") => ("typescript", "TypeScript"),
            (_, "tsx") => ("tsx", "TypeScript JSX"),
            (_, "js" | "mjs" | "cjs" | "jsx") => ("javascript", "JavaScript"),
            (_, "json" | "jsonc" | "lock") if file_name != "cargo.lock" => ("json", "JSON"),
            (_, "toml") | ("cargo.lock", _) => ("toml", "TOML"),
            (_, "yaml" | "yml") => ("yaml", "YAML"),
            (_, "md" | "mdx" | "markdown") => ("markdown", "Markdown"),
            (_, "py" | "pyi") => ("python", "Python"),
            (_, "go") => ("go", "Go"),
            (_, "c" | "h") => ("c", "C"),
            (_, "cpp" | "cc" | "cxx" | "hpp" | "hh" | "hxx") => ("cpp", "C++"),
            (_, "cs") => ("csharp", "C#"),
            (_, "java") => ("java", "Java"),
            (_, "kt" | "kts") => ("kotlin", "Kotlin"),
            (_, "swift") => ("swift", "Swift"),
            (_, "zig") => ("zig", "Zig"),
            (_, "lua") => ("lua", "Lua"),
            (_, "rb") => ("ruby", "Ruby"),
            (_, "php") => ("php", "PHP"),
            (_, "scala") => ("scala", "Scala"),
            (_, "ex" | "exs") => ("elixir", "Elixir"),
            (_, "sh" | "bash" | "zsh") => ("bash", "Shell Script"),
            (_, "html" | "htm") => ("html", "HTML"),
            (_, "css" | "scss") => ("css", "CSS"),
            (_, "svelte") => ("svelte", "Svelte"),
            (_, "astro") => ("astro", "Astro"),
            (_, "sql") => ("sql", "SQL"),
            (_, "graphql" | "gql") => ("graphql", "GraphQL"),
            (_, "proto") => ("proto", "Protocol Buffers"),
            (_, "diff" | "patch") => ("diff", "Diff"),
            _ => return Language::PLAIN,
        };
        Language { id, display_name }
    }
}
