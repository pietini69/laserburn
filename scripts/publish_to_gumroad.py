#!/usr/bin/env python3
"""
publish_to_gumroad.py
Creates the official LaserBurn multi-platform product on Gumroad with:
- macOS Universal 2 (.dmg)
- Windows x64 (.zip)
- Linux x86_64 (.tar.gz)
- Cover image & 1000x1000 square thumbnail
- Rich HTML sales copy, quick start guide, system requirements & installation instructions
"""

import os
import sys
import subprocess
import argparse

PROJECT_DIR = "/Volumes/Dataa/Ohjelmointi/ProjektitRust/laser_burn"
DIST_DIR = os.path.join(PROJECT_DIR, "dist")
ASSETS_DIR = os.path.join(DIST_DIR, "gumroad_assets")

PRODUCT_NAME = "LaserBurn – Modern Laser CAM & CNC Software (macOS, Windows, Linux)"
CUSTOM_PERMALINK = "laserburn"
CURRENCY = "eur"
PRICE = "0"  # Pay What You Want (0€+) or set a fixed price e.g. "19"
SUGGESTED_PRICE = "15"

TAGS = [
    "laser cutting",
    "laser engraving",
    "cam software",
    "lightburn",
    "grbl",
    "ruida",
    "cnc",
    "macos",
    "windows",
    "linux"
]

SUMMARY = "High-performance, modern laser cutting & engraving CAM software built in Rust. LightBurn .lbrn2 compatible, advanced dithering, camera overlay & path optimizer for macOS, Windows and Linux."

DESCRIPTION_HTML = """
<h2>🔥 Meet LaserBurn: High-Performance Laser CAM & CNC Software</h2>
<p>
LaserBurn is a brand-new, modern laser cutting and engraving CAM application engineered from the ground up in <strong>Rust</strong>. Designed for hobbyists, craft businesses, and high-precision fabrication workshops, LaserBurn delivers blazing-fast vector path optimization, instant project loading, and multi-controller hardware control without subscriptions or bloat.
</p>
<p>
<strong>One purchase gives you full access to all three desktop operating systems:</strong> macOS (Universal 2 for Apple Silicon & Intel), Windows 10/11 (64-bit), and Linux (x86_64).
</p>

<hr>

<h3>⚡ Key Features</h3>
<ul>
  <li><strong>📂 LightBurn .lbrn2 Project Compatibility:</strong> Open and save modern LightBurn project files directly, including vector paths, layer assignments, cut speeds, power settings, and multi-pass rules.</li>
  <li><strong>🚀 Ultra-Fast Rust Engine:</strong> Sub-second calculations even on massive vectors with millions of points. Instant 60 FPS pan, zoom, and transform response.</li>
  <li><strong>🧠 Smart Path Optimizer:</strong> Traveling Salesperson Problem (TSP) solver and directional sort algorithms minimize rapid travel time and prevent material burning.</li>
  <li><strong>🎚️ Multi-Layer Presets & Materials:</strong> Assign custom names, colors, speeds (mm/min), power (%), and kerf offset compensation (inner/outer contour expansion).</li>
  <li><strong>📷 Live Camera Bed Overlay:</strong> Connect any USB webcam or overhead machine camera. Perspective undistortion and 4-point calibration allow you to align designs directly over scraps and wood blanks with millimeter precision.</li>
  <li><strong>🖼️ Studio-Grade Image Dithering:</strong> 8 advanced dithering algorithms for photorealistic wood, acrylic, slate, and leather engraving (Atkinson, Floyd-Steinberg, Jarvis-Judice-Ninke, Stucki, Burkes, Sierra3, and Halftone).</li>
  <li><strong>🖨️ Multi-Controller Support:</strong> Direct USB serial communication with <strong>GRBL</strong> (1.1+) diodes/CO2 lasers (Ortur, xTool, Sculpfun, Atomstack, Creality, Two Trees, Neje, generic 3018) and network/USB support for <strong>Ruida</strong> DSP controllers.</li>
  <li><strong>⚙️ Hardware Safety & Limit Switches:</strong> Mechanical limit switches, soft limits, custom learned boundaries, and emergency stop protection.</li>
</ul>

<hr>

<h3>📦 What’s Included in Your Download</h3>
<p>When you download LaserBurn, you receive immediate access to all three platform builds:</p>
<ol>
  <li><strong>macOS Universal 2 Installer (<code>LaserBurn-1.0.0-beta.1-universal.dmg</code>):</strong> Native performance on all Macs — Apple Silicon (M1/M2/M3/M4) and Intel (x86_64). Code-signed with Apple Developer certificate.</li>
  <li><strong>Windows 64-bit Standalone (<code>LaserBurn-1.0.0-beta.1-windows-x64.zip</code>):</strong> Portable standalone executable for Windows 10 & Windows 11. No complex installation or registry keys required.</li>
  <li><strong>Linux 64-bit Package (<code>LaserBurn-1.0.0-beta.1-linux-x86_64.tar.gz</code>):</strong> Native 64-bit ELF binary, system icon, and <code>.desktop</code> launcher for Ubuntu, Debian, Fedora, Arch, and Linux Mint.</li>
  <li><strong>Documentation & Quick Start Guide (PDF / TXT):</strong> Setup and controller connection guide.</li>
</ol>

<hr>

<h3>🚀 Quick Start Guide</h3>
<ol>
  <li><strong>Launch LaserBurn</strong> on your machine (macOS, Windows, or Linux).</li>
  <li><strong>Connect your Laser:</strong> Connect the USB cable from your laser cutter to your computer. In LaserBurn, open the <em>Device</em> panel, select your serial COM / tty port, and click <strong>Connect</strong>.</li>
  <li><strong>Import or Draw:</strong> Use <strong>File → Open</strong> to load a <code>.lbrn2</code>, <code>.svg</code>, or image file (PNG/JPG), or use the built-in shape and text tools.</li>
  <li><strong>Configure Layers:</strong> Select your vectors and assign them to cut or engrave layers. Set your speed, laser power %, and number of passes.</li>
  <li><strong>Frame & Run:</strong> Use the <strong>Frame (Bounding Box)</strong> tool with weak laser light enabled to verify placement on your material blank, then press <strong>Start Job</strong>.</li>
</ol>

<hr>

<h3>💻 Minimum System Requirements</h3>

<h4>🍎 macOS:</h4>
<ul>
  <li><strong>Operating System:</strong> macOS 11.0 (Big Sur), 12 (Monterey), 13 (Ventura), 14 (Sonoma), 15 (Sequoia) or newer.</li>
  <li><strong>Processor:</strong> Apple Silicon (M1/M2/M3/M4) or 64-bit Intel Core i5/i7/i9 / Xeon.</li>
  <li><strong>Memory:</strong> 4 GB RAM (8 GB recommended).</li>
  <li><strong>Disk Space:</strong> 50 MB free space.</li>
</ul>

<h4>🪟 Windows:</h4>
<ul>
  <li><strong>Operating System:</strong> Windows 10 (64-bit, v1809+) or Windows 11 (64-bit).</li>
  <li><strong>Processor:</strong> 64-bit Intel or AMD processor with SSE2 support.</li>
  <li><strong>Graphics:</strong> OpenGL 2.1 or DirectX 11 compatible GPU (integrated or dedicated).</li>
  <li><strong>Memory:</strong> 4 GB RAM (8 GB recommended).</li>
  <li><strong>Disk Space:</strong> 50 MB free space.</li>
</ul>

<h4>🐧 Linux:</h4>
<ul>
  <li><strong>Operating System:</strong> Any modern 64-bit Linux distribution with glibc 2.17+ (Ubuntu 18.04+, Debian 10+, Fedora 28+, Arch Linux, Linux Mint 19+).</li>
  <li><strong>Display Server:</strong> X11 or Wayland with Mesa OpenGL drivers.</li>
  <li><strong>Memory:</strong> 2–4 GB RAM.</li>
  <li><strong>Disk Space:</strong> 60 MB free space.</li>
</ul>

<hr>

<h3>🛠️ Installation Instructions</h3>

<h4>🍎 macOS:</h4>
<p>
1. Open <code>LaserBurn-1.0.0-beta.1-universal.dmg</code> and drag the <strong>LaserBurn</strong> icon into your <em>Applications</em> folder.<br>
2. <strong>First Launch (Gatekeeper Security):</strong><br>
Because LaserBurn is freshly packaged, macOS may show a security notice on first launch. To open it smoothly:<br>
• <strong>Right-click (or Control-click)</strong> <code>LaserBurn.app</code> in your Applications folder and select <strong>Open</strong>, then click <strong>Open</strong> in the prompt.<br>
• <em>Alternatively:</em> Open <strong>System Settings → Privacy & Security</strong>, scroll down to the Security section, and click <strong>"Open Anyway"</strong>.
</p>

<h4>🪟 Windows:</h4>
<p>
1. Extract <code>LaserBurn-1.0.0-beta.1-windows-x64.zip</code> into any folder (e.g. <code>C:\\LaserBurn</code>).<br>
2. Double-click <code>LaserBurn.exe</code> to launch.<br>
3. <strong>First Launch (Windows SmartScreen):</strong><br>
If the blue <em>"Windows protected your PC"</em> screen appears:<br>
• Click the link <strong>"More info"</strong> (Lisätietoja).<br>
• Click the button <strong>"Run anyway"</strong> (Suorita silti).<br>
<em>(This only needs to be clicked once; Windows will remember your choice).</em>
</p>

<h4>🐧 Linux:</h4>
<p>
1. Extract <code>LaserBurn-1.0.0-beta.1-linux-x86_64.tar.gz</code> and run <code>./LaserBurn</code>.<br>
2. <strong>USB Serial Port Permissions:</strong> To allow LaserBurn to communicate with your laser via USB, ensure your user belongs to the serial group:<br>
• Ubuntu / Debian / Mint: <code>sudo usermod -aG dialout $USER</code><br>
• Arch / Manjaro / Fedora: <code>sudo usermod -aG uucp $USER</code><br>
<em>(Log out and back in once for the group permission to take effect).</em>
</p>

<hr>

<p><em>LaserBurn v1.0.0 Beta. Developed by Niko Pietinen. Free updates and continuous improvements included.</em></p>
""".strip()

def main():
    parser = argparse.ArgumentParser(description="Publish or draft LaserBurn on Gumroad")
    parser.add_argument("--draft", action="store_true", help="Create as draft without publishing immediately")
    parser.add_argument("--dry-run", action="store_true", help="Print command without executing")
    parser.add_argument("--price", default=PRICE, help="Price in EUR (default: 0 for PWYW)")
    args = parser.parse_args()

    # Paths to files
    mac_dmg = os.path.join(DIST_DIR, "LaserBurn-1.0.0-beta.1-universal.dmg")
    win_zip = os.path.join(DIST_DIR, "LaserBurn-1.0.0-beta.1-windows-x64.zip")
    lin_tar = os.path.join(DIST_DIR, "LaserBurn-1.0.0-beta.1-linux-x86_64.tar.gz")

    cover_img = os.path.join(ASSETS_DIR, "gumroad_cover.jpg")
    thumb_img = os.path.join(ASSETS_DIR, "gumroad_thumb_square.jpg")

    for f in [mac_dmg, win_zip, lin_tar, cover_img, thumb_img]:
        if not os.path.exists(f):
            print(f"Error: Required file not found: {f}", file=sys.stderr)
            sys.exit(1)

    cmd = [
        "gumroad", "products", "create",
        "--name", PRODUCT_NAME,
        "--price", args.price,
        "--currency", CURRENCY,
        "--custom-permalink", CUSTOM_PERMALINK,
        "--custom-summary", SUMMARY,
        "--description", DESCRIPTION_HTML,
        "--cover-image", cover_img,
        "--thumbnail", thumb_img,
        "--pay-what-you-want",
        "--suggested-price", SUGGESTED_PRICE,
        # File 1: macOS Universal
        "--file", mac_dmg,
        "--file-name", "LaserBurn-1.0.0-beta.1-macOS-Universal.dmg",
        "--file-description", "macOS Universal 2 Installer (Apple Silicon M1-M4 & Intel 64-bit). macOS 11.0+",
        # File 2: Windows
        "--file", win_zip,
        "--file-name", "LaserBurn-1.0.0-beta.1-Windows-x64.zip",
        "--file-description", "Windows 10 & 11 (64-bit) Portable Standalone Executable",
        # File 3: Linux
        "--file", lin_tar,
        "--file-name", "LaserBurn-1.0.0-beta.1-Linux-x86_64.tar.gz",
        "--file-description", "Linux 64-bit Native Binary, Desktop Launcher & App Icon",
    ]

    for t in TAGS:
        cmd.extend(["--tag", t])

    if args.draft:
        cmd.append("--draft")

    if args.dry_run:
        print("Dry run mode. Command that would be executed:")
        print(" ".join(cmd[:15]) + " ... [files and description omitted for brevity]")
        return

    print("==> Creating LaserBurn product on Gumroad...")
    res = subprocess.run(cmd, capture_output=True, text=True)
    print("STDOUT:", res.stdout)
    if res.stderr:
        print("STDERR:", res.stderr)

    if res.returncode == 0:
        print("==> SUCCESS: LaserBurn created on Gumroad!")
    else:
        print(f"==> FAILED with code {res.returncode}")

if __name__ == "__main__":
    main()
