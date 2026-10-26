# LaserBurn — Project Status & Architecture Context

## Current Status (v1.0.0 Official Release — October 26, 2026)
LaserBurn is a high-performance, modern laser CAM/CNC software written in Rust (`eframe`/`egui`).
It is at official version **v1.0.0**, all 55 unit tests pass, and it is deployed as a native signed macOS app bundle (`/Applications/LaserBurn.app`), as well as multi-platform distribution packages in `dist/` (macOS Universal 2 DMG/ZIP, macOS arm64 DMG, macOS Intel DMG, Windows x64 ZIP, and Linux x86_64 tar.gz).

### Key Milestones & Features Implemented in v1.0.0:
1. **True Analog Grayscale Power Modulation & Ceramic Paint Layer Simulator:**
   - Real analog variable laser power engraving (`DitherMode::GrayscalePower`) modulating dynamic PWM power (`M4 S...`) between `CutSetting.power_min` and `CutSetting.power_max`.
   - **Overscan margin (lead-in & lead-out):** Accelerates/decelerates outside the active burn zone at 0% power to prevent burned edges.
   - **Negative / Invert toggle:** Inverts grayscale mapping for dark-coated ceramic tiles.
   - **Multi-Color Ceramic Tile Paint Layer Simulator:** Interactive paint stack preview simulating how laser power levels ablate through top/middle/base paint layers down to the ceramic tile substrate.

2. **Vector CAD, 2D Boolean Engine, SDF Contour Offset & Undo/Redo:**
   - Multi-step Undo/Redo (`Cmd+Z` / `Cmd+Shift+Z` / `Cmd+Y`) across all canvas and transform actions.
   - 2D Boolean operations (`Weld / Union`, `Subtract A - B`, `Intersect`) via high-resolution raster contour reconstruction.
   - Signed Distance Field (`SDF`) inner and outer contour offset tool.
   - Interactive 8-handle non-uniform & aspect-locked resizing, circle-to-oval morphing, and full SVG affine transformation support.

3. **Built-in Material Test Matrix Generator (Speed × Power Grid):**
   - One-click parametric calibration matrix generator creating multi-layer test squares with automatic vector speed/power labels.

4. **Camera Alignment, Homography & 1-Click Workpiece Auto-Fit:**
   - 4-point perspective homography warp solver with floating 2x–8x zoom loupe.
   - 1-Click `Detect & Center on Workpiece` computer vision routine that locates rectangular blanks (tiles, coasters, wood scraps) on the bed and centers/scales selected shapes onto the blank.

5. **Dedicated Real-Time Data Traffic Log (TX / RX) & Plain-English G-code Explainer:**
   - Side-by-side `Data Traffic (TX / RX)` panel docked next to the right inspector.
   - `[✓] Explain` mode (`DeviceManager::explain_traffic_line`) providing inline plain-English explanations for G-code/M-code commands, GRBL status frames, `$` settings, errors, and alarms.
   - `[✓] Save to Log File` streaming timestamped traffic to `~/Documents/LaserBurn_traffic.log`.

6. **Dark / Light Theme Toggle, Origin-Aware Jog D-Pad & macOS Apple Shortcuts Automation:**
   - High-contrast Dark Mode and Light Mode toggle persisted in `DeviceSettings.dark_mode`.
   - Dynamic `X+`/`X-`/`Y+`/`Y-` Jog D-pad buttons that automatically adapt to the machine's `OriginCorner` (`0,0` location).
   - macOS-exclusive (`#[cfg(target_os = "macos")]`) Apple Shortcuts integration (`On Job Start` / `On Job Complete`) with installed Shortcut discovery (`shortcuts list`) and one-click test runner (`shortcuts run`).

7. **Multi-Platform Distribution & Gumroad Store (`v1.0.0`):**
   - Product Page: `https://shop.pietiniart.com/l/laserburn`
   - Pricing: `0 €+` Pay What You Want with a coffee-cup suggested price (`5 €`).
   - Release packages in `dist/`:
     - `LaserBurn-1.0.0-universal.dmg` & `LaserBurn-1.0.0-macos-universal.zip`
     - `LaserBurn-1.0.0-arm64.dmg` & `LaserBurn-1.0.0-intel-x86_64.dmg`
     - `LaserBurn-1.0.0-windows-x64.zip`
     - `LaserBurn-1.0.0-linux-x86_64.tar.gz`

---

## Build, Test & Deployment Commands
```bash
# Run all unit tests:
cargo test --manifest-path /Volumes/Dataa/Ohjelmointi/ProjektitRust/laser_burn/Cargo.toml --features gui

# Build all distribution packages and auto-deploy to /Applications/LaserBurn.app:
./scripts/build_all.sh
```
