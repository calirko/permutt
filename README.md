# permutt

A simple GTK4 desktop app for converting images between formats on Linux.

Drag and drop (or browse) images on the left, pick your output format on the right, and convert. Files land in `~/Downloads/permutt/` by default, or a folder of your choice.

**Supported formats:** PNG, JPG, WEBP, GIF, BMP, TIFF, ICO (input) → PNG, JPG, WEBP, GIF (output)

## Build

Requires Rust and GTK4.

```bash
# Arch
sudo pacman -S gtk4

cargo build --release
cargo run
```

## License

Apache 2.0 — see [LICENSE](LICENSE).
