#!/usr/bin/env python3
"""
update_gumroad_to_v126.py
Updates Gumroad product (1SXqccPmq2GJB3uOomjR3Q==) to PhotonCAM v1.2.6:
1. Uploads the latest v1.2.6 binaries (macOS Universal DMG, Windows x64 zip, Linux x86_64 tar.gz).
2. Updates product name, custom summary, tags, and HTML description.
3. Sets rich content download page so only the new v1.2.6 files are embedded.
4. Updates local tracking JSON.
"""

import os
import sys
import json
import uuid
import subprocess

PROJECT_DIR = "/Volumes/Dataa/Ohjelmointi/ProjektitRust/laser_burn"
DIST_DIR = os.path.join(PROJECT_DIR, "dist")
PRODUCT_ID = "1SXqccPmq2GJB3uOomjR3Q=="

mac_dmg = os.path.join(DIST_DIR, "PhotonCAM-1.2.6-universal.dmg")
win_zip = os.path.join(DIST_DIR, "PhotonCAM-1.2.6-windows-x64.zip")
lin_tar = os.path.join(DIST_DIR, "PhotonCAM-1.2.6-linux-x86_64.tar.gz")

DESCRIPTION_HTML = """
<h2>🔥 Meet PhotonCAM v1.2.6: High-Performance Laser CAM & Wireless Web Remote</h2>
<p>
<strong>PhotonCAM</strong> is a modern, blazing-fast laser cutting and engraving CAM application engineered from the ground up in <strong>Rust</strong>. Designed for makers, craft businesses, ceramic tile engravers, and high-precision fabrication workshops, PhotonCAM delivers sub-second vector path optimization, instant project loading, and full multi-controller hardware control — with <strong>no subscriptions and no bloat</strong>.
</p>
<p>
<strong>Free / Pay What You Want (0 €+):</strong> Download and use PhotonCAM completely free, or support independent development with the price of a cup of coffee (3 € – 5 €)!<br>
<strong>All 3 Desktop Platforms Included:</strong> macOS (Universal 2 for Apple Silicon & Intel), Windows 10/11 (64-bit), and Linux (x86_64).
</p>

<hr>

<h3>🚀 What's New in Version 1.2.6</h3>
<ul>
  <li><strong>🌐 Network (TCP) Connection to Raspberry Pi:</strong> Connect directly to your laser cutter over Wi-Fi/Ethernet from PhotonCAM desktop. In the Connection window, select <strong>Network (Raspberry Pi)</strong> and enter your Pi's hostname or IP (e.g. <code>photoncam.local</code> or <code>192.168.68.73:2323</code>). All jogging, framing, job streaming, and status monitoring work seamlessly over the network!</li>
  <li><strong>🔌 Bidirectional Bridge Protocol:</strong> The Raspberry Pi headless bridge (<code>photoncam-bridge</code>) now fully relays real-time bytes (<code>?</code>, <code>!</code>, <code>~</code>, soft-reset <code>0x18</code>) and streams all laser controller output back to connected desktop clients in real time.</li>
  <li><strong>🍓 Raspberry Pi Web Remote & Headless Bridge:</strong> Control your laser cutter wire-free from an iPad, Android tablet, phone, or any browser on your local network. Features touch-first responsive controls, interactive 2D bed preview, jog D-Pad, live coordinate streaming, file upload (.gcode, .lbrn2, .svg), and instant E-Stop.</li>
  <li><strong>🎯 4-Quadrant Machine Origin:</strong> Built-in visual mapping and jog compensation for all homing configurations (Takaoikea / Top-Right default for Ortur, xTool, Sculpfun, Atomstack, Two Trees, Neje).</li>
  <li><strong>📷 Optional Camera Support (Distraction-Free Minimal UI):</strong> Don't have an overhead camera? Turn off camera support in Settings (<code>Cmd+,</code> / <code>Ctrl+,</code>) to completely hide all camera buttons, menus, and shortcuts, leaving a 100% clean, minimal CAD/CAM layout.</li>
  <li><strong>🔍 Job Preview & Cut Simulation:</strong> Interactive toolpath simulator with animated laser head, scrub timeline, accurate time estimation, and color-coded rapid traverse vs cutting/engraving moves.</li>
  <li><strong>📏 Precision Dual Units:</strong> Flawless real-time switching between Metric (mm) and Imperial (in) across rulers, transform controls, status bar coordinates, and jog steps.</li>
</ul>

<hr>

<h3>📜 Versiohistoria & Muutosloki (Full Version History)</h3>

<h4>Version 1.2.6 (2026-10-28)</h4>
<ul>
  <li>Added <strong>Network (TCP) Connection</strong> to Raspberry Pi Bridge in PhotonCAM desktop connection dialog.</li>
  <li>Added <strong>Bidirectional Real-Time Communication</strong> in <code>photoncam-bridge</code> for external CAD/CAM software.</li>
  <li>Improved installer script and headless systemd service automation.</li>
</ul>

<h4>Version 1.2.5 (2026-10-27) – Rebranded to PhotonCAM</h4>
<ul>
  <li><strong>Rebranded to PhotonCAM</strong> for unique identity and global brand protection.</li>
  <li>Added <strong>Web Remote & iPad PWA Support</strong>: zero-dependency HTTP/1.1 & Server-Sent Events (SSE) server for responsive browser control over Wi-Fi.</li>
  <li>Added <strong>Raspberry Pi Headless Bridge</strong> (<code>photoncam-bridge</code>) & ready-to-flash Raspberry Pi OS image.</li>
  <li>Added <strong>4-Quadrant Machine Origin</strong> setup modal with inverted X/Y jog mapping.</li>
</ul>

<hr>

<h3>💻 Installation & Quick Start</h3>

<h4>🍎 macOS:</h4>
<p>
1. Open <code>PhotonCAM-1.2.6-universal.dmg</code> and drag the <strong>PhotonCAM</strong> icon into your <em>Applications</em> folder.<br>
2. <strong>First Launch (Gatekeeper Security):</strong><br>
Right-click (or Control-click) <code>PhotonCAM.app</code> in your Applications folder and select <strong>Open</strong>, then click <strong>Open</strong> in the prompt. (Or go to System Settings → Privacy & Security → click "Open Anyway").
</p>

<h4>🪟 Windows:</h4>
<p>
1. Extract <code>PhotonCAM-1.2.6-windows-x64.zip</code> into any folder (e.g. <code>C:\\PhotonCAM</code>).<br>
2. Double-click <code>PhotonCAM.exe</code> to launch.<br>
3. <strong>First Launch (SmartScreen):</strong> Click "More info" → "Run anyway".
</p>

<h4>🐧 Linux:</h4>
<p>
1. Extract <code>PhotonCAM-1.2.6-linux-x86_64.tar.gz</code> and run <code>./PhotonCAM</code>.<br>
2. Ensure your user belongs to the serial group: <code>sudo usermod -aG dialout $USER</code>.
</p>

<h4>🍓 Raspberry Pi (Wireless Laser Server):</h4>
<p>
To run a wireless laser server on your Raspberry Pi:<br>
<code>curl -sSL https://raw.githubusercontent.com/pietini69/photoncam/master/scripts/install_pi_bridge.sh | sudo bash</code><br>
Or download the pre-configured OS disk image from <a href="https://github.com/pietini69/photoncam/releases/tag/v1.2.6">GitHub Releases</a>.
</p>

<hr>

<p><em>PhotonCAM v1.2.6. Developed by Niko Pietinen. Free updates and continuous improvements included.</em></p>
""".strip()

SUMMARY = "High-performance, modern laser CAM & CNC software built in Rust (v1.2.6). Features wireless Raspberry Pi network connection, Web Remote for iPad/Tablets, Centerline Trace, dual Imperial/Metric units, lockable layers, preview simulation, LightBurn .lbrn2 XML compatibility & optional camera mode for macOS, Windows & Linux."

def main():
    for f in [mac_dmg, win_zip, lin_tar]:
        if not os.path.exists(f):
            print(f"Error: Required binary not found: {f}", file=sys.stderr)
            sys.exit(1)

    print(f"==> Updating product on Gumroad to PhotonCAM v1.2.6 (Product ID: {PRODUCT_ID})...")
    cmd = [
        "gumroad", "products", "update", PRODUCT_ID,
        "--name", "PhotonCAM 1.2.6 – Modern Laser CAM & CNC Software (macOS, Windows, Linux, Raspberry Pi)",
        "--custom-summary", SUMMARY,
        "--description", DESCRIPTION_HTML,
        "--file", mac_dmg,
        "--file-name", "PhotonCAM-1.2.6-macOS-Universal.dmg",
        "--file-description", "macOS Universal 2 Installer (Apple Silicon M1-M4 & Intel 64-bit). macOS 11.0+",
        "--file", win_zip,
        "--file-name", "PhotonCAM-1.2.6-Windows-x64.zip",
        "--file-description", "Windows 10 / 11 64-bit Standalone Executable & Package",
        "--file", lin_tar,
        "--file-name", "PhotonCAM-1.2.6-Linux-x86_64.tar.gz",
        "--file-description", "Linux 64-bit x86_64 Standalone Tarball (glibc 2.31+)"
    ]

    res = subprocess.run(cmd, capture_output=True, text=True)
    if res.returncode != 0:
        print(f"Error updating product: {res.stderr}", file=sys.stderr)
        sys.exit(res.returncode)

    print("==> Base product metadata and files uploaded successfully!")

    # Now inspect files to find the newly uploaded file IDs for v1.2.6
    view_res = subprocess.run(["gumroad", "products", "view", PRODUCT_ID, "--json"], capture_output=True, text=True)
    if view_res.returncode != 0:
        print(f"Error viewing product: {view_res.stderr}", file=sys.stderr)
        sys.exit(view_res.returncode)

    prod_json = json.loads(view_res.stdout)
    all_files = prod_json.get("product", {}).get("files", [])

    # Find the latest files matching 1.2.6
    v126_mac = None
    v126_win = None
    v126_lin = None

    for f in all_files:
        name = f.get("name", "")
        fid = f.get("id")
        if "PhotonCAM-1.2.6-macOS-Universal.dmg" in name:
            v126_mac = fid
        elif "PhotonCAM-1.2.6-Windows-x64.zip" in name:
            v126_win = fid
        elif "PhotonCAM-1.2.6-Linux-x86_64.tar.gz" in name:
            v126_lin = fid

    print(f"==> Identified v1.2.6 file IDs: Mac={v126_mac}, Win={v126_win}, Linux={v126_lin}")

    keep_ids = [fid for fid in [v126_mac, v126_win, v126_lin] if fid]
    if keep_ids:
        # Build clean rich content with only v1.2.6 files
        doc_content = [
            {
                "type": "heading",
                "attrs": {"level": 2},
                "content": [{"type": "text", "text": "PhotonCAM v1.2.6 Downloads"}]
            },
            {
                "type": "paragraph",
                "content": [
                    {
                        "type": "text",
                        "text": "Download the build for your operating system (macOS, Windows, Linux). For Raspberry Pi OS pre-configured SD card image and wireless bridge: "
                    },
                    {
                        "type": "text",
                        "marks": [{"type": "link", "attrs": {"href": "https://github.com/pietini69/photoncam/releases/tag/v1.2.6"}}],
                        "text": "GitHub Releases v1.2.6"
                    }
                ]
            }
        ]

        for fid in keep_ids:
            doc_content.append({
                "type": "fileEmbed",
                "attrs": {"id": fid, "uid": str(uuid.uuid4()), "collapsed": False}
            })

        content_page = [
            {
                "id": "sBSkbwbvuIfzxLIvwrrH3A==",
                "page_id": "sBSkbwbvuIfzxLIvwrrH3A==",
                "title": "PhotonCAM v1.2.6",
                "variant_id": None,
                "description": {
                    "type": "doc",
                    "content": doc_content
                }
            }
        ]

        tmp_content_path = "/tmp/gumroad_content_v126.json"
        with open(tmp_content_path, "w", encoding="utf-8") as f:
            json.dump(content_page, f)

        set_res = subprocess.run(["gumroad", "products", "content", "set", PRODUCT_ID, tmp_content_path, "--yes"], capture_output=True, text=True)
        if set_res.returncode == 0:
            print("==> Rich content updated with clean v1.2.6 downloads!")
        else:
            print(f"Warning setting content: {set_res.stderr}", file=sys.stderr)

    # Update local tracking JSON
    tracking_path = "/Volumes/Dataa/EtsyTuotteet/gumroad_published.json"
    if os.path.exists(tracking_path):
        try:
            with open(tracking_path, "r", encoding="utf-8") as f:
                data = json.load(f)
            data["photoncam"] = {
                "product_id": PRODUCT_ID,
                "version": "1.2.6",
                "name": "PhotonCAM 1.2.6 – Modern Laser CAM & CNC Software (macOS, Windows, Linux, Raspberry Pi)",
                "updated_at": "2026-10-28T21:20:00+02:00",
                "url": "https://shop.pietiniart.com/l/photoncam",
                "files": [
                    "PhotonCAM-1.2.6-universal.dmg",
                    "PhotonCAM-1.2.6-windows-x64.zip",
                    "PhotonCAM-1.2.6-linux-x86_64.tar.gz"
                ]
            }
            with open(tracking_path, "w", encoding="utf-8") as f:
                json.dump(data, f, indent=2, ensure_ascii=False)
            print("==> Local tracking file gumroad_published.json updated!")
        except Exception as e:
            print(f"Warning updating tracking JSON: {e}", file=sys.stderr)

    print("\n🎉 PhotonCAM v1.2.6 is now LIVE on Gumroad!")

if __name__ == "__main__":
    main()
