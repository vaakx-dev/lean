# Lean

Lean is a small, fast native code editor built with [GPUI](https://www.gpui.rs) and
[GPUI Component](https://github.com/longbridge/gpui-component).

It focuses on a few things: editing code, a clear git view, a built-in terminal and a minimap.

## Download

Builds for Linux, Windows and macOS are on the [releases page](https://github.com/vaakx-dev/lean/releases).
The `nightly` release is rebuilt from `main` each day it changes.

## Build and run

Install Rust with rustup (the toolchain is pinned in `rust-toolchain.toml`) and the GPUI
Linux dependencies (Vulkan, Wayland/X11, xkbcommon, fontconfig).

```sh
git clone https://github.com/vaakx-dev/lean.git
cd lean
cargo run -- /path/to/project
```

## License

Lean is MIT licensed, see [LICENSE](LICENSE). The patched GPUI crates in `vendor/` keep their
upstream Apache-2.0 licenses.
