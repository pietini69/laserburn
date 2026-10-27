#!/usr/bin/env python3
"""
update_gumroad_to_v120.py
Updates LaserBurn on Gumroad to v1.2.0 with the latest binaries,
release notes, and detailed version-by-version changelog section.
"""

import os
import sys
import json
import subprocess

PROJECT_DIR = "/Volumes/Dataa/Ohjelmointi/ProjektitRust/laser_burn"
DIST_DIR = os.path.join(PROJECT_DIR, "dist")
PRODUCT_ID = "1SXqccPmq2GJB3uOomjR3Q=="

mac_dmg = os.path.join(DIST_DIR, "LaserBurn-1.2.0-universal.dmg")
win_zip = os.path.join(DIST_DIR, "LaserBurn-1.2.0-windows-x64.zip")
lin_tar = os.path.join(DIST_DIR, "LaserBurn-1.2.0-linux-x86_64.tar.gz")

for f in [mac_dmg, win_zip, lin_tar]:
    if not os.path.exists(f):
        print(f"Error: Required binary not found: {f}", file=sys.stderr)
        sys.exit(1)

DESCRIPTION_HTML = """
<h2>🔥 Meet LaserBurn v1.2.0: High-Performance Laser CAM & CNC Software</h2>
<p>
LaserBurn is a modern, blazing-fast laser cutting and engraving CAM application engineered from the ground up in <strong>Rust</strong>. Designed for makers, craft businesses, ceramic tile engravers, and high-precision fabrication workshops, LaserBurn delivers sub-second vector path optimization, instant project loading, and full multi-controller hardware control — with <strong>no subscriptions and no bloat</strong>.
</p>
<p>
<strong>Free / Pay What You Want (0 €+):</strong> Download and use LaserBurn completely free, or support independent development with the price of a cup of coffee (3 € – 5 €)!<br>
<strong>All 3 Desktop Platforms Included:</strong> macOS (Universal 2 for Apple Silicon & Intel), Windows 10/11 (64-bit), and Linux (x86_64).
</p>

<hr>

<h3>🚀 What's New in Version 1.2.0</h3>
<ul>
  <li><strong>🖊️ Centerline Trace (Keskiviivajäljitys / Single Stroke):</strong> High-precision vectorization using Zhang-Suen morphological skeletonization. Reduces thick bitmap handwriting, signatures, ink sketches, and line art into true single-stroke vector paths in seconds — saving up to 90% cutting time compared to raster engraving!</li>
  <li><strong>📏 Full Dual Units System (Imperial & Metric / Tuumat ja Millimetrit):</strong> Instant 1-click toggle between Millimeters (mm) and Inches (in) across the entire application: dynamic canvas rulers with inch markings (1", 2", 3"...), Transform & Size controls (0.001 in precision), status bar coordinates, workbed dimensions, and jog steps.</li>
  <li><strong>🔒 Interactive Lockable Layers (Lukittavat tasot):</strong> Click the <code>[🔒]</code> / <code>[🔓]</code> button on any layer to lock it in place. Shapes on locked layers cannot be accidentally clicked, dragged, box-selected, or deleted — ideal for alignment jigs, material cut boundaries, and reference templates.</li>
  <li><strong>🛡️ Parameter-Isolated Undo Stack:</strong> Geometric canvas Undo (<code>Cmd+Z</code> / <code>Ctrl+Z</code>) is strictly separated from laser cutting parameters. Undoing a drawing mistake or shape move will never wipe or reset your carefully calibrated speed, power, passes, or air assist settings.</li>
  <li><strong>⚡ Auto-Nest Materials Optimizer:</strong> Built-in 2D bottom-left packing with multi-angle rotation to pack parts tightly onto the sheet and minimize scrap waste.</li>
</ul>

<hr>

<h3>📜 Versiohistoria & Muutosloki (Full Version History)</h3>

<h4>Version 1.2.0 (2026-10-27)</h4>
<ul>
  <li>Added <strong>Centerline Trace</strong> with Zhang-Suen morphological thinning and RDP path smoothing for handwriting and single-stroke vector scoring.</li>
  <li>Added full <strong>Dual Units System</strong> (Millimeters & Inches) with dynamic inch canvas rulers, status bar coordinates, workbed dimensions, and jog steps.</li>
  <li>Added <strong>Lockable Layers</strong> with interactive lock buttons in the cuts/layers table to protect alignment templates and jigs from accidental editing.</li>
  <li>Added <strong>Parameter-Isolated Undo</strong>: Canvas vector undos never clobber laser power or speed settings.</li>
</ul>

<h4>Version 1.1.0 (2026-10-26)</h4>
<ul>
  <li>Added <strong>Apple Shortcuts Automation</strong> on macOS to trigger smart workshop automations (exhaust fans, lighting, iOS push notifications) on job start/finish.</li>
  <li>Added full <strong>LightBurn .lbrn2 XML Compatibility</strong> for opening and saving modern LightBurn project files with full layer hierarchy and cut parameters.</li>
  <li>Added <strong>Real-Time Pin Reporting</strong> ($10=19) for limit switches, door interlocks, and touch probes, plus <strong>Hard Limits Protection</strong> ($21=1).</li>
  <li>Added <strong>8-Point Interactive Transform Handles</strong> for free aspect-ratio scaling and instant circle-to-oval resizing.</li>
  <li>Added <strong>Speed × Power Material Test Matrix Generator</strong>.</li>
</ul>

<h4>Version 1.0.0 (Initial Release)</h4>
<ul>
  <li>Sub-second Rust CAM optimization engine and physics-based acceleration profiling.</li>
  <li>Multi-controller support: direct USB serial for <strong>GRBL 1.1+</strong> and network/USB for <strong>Ruida DSP</strong> controllers.</li>
  <li>Studio-grade image dithering engine with 9 algorithms (Atkinson, Floyd-Steinberg, Jarvis-Judice-Ninke, Stucki, Burkes, Sierra3, Bayer 8x8, Halftone, Grayscale).</li>
  <li>Dynamic PWM grayscale power modulation with ceramic paint layer simulator.</li>
  <li>Live USB camera bed overlay with 4-point homography perspective calibration and 1-click tile auto-fitting.</li>
  <li>Vector CAD tools: 2D Booleans (Weld, Subtract, Intersect) and true SDF contour offsetting.</li>
</ul>

<hr>

<h3>⚡ Core Features</h3>
<ul>
  <li><strong>📂 LightBurn .lbrn2 & SVG Compatibility:</strong> Open and save modern LightBurn project files directly, including vector paths, layer assignments (C00–C29), cut speeds, power settings, and multi-pass rules.</li>
  <li><strong>🎨 True Analog Grayscale Power Modulation:</strong> Real dynamic PWM power modulation (<code>M4 S...</code>) with automatic overscan lead-in/lead-out. Includes ceramic paint layer simulator.</li>
  <li><strong>📷 Live Camera Bed Overlay & 1-Click Auto-Fit:</strong> Connect any USB webcam or overhead camera with 4-point homography perspective calibration.</li>
  <li><strong>🛠️ Vector CAD & 2D Boolean Tools:</strong> Built-in vector drawing, 2D Boolean operations (Weld, Subtract, Intersect), and true Signed Distance Field (SDF) contour offsetting.</li>
  <li><strong>🖨️ Multi-Controller Support:</strong> USB serial support for <strong>GRBL 1.1+</strong> (Ortur, xTool, Sculpfun, Atomstack, Creality, Two Trees, Neje) and network/USB support for <strong>Ruida DSP</strong> controllers.</li>
</ul>

<hr>

<h3>🛠️ Installation Instructions</h3>

<h4>🍎 macOS:</h4>
<p>
1. Open <code>LaserBurn-1.2.0-universal.dmg</code> and drag the <strong>LaserBurn</strong> icon into your <em>Applications</em> folder.<br>
2. <strong>First Launch (Gatekeeper Security):</strong><br>
Right-click (or Control-click) <code>LaserBurn.app</code> in your Applications folder and select <strong>Open</strong>, then click <strong>Open</strong> in the prompt. (Or go to System Settings → Privacy & Security → click "Open Anyway").
</p>

<h4>🪟 Windows:</h4>
<p>
1. Extract <code>LaserBurn-1.2.0-windows-x64.zip</code> into any folder (e.g. <code>C:\\LaserBurn</code>).<br>
2. Double-click <code>LaserBurn.exe</code> to launch.<br>
3. <strong>First Launch (SmartScreen):</strong> Click "More info" → "Run anyway".
</p>

<h4>🐧 Linux:</h4>
<p>
1. Extract <code>LaserBurn-1.2.0-linux-x86_64.tar.gz</code> and run <code>./LaserBurn</code>.<br>
2. Ensure your user belongs to the serial group: <code>sudo usermod -aG dialout $USER</code>.
</p>

<hr>

<p><em>LaserBurn v1.2.0. Developed by Niko Pietinen. Free updates and continuous improvements included.</em></p>
""".strip()

SUMMARY = "High-performance, modern laser CAM & CNC software built in Rust (v1.2.0). Features Centerline Trace, dual Imperial/Metric units, lockable layers, parameter-isolated undo, LightBurn .lbrn2 XML compatibility, true analog grayscale modulation & live camera auto-fit for macOS, Windows & Linux."

def main():
    print(f"==> Updating LaserBurn to v1.2.0 on Gumroad (Product ID: {PRODUCT_ID})...")
    cmd = [
        "gumroad", "products", "update", PRODUCT_ID,
        "--name", "LaserBurn 1.2.0 – Modern Laser CAM & CNC Software (macOS, Windows, Linux)",
        "--custom-summary", SUMMARY,
        "--description", DESCRIPTION_HTML,
        "--file", mac_dmg,
        "--file-name", "LaserBurn-1.2.0-macOS-Universal.dmg",
        "--file-description", "macOS Universal 2 Installer (Apple Silicon M1-M4 & Intel 64-bit). macOS 11.0+",
        "--file", win_zip,
        "--file-name", "LaserBurn-1.2.0-Windows-x64.zip",
        "--file-description", "Windows 10 / 11 64-bit Standalone Executable & Package",
        "--file", lin_tar,
        "--file-name", "LaserBurn-1.2.0-Linux-x86_64.tar.gz",
        "--file-description", "Linux 64-bit x86_64 Standalone Tarball (glibc 2.31+)"
    ]

    res = subprocess.run(cmd, capture_output=True, text=True)
    if res.returncode != 0:
        print(f"Error updating product: {res.stderr}", file=sys.stderr)
        sys.exit(res.returncode)

    print("==> Base product metadata updated successfully!")

    # Set clean customer content download cards
    content_text = """
Thank you for downloading LaserBurn v1.2.0!

Choose the download package for your operating system:

• macOS: Download LaserBurn-1.2.0-macOS-Universal.dmg (Supports both Apple Silicon M1/M2/M3/M4 and Intel Macs). Drag to Applications and open!
• Windows: Download LaserBurn-1.2.0-Windows-x64.zip (Windows 10 & 11 64-bit). Extract and run LaserBurn.exe!
• Linux: Download LaserBurn-1.2.0-Linux-x86_64.tar.gz (Ubuntu, Debian, Fedora, Arch). Extract and run ./LaserBurn!

Enjoy blazing-fast laser CAM, Centerline Trace, dual units, and precision G-code generation!
""".strip()

    res_content = subprocess.run([
        "gumroad", "products", "content", "set", PRODUCT_ID,
        "--text", content_text
    ], capture_output=True, text=True)

    if res_content.returncode != 0:
        print(f"Warning: Could not update rich content: {res_content.stderr}", file=sys.stderr)
    else:
        print("==> Product download content updated successfully!")

    # Update local tracking JSON
    tracking_path = "/Volumes/Dataa/EtsyTuotteet/gumroad_published.json"
    if os.path.exists(tracking_path):
        try:
            with open(tracking_path, "r", encoding="utf-8") as f:
                data = json.load(f)
            data["laser_burn"] = {
                "product_id": PRODUCT_ID,
                "version": "1.2.0",
                "name": "LaserBurn 1.2.0 – Modern Laser CAM & CNC Software (macOS, Windows, Linux)",
                "updated_at": "2026-10-27T09:47:00+02:00",
                "files": [
                    "LaserBurn-1.2.0-universal.dmg",
                    "LaserBurn-1.2.0-windows-x64.zip",
                    "LaserBurn-1.2.0-linux-x86_64.tar.gz"
                ]
            }
            with open(tracking_path, "w", encoding="utf-8") as f:
                json.dump(data, f, indent=2, ensure_ascii=False)
            print("==> Local tracking file gumroad_published.json updated!")
        except Exception as e:
            print(f"Warning updating tracking JSON: {e}", file=sys.stderr)

    print("\n🎉 LaserBurn v1.2.0 is now LIVE on Gumroad!")

if __name__ == "__main__":
    main()
