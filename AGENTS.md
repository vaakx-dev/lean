# Lean agent notes

- Keep code clean, focused and readable: small files, clear names.
- Don't write code comments, doc files (READMEs, design docs, notes) or tests.
- Organise by feature, not by file type.
- Split a feature into subfolders as it grows.
- Keep the root small.
- Rust + GPUI editor.
- Use `~/.cargo/bin` (rustup) for `cargo`; the toolchain is pinned in `rust-toolchain.toml`.
- Never run `cargo build`, `cargo check`, `cargo clippy` or `cargo run`; the user builds and reports errors. `cargo fmt`, `cargo fetch` and `cargo add` are fine.
- Verify APIs by reading dependency sources in `~/.cargo/registry/src/` instead of compiling.
- UI components come from `gpui-kit` (`gpui_kit::component::*`); prefer them over custom widgets.
