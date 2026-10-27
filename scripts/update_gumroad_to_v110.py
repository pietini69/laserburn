#!/usr/bin/env python3
"""
update_gumroad_to_v110.py
Updates LaserBurn on Gumroad to v1.1.0 with the latest binaries,
release notes, and updated documentation.
"""

import os
import sys
import json
import subprocess

PROJECT_DIR = "/Volumes/Dataa/Ohjelmointi/ProjektitRust/laser_burn"
DIST_DIR = os.path.join(PROJECT_DIR, "dist")
PRODUCT_ID = "1SXqccPmq2GJB3uOomjR3Q=="

mac_dmg = os.path.join(DIST_DIR, "LaserBurn-1.1.0-universal.dmg")
win_zip = os.path.join(DIST_DIR, "LaserBurn-1.1.0-windows-x64.zip")
lin_tar = os.path.join(DIST_DIR, "LaserBurn-1.1.0-linux-x86_64.tar.gz")

for f in [mac_dmg, win_zip, lin_tar]:
    if not os.path.exists(f):
        print(f"Error: Required binary not found: {f}", file=sys.stderr)
        sys.exit(1)

DESCRIPTION_HTML = """
<h2>🔥 Meet LaserBurn v1.1.0: High-Performance Laser CAM & CNC Software</h2>
<p>
LaserBurn is a modern, blazing-fast laser cutting and engraving CAM application engineered from the ground up in <strong>Rust</strong>. Designed for hobbyists, craft businesses, ceramic tile engravers, and high-precision fabrication workshops, LaserBurn delivers sub-second vector path optimization, instant project loading, and full multi-controller hardware control — with <strong>no subscriptions and no bloat</strong>.
</p>
<p>
<strong>Free / Pay What You Want (0 €+):</strong> Download and use LaserBurn completely free, or support independent development with the price of a cup of coffee (3 € – 5 €)!<br>
<strong>All 3 Desktop Platforms Included:</strong> macOS (Universal 2 for Apple Silicon & Intel), Windows 10/11 (64-bit), and Linux (x86_64).
</p>

<hr>

<h3>🚀 What's New in Version 1.1.0</h3>
<ul>
  <li><strong>🍎 Apple Shortcuts Automation (macOS):</strong> Trigger native macOS Shortcuts automatically when a job starts or finishes (e.g. turn on workshop exhaust fans, adjust smart lighting, and receive push notifications on iPhone/Apple Watch).</li>
  <li><strong>📂 Native LightBurn .lbrn2 XML Compatibility:</strong> Full two-way support for opening and saving modern LightBurn project files with vector paths, circles, rectangles, cut settings, kerf compensation, and scan speeds.</li>
  <li><strong>🛡️ Real-Time Pin Reporting & Safety Protection:</strong> Live GRBL pin status reporting ($10=19) for limit switches, door interlocks, and touch probes, with one-click hard limit protection ($21=1) and emergency stop origin preservation.</li>
  <li><strong>🔄 Interactive Transform Handles & Free Resizing:</strong> 8 interactive boundary handles for free aspect-ratio scaling or constrained proportional scaling, with instant circle-to-oval conversion.</li>
  <li><strong>⚡ Sub-Second Rust G-Code Engine:</strong> Optimized path planning and real-time execution monitor with physics-based acceleration profiling.</li>
</ul>

<hr>

<h3>⚡ Core Features</h3>
<ul>
  <li><strong>📂 LightBurn .lbrn2 & SVG Compatibility:</strong> Open and save modern LightBurn project files directly, including vector paths, layer assignments (C00–C29), cut speeds, power settings, and multi-pass rules.</li>
  <li><strong>🎨 True Analog Grayscale Power Modulation & Ceramic Paint Layer Simulator:</strong> Real dynamic PWM power modulation (<code>M4 S...</code>) with automatic overscan lead-in/lead-out. Includes a specialized multi-color paint simulator to preview burned tile layers.</li>
  <li><strong>📷 Live Camera Bed Overlay & 1-Click Auto-Fit:</strong> Connect any USB webcam or overhead camera with 4-point homography perspective calibration and sub-pixel zoom loupe.</li>
  <li><strong>🛠️ Vector CAD & 2D Boolean Tools:</strong> Built-in vector drawing, multi-step Undo/Redo (<code>Cmd+Z</code>), 2D Boolean operations (Weld, Subtract, Intersect), and true Signed Distance Field (SDF) contour offsetting.</li>
  <li><strong>🧪 Material Speed × Power Test Matrix Generator:</strong> Generate ready-to-burn calibration grids with automatic vector labels in one click.</li>
  <li><strong>🖼️ Studio-Grade Image Dithering (9 Modes):</strong> Atkinson, Floyd-Steinberg, Jarvis-Judice-Ninke, Stucki, Burkes, Sierra3, Ordered Bayer 8x8, Halftone, and True Grayscale.</li>
  <li><strong>🖨️ Multi-Controller Support:</strong> USB serial support for <strong>GRBL 1.1+</strong> (Ortur, xTool, Sculpfun, Atomstack, Creality, Two Trees, Neje) and network/USB support for <strong>Ruida DSP</strong> controllers.</li>
</ul>

<hr>

<h3>🛠️ Installation Instructions</h3>

<h4>🍎 macOS:</h4>
<p>
1. Open <code>LaserBurn-1.1.0-universal.dmg</code> and drag the <strong>LaserBurn</strong> icon into your <em>Applications</em> folder.<br>
2. <strong>First Launch (Gatekeeper Security):</strong><br>
Right-click (or Control-click) <code>LaserBurn.app</code> in your Applications folder and select <strong>Open</strong>, then click <strong>Open</strong> in the prompt. (Or go to System Settings → Privacy & Security → click "Open Anyway").
</p>

<h4>🪟 Windows:</h4>
<p>
1. Extract <code>LaserBurn-1.1.0-windows-x64.zip</code> into any folder (e.g. <code>C:\\LaserBurn</code>).<br>
2. Double-click <code>LaserBurn.exe</code> to launch.<br>
3. <strong>First Launch (SmartScreen):</strong> Click "More info" → "Run anyway".
</p>

<h4>🐧 Linux:</h4>
<p>
1. Extract <code>LaserBurn-1.1.0-linux-x86_64.tar.gz</code> and run <code>./LaserBurn</code>.<br>
2. Ensure your user belongs to the serial group: <code>sudo usermod -aG dialout $USER</code>.
</p>

<hr>

<p><em>LaserBurn v1.1.0. Developed by Niko Pietinen. Free updates and continuous improvements included.</em></p>
""".strip()

SUMMARY = "High-performance, modern laser cutting & engraving CAM software built in Rust (v1.1.0). Native LightBurn .lbrn2 XML compatible, true grayscale power modulation, ceramic paint layer simulator, live camera auto-fit, Apple Shortcuts automation & path optimizer for macOS, Windows and Linux."

def main():
    print(f"==> Updating LaserBurn to v1.1.0 on Gumroad (Product ID: {PRODUCT_ID})...")
    cmd = [
        "gumroad", "products", "update", PRODUCT_ID,
        "--name", "LaserBurn 1.1.0 – Modern Laser CAM & CNC Software (macOS, Windows, Linux)",
        "--custom-summary", SUMMARY,
        "--description", DESCRIPTION_HTML,
        "--file", mac_dmg,
        "--file-name", "LaserBurn-1.1.0-macOS-Universal.dmg",
        "--file-description", "macOS Universal 2 Installer (Apple Silicon M1-M4 & Intel 64-bit). macOS 11.0+",
        "--file", win_zip,
        "--file-name", "LaserBurn-1.1.0-Windows-x64.zip",
        "--file-description", "Windows 10 & 11 (64-bit) Portable Standalone Executable",
        "--file", lin_tar,
        "--file-name", "LaserBurn-1.1.0-Linux-x86_64.tar.gz",
        "--file-description", "Linux 64-bit Native Binary, Desktop Launcher & App Icon",
        "--non-interactive",
        "--json"
    ]

    res = subprocess.run(cmd, capture_output=True, text=True)
    if res.returncode == 0:
        print("[SUCCESS] LaserBurn successfully updated to v1.1.0 on Gumroad!")
        print(f"Product URL: https://shop.pietiniart.com/l/laserburn")
    else:
        print(f"[ERROR] Failed to update Gumroad: {res.stderr}")
        sys.exit(1)

if __name__ == "__main__":
    main()
