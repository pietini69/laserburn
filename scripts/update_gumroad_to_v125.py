#!/usr/bin/env python3
"""
update_gumroad_to_v125.py
Updates LaserBurn on Gumroad to v1.2.5 with the latest binaries,
release notes, and detailed version-by-version changelog section.
"""

import os
import sys
import json
import subprocess

PROJECT_DIR = "/Volumes/Dataa/Ohjelmointi/ProjektitRust/laser_burn"
DIST_DIR = os.path.join(PROJECT_DIR, "dist")
PRODUCT_ID = "1SXqccPmq2GJB3uOomjR3Q=="

mac_dmg = os.path.join(DIST_DIR, "LaserBurn-1.2.5-universal.dmg")
win_zip = os.path.join(DIST_DIR, "LaserBurn-1.2.5-windows-x64.zip")
lin_tar = os.path.join(DIST_DIR, "LaserBurn-1.2.5-linux-x86_64.tar.gz")

DESCRIPTION_HTML = """
<h2>🔥 Meet LaserBurn v1.2.5: High-Performance Laser CAM & CNC Software</h2>
<p>
LaserBurn is a modern, blazing-fast laser cutting and engraving CAM application engineered from the ground up in <strong>Rust</strong>. Designed for makers, craft businesses, ceramic tile engravers, and high-precision fabrication workshops, LaserBurn delivers sub-second vector path optimization, instant project loading, and full multi-controller hardware control — with <strong>no subscriptions and no bloat</strong>.
</p>
<p>
<strong>Free / Pay What You Want (0 €+):</strong> Download and use LaserBurn completely free, or support independent development with the price of a cup of coffee (3 € – 5 €)!<br>
<strong>All 3 Desktop Platforms Included:</strong> macOS (Universal 2 for Apple Silicon & Intel), Windows 10/11 (64-bit), and Linux (x86_64).
</p>

<hr>

<h3>🚀 What's New in Version 1.2.5</h3>
<ul>
  <li><strong>🌐 Web Remote & iPad / Tablet Support:</strong> Control your laser cutter wire-free from an iPad, Android tablet, phone, or any browser on your local network. Features touch-first responsive controls, interactive 2D bed preview, jog D-Pad, live coordinate streaming, file upload (.gcode, .lbrn2, .svg), and instant E-Stop.</li>
  <li><strong>🍓 Raspberry Pi Headless Bridge:</strong> Run LaserBurn headlessly on a Raspberry Pi connected directly to your laser via USB, and manage everything over Wi-Fi without needing a tethered computer in your workshop.</li>
  <li><strong>📷 Optional Camera Support (Distraction-Free Minimal UI):</strong> Don't have an overhead camera? Turn off camera support in Settings (<code>Cmd+,</code> / <code>Ctrl+,</code>) to completely hide all camera buttons, menus, and shortcuts, leaving a 100% clean, minimal CAD/CAM layout.</li>
  <li><strong>🔍 Job Preview & Cut Simulation:</strong> Interactive toolpath simulator with animated laser head, scrub timeline, accurate time estimation, and color-coded rapid traverse vs cutting/engraving moves.</li>
  <li><strong>📏 Precision Dual Units:</strong> Flawless real-time switching between Metric (mm) and Imperial (in) across rulers, transform controls, status bar coordinates, and jog steps.</li>
</ul>

<hr>

<h3>📜 Versiohistoria & Muutosloki (Full Version History)</h3>

<h4>Version 1.2.5 (2026-10-27)</h4>
<ul>
  <li>Added <strong>Web Remote & iPad PWA Support</strong>: zero-dependency HTTP/1.1 & Server-Sent Events (SSE) server for responsive browser control over Wi-Fi.</li>
  <li>Added <strong>Raspberry Pi Headless Bridge</strong> (<code>laserburn-bridge</code>) & ready-to-flash Raspberry Pi OS image.</li>
  <li>Added <strong>Optional Camera Mode</strong> in Settings to completely hide camera buttons and menus for a clean, minimal workspace.</li>
  <li>Added <strong>Interactive Job Preview & Cut Simulation</strong> with timeline scrubbing and time estimation.</li>
  <li>Improved <strong>Imperial (Inches) Display</strong> across status bar and jog controls.</li>
</ul>

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
1. Open <code>LaserBurn-1.2.5-universal.dmg</code> and drag the <strong>LaserBurn</strong> icon into your <em>Applications</em> folder.<br>
2. <strong>First Launch (Gatekeeper Security):</strong><br>
Right-click (or Control-click) <code>LaserBurn.app</code> in your Applications folder and select <strong>Open</strong>, then click <strong>Open</strong> in the prompt. (Or go to System Settings → Privacy & Security → click "Open Anyway").
</p>

<h4>🪟 Windows:</h4>
<p>
1. Extract <code>LaserBurn-1.2.5-windows-x64.zip</code> into any folder (e.g. <code>C:\\LaserBurn</code>).<br>
2. Double-click <code>LaserBurn.exe</code> to launch.<br>
3. <strong>First Launch (SmartScreen):</strong> Click "More info" → "Run anyway".
</p>

<h4>🐧 Linux:</h4>
<p>
1. Extract <code>LaserBurn-1.2.5-linux-x86_64.tar.gz</code> and run <code>./LaserBurn</code>.<br>
2. Ensure your user belongs to the serial group: <code>sudo usermod -aG dialout $USER</code>.
</p>

<hr>

<p><em>LaserBurn v1.2.5. Developed by Niko Pietinen. Free updates and continuous improvements included.</em></p>
""".strip()

SUMMARY = "High-performance, modern laser CAM & CNC software built in Rust (v1.2.5). Features Web Remote for iPad/Tablets, Raspberry Pi bridge, Centerline Trace, dual Imperial/Metric units, lockable layers, preview simulation, LightBurn .lbrn2 XML compatibility & optional camera mode for macOS, Windows & Linux."

def main():
    for f in [mac_dmg, win_zip, lin_tar]:
        if not os.path.exists(f):
            print(f"Error: Required binary not found: {f}", file=sys.stderr)
            sys.exit(1)

    print(f"==> Updating LaserBurn to v1.2.5 on Gumroad (Product ID: {PRODUCT_ID})...")
    cmd = [
        "gumroad", "products", "update", PRODUCT_ID,
        "--name", "LaserBurn 1.2.5 – Modern Laser CAM & CNC Software (macOS, Windows, Linux)",
        "--custom-summary", SUMMARY,
        "--description", DESCRIPTION_HTML,
        "--file", mac_dmg,
        "--file-name", "LaserBurn-1.2.5-macOS-Universal.dmg",
        "--file-description", "macOS Universal 2 Installer (Apple Silicon M1-M4 & Intel 64-bit). macOS 11.0+",
        "--file", win_zip,
        "--file-name", "LaserBurn-1.2.5-Windows-x64.zip",
        "--file-description", "Windows 10 / 11 64-bit Standalone Executable & Package",
        "--file", lin_tar,
        "--file-name", "LaserBurn-1.2.5-Linux-x86_64.tar.gz",
        "--file-description", "Linux 64-bit x86_64 Standalone Tarball (glibc 2.31+)"
    ]

    res = subprocess.run(cmd, capture_output=True, text=True)
    if res.returncode != 0:
        print(f"Error updating product: {res.stderr}", file=sys.stderr)
        sys.exit(res.returncode)

    print("==> Base product metadata updated successfully!")

    # Set clean customer content download cards
    content_text = """
Thank you for downloading LaserBurn v1.2.5!

Choose the download package for your operating system:

• macOS: Download LaserBurn-1.2.5-macOS-Universal.dmg (Supports both Apple Silicon M1/M2/M3/M4 and Intel Macs). Drag to Applications and open!
• Windows: Download LaserBurn-1.2.5-Windows-x64.zip (Windows 10 & 11 64-bit). Extract and run LaserBurn.exe!
• Linux: Download LaserBurn-1.2.5-Linux-x86_64.tar.gz (Ubuntu, Debian, Fedora, Arch). Extract and run ./LaserBurn!

New in v1.2.5:
• Web Remote for iPad, tablet, and mobile browsers over Wi-Fi
• Headless Raspberry Pi Bridge & ready-to-flash disk image
• Distraction-free clean UI mode: toggle off camera buttons and menus in Settings
• Interactive Job Preview & Cut Simulation with speed timeline and time estimation

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
                "version": "1.2.5",
                "name": "LaserBurn 1.2.5 – Modern Laser CAM & CNC Software (macOS, Windows, Linux)",
                "updated_at": "2026-10-27T10:45:00+02:00",
                "files": [
                    "LaserBurn-1.2.5-universal.dmg",
                    "LaserBurn-1.2.5-windows-x64.zip",
                    "LaserBurn-1.2.5-linux-x86_64.tar.gz"
                ]
            }
            with open(tracking_path, "w", encoding="utf-8") as f:
                json.dump(data, f, indent=2, ensure_ascii=False)
            print("==> Local tracking file gumroad_published.json updated!")
        except Exception as e:
            print(f"Warning updating tracking JSON: {e}", file=sys.stderr)

    print("\n🎉 LaserBurn v1.2.5 is now LIVE on Gumroad!")

if __name__ == "__main__":
    main()
