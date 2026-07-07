# DeflateAwning Image Measurement Tool

A small native Rust desktop app for reverse-engineering dimensions from a
photo or scan. Load an image, calibrate it against **one known length**, then
click any two points on the image to read off their real-world distance.

Built with [`egui`/`eframe`](https://github.com/emilk/egui) (immediate-mode
GUI) + the [`image`](https://crates.io/crates/image) crate. Single native
binary, no browser/Electron/network required.

## Workflow

1. **Load Image** (top bar) — opens a native file picker (PNG/JPG/BMP/GIF/TIFF/WebP).
2. Switch mode to **Calibrate**, then click two points on the image that span
   a length you already know (e.g. the two ends of a ruler visible in the
   photo, or a datasheet dimension you trust). Enter that **Real length** and
   **Unit** (e.g. `30`, `mm`) in the side panel — the scale (units-per-pixel)
   updates live.
3. Switch mode to **Measure**, then click any two other points on the image.
   The tool computes pixel distance and converts it to real-world units using
   the active calibration, and adds it to the **Measurements** list.
4. Repeat step 3 for as many dimensions as you need. Rename each measurement
   inline, delete individual rows, or re-calibrate and hit **Recompute
   Measurements** to re-derive every recorded pixel distance from a new
   calibration (e.g. if you find a more accurate reference length).
5. **Export CSV** to get a spreadsheet of every measurement (pixel + real
   distance + unit), or **Save Project** to a JSON file you can **Load
   Project** later to resume exactly where you left off (image path,
   calibration, and all measurements).

## Controls

- **Left click** — place a point (Calibrate/Measure modes only).
- **Scroll wheel** — zoom in/out, centered on the cursor.
- **Right-drag or middle-drag** — pan, in any mode.
- **Left-drag** — pan, only while in *Idle / Pan* mode (so it doesn't
  conflict with point placement).
- **Reset View** — re-fits the image and resets zoom/pan.

All click coordinates are stored in image-pixel space, so calibration and
measurements stay correct regardless of zoom/pan/window size.

## Building from source

Requires a Rust toolchain (edition 2021, tested with rustc 1.75+).
Install one from https://rustup.rs if you don't already have `cargo`.

```bash
cargo build --release
./target/release/da-image-measurement-tool
```

### Linux system dependencies

The GUI (via `winit`) and native file dialogs (via `rfd`) link against your
system's windowing/GTK libraries. On Debian/Ubuntu:

```bash
sudo apt-get install -y libgtk-3-dev libxcb1-dev libxkbcommon-dev \
  libx11-dev libxrandr-dev libxi-dev libgl1-mesa-dev libxcursor-dev \
  libxinerama-dev
```

Windows and macOS need no extra system packages beyond a normal Rust install.

### Note on dependency pins

A handful of transitive dependencies (`indexmap`, `wayland-protocols`,
`smithay-clipboard`, `idna_adapter`, `home`) are pinned to slightly older
versions in `Cargo.toml`. This is only necessary if you're building with an
older `cargo` (< 1.85) that doesn't support the 2024 Rust edition yet — those
newer transitive versions declare `edition = "2024"` in their manifest and
fail to parse otherwise. If your toolchain is current, feel free to remove
the pins with `cargo update` and it'll resolve to the latest compatible
versions on its own.

## Project layout

- `src/main.rs` — app entry point / window setup.
- `src/model.rs` — `Calibration`, `Measurement`, and mode/state enums.
- `src/geometry.rs` — pixel-distance math and screen ↔ image coordinate transforms (handles pan/zoom).
- `src/app.rs` — the `eframe::App` implementation: UI panels, canvas drawing, and click handling.
- `src/export.rs` — CSV export and JSON project save/load.

## Limitations / ideas for extension

- Assumes the image is captured with no significant lens distortion or
  perspective skew along the measured axis (i.e. it's a 2D scale factor, not
  a full perspective/homography correction). For photos taken at an angle,
  keep the calibration line and measurement lines roughly parallel/coplanar
  with the reference for best accuracy.
- Only one calibration is active at a time. Multiple simultaneous scale
  factors (e.g. separate X/Y scales for a non-square-pixel scan) aren't
  supported, but `Calibration` in `model.rs` is a small, well-isolated type if
  you want to extend it (e.g. add a second calibration for a perpendicular
  axis).
