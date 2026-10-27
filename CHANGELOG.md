# 📜 PhotonCAM Changelog & Release History

All notable changes, new features, and bug fixes for **PhotonCAM** across every release.

---

## [v1.2.5] – 2026-10-27

### 🌟 New Features & Major Additions
* **🌐 Web Remote & iPad / Tablet Touch Support:**
  * Embedded zero-dependency HTTP/1.1 & Server-Sent Events (SSE) web server running on port 8080 (configurable).
  * Responsive HTML5 + Canvas Web Remote interface optimized for iPad, tablets, and mobile phones with dark mode and Apple PWA standalone support.
  * Interactive 2D bed preview, jog D-Pad with customizable steps, dual units toggle (mm/in), live coordinate streaming at 10Hz, file upload (`.gcode`, `.lbrn2`, `.svg`), start/pause/stop job streaming, focus light toggle, and instant E-Stop.
  * Transparent TCP-to-Serial bridge on port 2323 for wireless network connection.
  * One-click start/stop and browser launch directly from the Desktop toolbar (`🌐 Web Remote`) or headless via `laserburn-bridge`.
* **🍓 Raspberry Pi Headless Bridge & Ready-to-Flash OS Image:**
  * Standalone lightweight CLI daemon `laserburn-bridge` for Raspberry Pi (ARM64 & ARMv7) and Linux.
  * Automated 1-line installation script (`curl -sSL https://raw.githubusercontent.com/pietini69/laserburn/master/scripts/install_pi_bridge.sh | bash`).
  * Ready-to-flash Raspberry Pi OS Disk Image (`PhotonCAM-Pi-OS-arm64.img.xz`) with Avahi mDNS (`laserburn.local:8080`) and auto-launch systemd service.
* **📷 Optional Camera Mode (Distraction-Free Minimal UI):**
  * Added **"Enable Camera Support"** setting in Settings (`Cmd+,`).
  * When unchecked, all camera toolbar buttons (`Camera`, `Cam BG`, `Update Overlay`, `Snap to Tile`), Camera menus, and camera shortcuts are completely hidden from the workspace, providing a clean, distraction-free CAD/CAM layout for users without an overhead camera.
* **🔍 Job Preview & Cut Simulation:**
  * Interactive simulation window with animated laser head, scrub timeline, accurate time estimation, and color-coded rapid traverse vs cutting/engraving moves.
* **📏 Dual Units Display Improvements:**
  * Status bar coordinates and jog controls now dynamically format in inches when Imperial mode is active.

---

## [v1.2.0] – 2026-10-27

### 🌟 New Features & Major Capabilities
* **Centerline Trace (Keskiviivajäljitys / Zhang-Suen Morphological Skeletonization):**
  * Added single-stroke centerline vectorization alongside contour tracing in the *Trace Image* dialog.
  * Uses the two-phase Zhang-Suen morphological thinning algorithm to reduce thick bitmap lines into true 1-pixel-wide mathematical skeletons.
  * Graph-based 8-connected path generator with Ramer-Douglas-Peucker simplification, turning handwriting, ink drawings, and sketches into smooth open/closed vector cutting paths.
  * Perfect for handwritten family recipes, signatures, and line art — saves up to 90% cutting time compared to raster engraving.
* **Full Dual Units System (Imperial & Metric / Tuumat ja Millimetrit):**
  * One-click unit toggle: `[📏 mm]` / `[📏 in]` in top menubar, Transform panel header, and Settings dialog.
  * **Dynamic Canvas Rulers & Grid:** When Inches is active, rulers display inch markings (`1"`, `2"`, `3"`, `4"`, etc.) with 0.25", 0.5", and 1.0" subdivisions.
  * **Transform & Size Controls:** Direct position (`X`, `Y`) and dimensions (`Width`, `Height`) in inches with 0.001 in precision.
  * **Status Bar & Bed Dimensions:** Live machine coordinates and workbed dimensions format seamlessly in inches with dual metric helper.
  * **Jog Steps:** Siirtoaskeleet vaihtuvat tuumiksi (`0.1"`, `0.5"`, `1.0"`).
* **Interactive Lockable Layers (Lukittavat tasot):**
  * Interactive `[🔒]` / `[🔓]` lock button on every layer in the Cuts & Layers panel.
  * Protects shapes on locked layers from accidental mouse clicks, dragging, resizing, marquee selection, and deletion.
  * Ideal for alignment jigs, material cut boundaries, and background reference templates.
* **Parameter-Isolated Undo Stack (Turvallinen Undo):**
  * Geometric canvas Undo (`Cmd+Z` / `Ctrl+Z`) is strictly separated from laser cutting parameters.
  * Undoing drawing edits or shape moves will never reset carefully tuned laser speed, power, passes, or air-assist parameters.

---

## [v1.1.0] – 2026-10-26

### 🚀 Highlights
* **🍎 Apple Shortcuts Automation (macOS):**
  * Trigger native macOS Shortcuts automatically when a job starts or completes (turn on workshop exhaust fans, adjust lighting, send push notifications to iPhone/Apple Watch).
* **📂 Native LightBurn .lbrn2 XML Compatibility:**
  * Full two-way support for opening and saving modern LightBurn project files with vector paths, circles, rectangles, cut settings, kerf compensation, and scan speeds.
* **🛡️ Real-Time Pin Reporting & Safety Protection:**
  * Live GRBL pin status reporting (`$10=19`) for limit switches, door interlocks, and touch probes.
  * One-click hard limits protection (`$21=1`) and soft limit boundary monitoring.
* **🔄 Interactive Transform Handles & Free Resizing:**
  * 8 interactive boundary handles for free aspect-ratio scaling or constrained proportional scaling, with instant circle-to-oval conversion.
* **⚡ Sub-Second Rust G-Code Engine:**
  * Optimized path planning and real-time execution monitor with physics-based acceleration profiling.

---

## [v1.0.0] – Initial Release

### ⚡ Core Foundation
* **Multi-Controller Support:** Direct USB serial support for **GRBL 1.1+** and network/USB support for **Ruida DSP** controllers.
* **Studio-Grade Image Dithering:** 9 high-quality dithering modes including Atkinson, Floyd-Steinberg, Jarvis-Judice-Ninke, Stucki, Burkes, Sierra3, Ordered Bayer 8x8, Halftone, and True Grayscale.
* **Sub-Pixel Marching Squares Vectorizer:** High-speed contour vectorization from raster logos and silhouettes.
* **Ceramic Tile & Multi-Color Simulation:** Dynamic PWM laser power modulation with real-time tile layer preview.
* **Live Camera Bed Alignment:** USB webcam support with 4-point homography calibration and 1-click tile auto-fitting.
* **Cross-Platform:** Native builds for macOS (Apple Silicon & Intel), Windows 10/11, and Linux.
