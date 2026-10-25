# LaserBurn — Project Status & Architecture Context

## Current Status (as of October 25, 2026)
LaserBurn is a high-performance, modern laser CAM/CNC software written in Rust (`eframe`/`egui`).
It is fully functional, all 51 unit tests pass, and it is deployed as a native signed macOS app bundle (`/Applications/LaserBurn.app`), as well as multi-platform distribution packages in `dist/` (macOS Universal 2 DMG/ZIP, Windows x64 ZIP, and Linux x86_64 tar.gz).

### Key Milestones & Features Implemented:
1. **True Analog Grayscale Power Modulation (Keramiikkalaatat & Monikerroskaiverrus):**
   - Implemented real analog variable laser power engraving (`DitherMode::GrayscalePower`).
   - Linearly modulates dynamic PWM laser power (`M4 S...`) between `CutSetting.power_min` and `CutSetting.power_max` based on pixel density.
   - **Overscan margin (lead-in & lead-out):** Allows the laser head to accelerate/decelerate outside the active burn zone at 0% power, completely eliminating burned tile edges.
   - **Negative / Invert toggle:** Invert grayscale mapping for dark-coated ceramic tiles where the laser reveals underlying white/colored base layers.
   - **Run-Length & tolerance compression:** Automatically groups adjacent pixels with similar power to prevent USB serial planner buffer underrun/stuttering on GRBL.

2. **8-Handle Non-Uniform Resizing & Aspect Ratio Control:**
   - Interactive 8 resize handles on canvas (TopLeft, Top, TopRight, Right, BottomRight, Bottom, BottomLeft, Left) with high-visibility 10x10 px squares and generous 16px screen hit tolerance.
   - Fixed handle drag hit detection by tracking mouse press origin (`press_origin`) and avoiding accidental deselection.
   - When **"Lock Aspect Ratio"** is unchecked:
     - Side handles (Left / Right) scale width exclusively.
     - Top / Bottom handles scale height exclusively.
     - Corner handles scale width and height independently based on mouse coordinates.
   - When **"Lock Aspect Ratio"** is checked:
     - All 8 handles preserve the exact aspect ratio of the selection.
   - Non-uniform scaling of `Shape::Circle` automatically morphs it into a smooth 48-segment closed `Shape::Path` oval/ellipse, allowing ovals of any ratio.
   - Direct `"Circle / Oval"` drawing tool: dragging without Shift creates an oval (with real-time ellipse preview), and holding Shift forces a perfect uniform circle.
   - Full image scaling: `Shape::Image` can be resized arbitrarily on canvas or via properties panel.

2. **Accurate Job Duration Estimation (Kestoaika-arvio):**
   - Implemented fast, non-blocking trapezoidal acceleration physics modeling (`estimate_document`).
   - Computes exact acceleration/cruise/deceleration travel times for both vector cuts and multi-thousand line raster images without heavy per-frame dithering lag.
   - **Pre-run estimate:** Displayed in the right control panel directly above the `[ START JOB (CUT) ]` button (`⏱ Arvioitu kesto: ~XX min YY s`) and in the `Simulation Stats` section.
   - **Live execution progress:** Displays elapsed time and dynamically extrapolated remaining time (`⏱ Kulunut: MM:SS | Jäljellä: ~MM:SS`) next to the progress bar.
   - Reports completed execution duration to device console upon job finish.

3. **High-Precision Image Import & Engraving Dithering:**
   - Supported formats: PNG, JPG, JPEG, and BMP via "File -> Open" (`Cmd+O`), "File -> Import Image..." (`Cmd+I`), or `[🖼 Import Image]` toolbar button.
   - 8 advanced dithering algorithms: Atkinson, Floyd-Steinberg, Jarvis-Judice-Ninke, Stucki, Burkes, Sierra3, Bayer 8x8, and Halftone.
   - Generates bidirectional serpentine raster scanlines directly into toolpaths.
   - Bypasses $O(N^2)$ TSP reordering on image layers to preserve optimal scanline sequence and eliminate processing delay.
   - Live GPU-accelerated texture preview on canvas.

4. **Full SVG Parser with 2D Affine Transformation:**
   - Full SVG path tokenizer & executor (`M/m`, `L/l`, `H/h`, `V/v`, `C/c`, `S/s`, `Q/q`, `T/t`, `A/a`, `Z/z`).
   - 2D affine matrix transformation (`Transform2D`) supporting nested `<g transform="...">` (`matrix`, `translate`, `scale`, `rotate`) and `<use>` elements.
   - Automatic workspace centering and layer color mapping.

5. **Multi-Platform Distribution & Auto-Deployment:**
   - Unified build script (`./scripts/build_all.sh`) creating all release assets in `dist/`:
     - `LaserBurn-1.0.0-beta.1-universal.dmg` (macOS Universal 2: Apple Silicon + Intel, code-signed).
     - `LaserBurn-1.0.0-beta.1-windows-x64.zip` (Windows 10/11 standalone 64-bit `.exe`).
     - `LaserBurn-1.0.0-beta.1-linux-x86_64.tar.gz` (Linux 64-bit ELF binary, `.desktop` launcher, icon).
   - Automatically installs the latest Universal 2 build into local Mac `/Applications/LaserBurn.app`.

6. **Gumroad Product Listing & Assets:**
   - Product Page: `https://shop.pietiniart.com/l/laserburn`
   - Store: Gumroad (`shop.pietiniart.com / nikopietinen.gumroad.com`).
   - Listing assets generated in `dist/gumroad_assets/`:
     - Cover image: `dist/gumroad_assets/gumroad_cover.jpg` (1280x720 px, 16:9).
     - Thumbnail icon: `dist/gumroad_assets/gumroad_thumb_square.jpg` (1000x1000 px, 1:1).
   - Sales copy and listing guide: `GUMROAD_LISTING_GUIDE.md`.

7. **Camera Alignment & Homography System:**
   - 4-point perspective homography warp solver using 4 scrap paper targets on the bed.
   - Floating zoom loupe (2x–8x magnifier) for sub-pixel crosshair centering.
   - Live bed background texture overlay (`Cmd+Shift+C`) mapped via inverse homography $H^{-1}$.

8. **Controller Communication & Safety:**
   - GRBL 1.1+ USB serial communication and Ruida DSP network/USB support.
   - Real-time limit switch status reporting ($10=19 Pn:), software limits, and learned workspace envelope.
   - Clean emergency stop (`!`) preserving coordinate origin (0,0) without losing homing position.

---

## Codebase Map & Modules (`src/`)
- `src/main.rs`: Application entry point, CLI commands (`clap`), native menu runner (`eframe`).
- `src/lib.rs`: Library exports and integration test suite (50 tests).
- `src/gui/mod.rs`:
  - Main user interface with 2D canvas, sticky rulers, adaptive grid.
  - Selection, 8-handle non-uniform resize, bounding box drag, rotation, alignment.
  - Job duration estimation display and live execution tracking.
  - Layer palette (C00–C29) with mode selection (Line, Fill, Image).
  - Photographic raster image rendering with GPU textures.
- `src/document/mod.rs`:
  - Document model, layer definitions, shape primitives (`Path`, `Rect`, `Circle`, `Image`, `Group`).
  - Non-uniform scaling logic (automatic Circle to Path oval conversion, Image scaling).
  - Raster scanline generation (`to_raster_polylines`) with dithering.
- `src/simulator/mod.rs`:
  - Trapezoidal velocity and acceleration simulation (`Simulator`).
  - Fast whole-document estimation (`estimate_document`).
  - Duration formatting helper (`format_duration`).
- `src/device/mod.rs`:
  - `DeviceManager`: GRBL and Ruida controller streaming and state machine.
  - Live job timer (`job_start_instant`, `job_estimated_total_sec`), progress calculation, pause/resume/stop.
- `src/formats/`:
  - `lbrn2.rs`: LightBurn 2.0 XML reader/writer (`roxmltree`).
  - `svg.rs`: Full SVG path tokenizer and 2D affine matrix parser.
  - `gcode.rs`: GRBL G-code generator.
  - `ruida.rs`: Binary Ruida `.rd` generator.
- `src/optimizer/mod.rs`:
  - Path optimization, inner-first contour sorting, TSP rapid travel minimization (bypassed on image layers).
- `src/dither/mod.rs`:
  - 8 grayscale dithering algorithms for wood, acrylic, slate engraving.
- `src/camera/mod.rs`:
  - `CameraManager`: USB webcam video streaming (`nokhwa`).
  - `CameraCalibration` & `LensUndistorter`: Lens distortion and 4-point homography calibration.

---

## Build, Test & Deployment Commands
```bash
# Manifest location:
/Volumes/Dataa/Ohjelmointi/ProjektitRust/laser_burn/Cargo.toml

# Run all 50 unit tests:
cargo test --manifest-path /Volumes/Dataa/Ohjelmointi/ProjektitRust/laser_burn/Cargo.toml --features gui

# Build all distribution packages and auto-deploy to /Applications/LaserBurn.app:
./scripts/build_all.sh

# Build macOS Universal 2 bundle only:
./scripts/build_dmg.sh universal

# Launch app:
open /Applications/LaserBurn.app
```
