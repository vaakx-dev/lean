# Lean

Lean is a small, fast native code editor built with [GPUI](https://www.gpui.rs) and
[GPUI Component](https://github.com/longbridge/gpui-component).

It focuses on a few things: editing code, a clear git view, a built-in terminal and a minimap.

## Build and run

Install Rust with rustup (the toolchain is pinned in `rust-toolchain.toml`) and the GPUI
Linux dependencies (Vulkan, Wayland/X11, xkbcommon, fontconfig).

```sh
cargo run -- /path/to/project
```

## License

MIT.
