# LaserBurn — Project Status & Architecture Context

## Current Status (as of October 25, 2026)
LaserBurn is a high-performance, modern laser CAM/CNC software written in Rust (`eframe`/`egui`).
It is fully functional, all 35 unit tests pass, and it is deployed as a native macOS app bundle (`/Applications/LaserBurn.app`) as well as a CLI binary (`~/.cargo/bin/laser_burn`).

### Key Recent Milestones & Features Implemented:
1. **Adaptive Grid Subdivisions & Sticky Rulers:**
   - Multi-tiered millimeter grid subdivisions (`10mm/50mm`, `5mm/20mm`, `2mm/10mm`, `1mm/5mm`, `0.5mm/2mm`) based on zoom factor.
   - Dynamic viewport-clamped sticky ruler badges along X and Y axes ensuring coordinates remain visible during pan/zoom.
2. **Native Async File Dialogs (`rfd::AsyncFileDialog`):**
   - Completely non-blocking background thread dialog execution using `pollster::block_on` and `std::sync::mpsc::channel`.
   - Resolves macOS `[NSSavePanel runModal]` re-entrancy conflict with the winit runloop (`panic_cannot_unwind`).
   - Supports `Open Project`, `Save Project` (`.lbrn2`), `Export G-Code`, `Export Ruida`, and `Browse Font`.
   - Keyboard shortcuts: `Cmd+O` and `Cmd+S`.
3. **Live Camera Feed & Optical Alignment System:**
   - Fisheye lens barrel/pincushion undistortion (`k1`, `k2`, optical center offset X/Y, scale X/Y, uniform zoom).
   - Alignment guide grid overlay.
   - Bed alignment with independent scale X/Y, offset X/Y (mm), rotation (°), and horizontal/vertical flip.
   - Canvas background overlay: renders directly on the bed grid beneath laser vectors, strictly clipped to `bed_rect`.
   - Hotkeys: `Cmd+K` toggles Camera Feed & Alignment window; `Cmd+Shift+C` toggles canvas background overlay.
   - Persistence: Calibration automatically serialized to `~/.config/laser_burn/camera_calibration.json`.
4. **Transparent Safe Work Envelope:**
   - Removed green fill overlay (`rect_filled`) to keep bed canvas completely clear, preserving only bright green boundary lines and corner markers.
5. **Clean UI & Resizing Stability:**
   - Stripped missing Unicode emojis/glyphs (`[ ]` / `.notdef`) across menus, buttons, and toolbars.
   - 100% English interface across all windows, menus, sliders, toolbars, and status messages.
   - Fixed camera window layout feedback loop: replaced unbounded row allocations with clamped widths and `egui::TopBottomPanel::bottom` + `egui::CentralPanel::default()` + `ScrollArea`, allowing effortless resizing smaller or larger.

---

## Codebase Map & Modules (`src/`)
- `src/main.rs`: Application entry point, CLI argument parsing (`clap`), native macOS menu/window runner (`eframe`).
- `src/lib.rs`: Module exports and integration test hooks.
- `src/gui/mod.rs`: Full `eframe`/`egui` user interface:
  - Canvas rendering, pan/zoom transformation, grid rulers, and adaptive subdivisions.
  - Shape selection, drag-to-move, bounding box handles, rotation, alignment tools.
  - Async file dialog receiver handling (`FileDialogAction`).
  - Cut layers palette (C00–C29) with power, speed, air-assist, passes.
  - Camera window (`Camera Feed & Alignment`), docked bottom bar, optical alignment, and bed canvas overlay.
  - Connection modal, Jogging/Laser Control modal, Settings modal, Text tool dialog.
- `src/camera/mod.rs`:
  - `CameraManager`: Nokhwa-based camera discovery, background thread frame capture, RGBA texture conversion.
  - `CameraCalibration`: Serde persistence for distortion parameters, optical centers, bed offsets, zoom factors.
  - `LensUndistorter`: Bilinear interpolation barrel/pincushion fisheye lens correction.
- `src/device/mod.rs`:
  - Controller abstraction (`Grbl`, `Smoothieware`, `RuidaUdp`, `VirtualDemo`).
  - Serial port and network communication (`serialport`, UDP sockets).
  - Machine envelope enforcement, software limits, emergency stop, laser jog, framing box.
- `src/formats/`:
  - `lbrn2.rs`: Reverse-engineered LightBurn 2.0 XML reader/writer (`roxmltree`), parsing shapes, layer cut settings, text objects.
  - `gcode.rs`: GRBL/Smoothie G-code generator (`M3`/`M4`, `G0`, `G1`, `S` duty cycle, `M8`/`M9`).
  - `ruida.rs`: Direct binary `.rd` generation for Ruida DSP controllers.
  - `svg.rs`: Standard SVG vector import/export.
- `src/optimizer/mod.rs`:
  - Traveling Salesperson (TSP) rapid move minimization, inner-cuts-first topological sorting, cut direction reversal.
- `src/hatch/mod.rs`:
  - Hatch fill engine: zig-zag serpentine scanning, cross-hatching, custom angles and line spacing.
- `src/dither/mod.rs`:
  - 8 photo dithering algorithms (Atkinson, Floyd-Steinberg, Jarvis-Judice-Ninke, Stucki, Bayer 8x8, Halftone, Threshold, Power Modulation).
- `src/simulator/mod.rs`:
  - Job run time estimation considering machine acceleration, maximum velocities, and trapezoidal velocity profiling.
- `src/text/mod.rs`:
  - Hershey single-line stroke font vectorizer and TrueType vector font parsing.
- `src/geometry/mod.rs` & `src/document/mod.rs`:
  - Document model, layer definitions, shape primitives (paths, rectangles, circles, gears, text), transformations, bounding box calculations.

---

## Build, Test & Deployment Commands
```bash
# Manifest location:
/Volumes/Dataa/Ohjelmointi/ProjektitRust/laser_burn/Cargo.toml

# Run unit tests (35 tests):
cargo test --manifest-path /Volumes/Dataa/Ohjelmointi/ProjektitRust/laser_burn/Cargo.toml --features gui

# Compile optimized release build:
cargo build --manifest-path /Volumes/Dataa/Ohjelmointi/ProjektitRust/laser_burn/Cargo.toml --release --features gui

# Deploy to macOS application bundle and CLI:
cp /Volumes/Dataa/Ohjelmointi/ProjektitRust/laser_burn/target/release/laser_burn /Applications/LaserBurn.app/Contents/MacOS/LaserBurn
cp /Volumes/Dataa/Ohjelmointi/ProjektitRust/laser_burn/target/release/laser_burn ~/.cargo/bin/laser_burn

# Launch/restart app:
pkill -x LaserBurn || true
open /Applications/LaserBurn.app
```

---

## Continuation Roadmap (Future Enhancements)
- **Automatic Camera Calibration:** Fiducial/checkerboard automated lens distortion solving using corner detection.
- **Trace Bitmap (Vectorization):** Potrace/contour-based vectorization of raster images directly inside LaserBurn.
- **Rotary Attachment Support:** Cylinder/chuck engraving coordinate transformation mode.
- **DXF Import/Export:** Direct AutoCAD DXF format parser.
