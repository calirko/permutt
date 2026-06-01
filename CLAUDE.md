# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
cargo build          # debug build
cargo build --release
cargo run            # run the GTK4 app (requires a display / WAYLAND_DISPLAY or DISPLAY)
cargo clippy         # lint
cargo check          # fast type-check without producing a binary
```

GTK4 must be present on the system (`pkg-config --modversion gtk4`). On Arch: `sudo pacman -S gtk4`.

## Architecture

Permutt is a GTK4 desktop app for image format conversion. The binary entry point is `src/main.rs`, which wires startup CSS loading and hands off to `app::build_app()`.

**Module responsibilities:**

| Module | Role |
|---|---|
| `app.rs` | Builds and owns the full window. Manages `AppState` (selected files, output dir, chosen format) behind an `Arc<Mutex<AppState>>`. Wires all signals. |
| `output_picker.rs` | `OutputPicker` struct — a radio-button group for selecting the output format. Exposes `selected_format() -> Option<ImageOutputFormat>`. |
| `converter.rs` | Pure conversion logic. `ConvertJob` / `ConvertResult` types, `convert(&job)` function (synchronous, runs on a background thread), and `is_supported_input(&path)` for file-type filtering. |
| `style.rs` | Embeds layout/typography CSS as a `&str` constant and loads it via `gtk::style_context_add_provider_for_display`. No color overrides — uses system theme. |

**UI layout:** The window is a horizontal `Box` — left panel (file input via drag-and-drop or browse dialog) separated from right panel (format picker + output dir + convert button). A `Stack` in the left panel switches between the drop-hint placeholder and the file `ListBox` once files are added.

**Threading model:** Conversion runs on a `std::thread::spawn` thread. Results are sent back to the GTK main thread via an `async_channel::bounded(1)` channel consumed with `glib::MainContext::default().spawn_local(async move { ... })`. GTK widgets captured in the async block must be cloned before the thread spawns.

**Adding a new output format:** Add a variant to `ImageOutputFormat` in `converter.rs` (implement `extension`, `as_image_format`, `Display`). It automatically appears in `OutputPicker` via `ImageOutputFormat::all()`.

**HEIC input** is currently handled by the `image` crate's built-in support only; a `heic` Cargo feature stub exists in `Cargo.toml` for future `libheif-rs` integration.
