#!/usr/bin/env python3
"""
update_gumroad_to_v128.py
Updates Gumroad product (1SXqccPmq2GJB3uOomjR3Q==) to PhotonCAM v1.2.8:
1. Uploads the latest v1.2.8 binaries (macOS Universal DMG, Windows x64 zip, Linux x86_64 tar.gz).
2. Updates product name, custom summary, tags, and HTML description.
3. Sets rich content download page so only the new v1.2.8 files are embedded.
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

mac_dmg = os.path.join(DIST_DIR, "PhotonCAM-1.2.8-universal.dmg")
win_zip = os.path.join(DIST_DIR, "PhotonCAM-1.2.8-windows-x64.zip")
lin_tar = os.path.join(DIST_DIR, "PhotonCAM-1.2.8-linux-x86_64.tar.gz")

DESCRIPTION_HTML = """
<h2>🔥 Meet PhotonCAM v1.2.8: High-Performance Laser CAM & 3D Sliced Relief Engraving</h2>
<p>
<strong>PhotonCAM</strong> is a modern, blazing-fast laser cutting and engraving CAM application engineered from the ground up in <strong>Rust</strong>. Designed for makers, craft businesses, woodworkers, ceramic tile engravers, and high-precision fabrication workshops, PhotonCAM delivers sub-second vector path optimization, instant project loading, and full multi-controller hardware control — with <strong>no subscriptions and no bloat</strong>.
</p>
<p>
<strong>Free / Pay What You Want (0 €+):</strong> Download and use PhotonCAM completely free, or support independent development with the price of a cup of coffee (3 € – 5 €)!<br>
<strong>All 3 Desktop Platforms Included:</strong> macOS (Universal 2 for Apple Silicon & Intel), Windows 10/11 (64-bit), and Linux (x86_64).
</p>

<hr>

<h3>🚀 What's New in Version 1.2.8</h3>
<ul>
  <li><strong>🧊 3D Sliced Relief Engraving (Monikerroksinen Korkokuva):</strong> Progressive multi-pass heightmap slicing engine for wood, stone, slate, and acrylic. Slices depth progressively from top to bottom, featuring orthogonal cross-pass angles (0°/90°) to completely eliminate banding and ridges.</li>
  <li><strong>🛡️ Material Thickness & Collision Prevention:</strong> Built-in punch-through protection (configured for 3mm plywood and custom stock) with automatic safety depth clamping. Automated Z-axis safety retract plane (Retract Clearance Z) prevents the air-assist nozzle from colliding with higher workpiece areas during rapid travel!</li>
  <li><strong>✏️ Vector Shape Fill & Hatch Controls:</strong> Direct selection between <strong>Viiva (Line)</strong>, <strong>Täyttö (Fill)</strong>, and <strong>Täyttö + Viiva (Fill + Line)</strong> across rectangles, circles, and polygons. Presets for scan angle (Vaaka 0°, Pysty 90°, 45°) and bi-directional Cross-Hatch with configurable line interval.</li>
  <li><strong>📱 Modern iPad & Web-CAD Remote:</strong> Real-time canvas hatch pattern and semi-transparent fill preview, automatic QR code module fills, 0,0 machine origin corner synchronization, parametric finger-joint box maker, and living hinge generator.</li>
  <li><strong>🍓 Raspberry Pi Wireless Bridge:</strong> Connect your desktop over Wi-Fi/Ethernet directly to the Raspberry Pi bridge (port 2323) or control the laser from any tablet browser (port 8080).</li>
  <li><strong>🔍 Job Preview & Cut Simulation:</strong> Interactive toolpath simulator with animated laser head, scrub timeline, accurate time estimation, and color-coded rapid traverse vs cutting/engraving moves.</li>
</ul>

<hr>

<h3>📜 Versiohistoria & Muutosloki (Full Version History)</h3>

<h4>Version 1.2.8 (2026-10-31)</h4>
<ul>
  <li>Added <strong>3D Sliced Relief Engraving</strong> with Z-axis collision prevention and material punch-through safety.</li>
  <li>Added <strong>Vector Fill & Hatch</strong> controls (Line, Fill, Fill+Line) to desktop and Web-CAD canvas.</li>
  <li>Fixed hatch toolpath generation parameter order in CAM optimizer.</li>
  <li>Enhanced SVG parser with automatic fill color and metadata detection.</li>
</ul>

<h4>Version 1.2.7 (2026-10-29)</h4>
<ul>
  <li>Major Web-CAD upgrade: 0,0 Origin selector, parametric finger-joint box maker, living hinge and QR generators.</li>
  <li>Raspberry Pi persistent systemd boot auto-start and ready-to-flash ARM64 OS image.</li>
</ul>

<h4>Version 1.2.6 (2026-10-28)</h4>
<ul>
  <li>Added <strong>Network (TCP) Connection</strong> to Raspberry Pi Bridge in PhotonCAM desktop connection dialog.</li>
  <li>Added <strong>Bidirectional Real-Time Communication</strong> in <code>photoncam-bridge</code> for external CAD/CAM software.</li>
</ul>

<h4>Version 1.2.5 (2026-10-27) – Rebranded to PhotonCAM</h4>
<ul>
  <li>Rebranded to PhotonCAM for unique identity and global brand protection.</li>
  <li>Added Web Remote & iPad PWA Support with zero-dependency embedded server.</li>
</ul>

<hr>

<h3>💻 Installation & Quick Start</h3>

<h4>🍎 macOS:</h4>
<p>
1. Open <code>PhotonCAM-1.2.8-universal.dmg</code> and drag the <strong>PhotonCAM</strong> icon into your <em>Applications</em> folder.<br>
2. <strong>First Launch (Gatekeeper Security):</strong><br>
Right-click (or Control-click) <code>PhotonCAM.app</code> in your Applications folder and select <strong>Open</strong>, then click <strong>Open</strong> in the prompt. (Or go to System Settings → Privacy & Security → click "Open Anyway").
</p>

<h4>🪟 Windows:</h4>
<p>
1. Extract <code>PhotonCAM-1.2.8-windows-x64.zip</code> into any folder (e.g. <code>C:\\PhotonCAM</code>).<br>
2. Double-click <code>PhotonCAM.exe</code> to launch.<br>
3. <strong>First Launch (SmartScreen):</strong> Click "More info" → "Run anyway".
</p>

<h4>🐧 Linux:</h4>
<p>
1. Extract <code>PhotonCAM-1.2.8-linux-x86_64.tar.gz</code> and run <code>./PhotonCAM</code>.<br>
2. Ensure your user belongs to the serial group: <code>sudo usermod -aG dialout $USER</code>.
</p>

<h4>🍓 Raspberry Pi (Wireless Laser Server):</h4>
<p>
To run a wireless laser server on your Raspberry Pi:<br>
<code>curl -sSL https://raw.githubusercontent.com/pietini69/photoncam/master/scripts/install_pi_bridge.sh | sudo bash</code><br>
Or download the pre-configured OS disk image from <a href="https://github.com/pietini69/photoncam/releases/tag/v1.2.8">GitHub Releases</a>.
</p>

<hr>

<p><em>PhotonCAM v1.2.8. Developed by Niko Pietinen. Free updates and continuous improvements included.</em></p>
""".strip()

SUMMARY = "High-performance, modern laser CAM & CNC software built in Rust (v1.2.8). Features 3D Sliced Relief engraving with Z-collision prevention, vector fill & cross-hatch, wireless Raspberry Pi bridge & iPad Web-CAD, LightBurn .lbrn2 compatibility, for macOS, Windows & Linux."

def main():
    for f in [mac_dmg, win_zip, lin_tar]:
        if not os.path.exists(f):
            print(f"Error: Required binary not found: {f}", file=sys.stderr)
            sys.exit(1)

    print(f"==> Updating product on Gumroad to PhotonCAM v1.2.8 (Product ID: {PRODUCT_ID})...")
    cmd = [
        "gumroad", "products", "update", PRODUCT_ID,
        "--name", "PhotonCAM 1.2.8 – Modern Laser CAM & CNC Software (macOS, Windows, Linux, Raspberry Pi)",
        "--custom-summary", SUMMARY,
        "--description", DESCRIPTION_HTML,
        "--file", mac_dmg,
        "--file-name", "PhotonCAM-1.2.8-macOS-Universal.dmg",
        "--file-description", "macOS Universal 2 Installer (Apple Silicon M1-M4 & Intel 64-bit). macOS 11.0+",
        "--file", win_zip,
        "--file-name", "PhotonCAM-1.2.8-Windows-x64.zip",
        "--file-description", "Windows 10 / 11 64-bit Standalone Executable & Package",
        "--file", lin_tar,
        "--file-name", "PhotonCAM-1.2.8-Linux-x86_64.tar.gz",
        "--file-description", "Linux 64-bit x86_64 Standalone Tarball (glibc 2.31+)"
    ]

    res = subprocess.run(cmd, capture_output=True, text=True)
    if res.returncode != 0:
        print(f"Error updating product: {res.stderr}", file=sys.stderr)
        sys.exit(res.returncode)

    print("==> Base product metadata and files uploaded successfully!")

    # Now inspect files to find the newly uploaded file IDs for v1.2.8
    view_res = subprocess.run(["gumroad", "products", "view", PRODUCT_ID, "--json"], capture_output=True, text=True)
    if view_res.returncode != 0:
        print(f"Error viewing product: {view_res.stderr}", file=sys.stderr)
        sys.exit(view_res.returncode)

    prod_json = json.loads(view_res.stdout)
    all_files = prod_json.get("product", {}).get("files", [])

    # Find the latest files matching 1.2.8
    v128_mac = None
    v128_win = None
    v128_lin = None

    for f in all_files:
        name = f.get("name", "")
        fid = f.get("id")
        if "PhotonCAM-1.2.8-macOS-Universal.dmg" in name:
            v128_mac = fid
        elif "PhotonCAM-1.2.8-Windows-x64.zip" in name:
            v128_win = fid
        elif "PhotonCAM-1.2.8-Linux-x86_64.tar.gz" in name:
            v128_lin = fid

    print(f"==> Identified v1.2.8 file IDs: Mac={v128_mac}, Win={v128_win}, Linux={v128_lin}")

    keep_ids = [fid for fid in [v128_mac, v128_win, v128_lin] if fid]
    if keep_ids:
        # Build clean rich content with only v1.2.8 files
        doc_content = [
            {
                "type": "heading",
                "attrs": {"level": 2},
                "content": [{"type": "text", "text": "PhotonCAM v1.2.8 Downloads"}]
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
                        "marks": [{"type": "link", "attrs": {"href": "https://github.com/pietini69/photoncam/releases/tag/v1.2.8"}}],
                        "text": "GitHub Releases v1.2.8"
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
                "title": "PhotonCAM v1.2.8",
                "variant_id": None,
                "description": {
                    "type": "doc",
                    "content": doc_content
                }
            }
        ]

        tmp_content_path = "/tmp/gumroad_content_v128.json"
        with open(tmp_content_path, "w", encoding="utf-8") as f:
            json.dump(content_page, f)

        set_res = subprocess.run(["gumroad", "products", "content", "set", PRODUCT_ID, tmp_content_path, "--yes"], capture_output=True, text=True)
        if set_res.returncode == 0:
            print("==> Rich content updated with clean v1.2.8 downloads!")
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
                "version": "1.2.8",
                "name": "PhotonCAM 1.2.8 – Modern Laser CAM & CNC Software (macOS, Windows, Linux, Raspberry Pi)",
                "updated_at": "2026-10-31T08:44:00+02:00",
                "url": "https://shop.pietiniart.com/l/photoncam",
                "files": [
                    "PhotonCAM-1.2.8-universal.dmg",
                    "PhotonCAM-1.2.8-windows-x64.zip",
                    "PhotonCAM-1.2.8-linux-x86_64.tar.gz"
                ]
            }
            with open(tracking_path, "w", encoding="utf-8") as f:
                json.dump(data, f, indent=2, ensure_ascii=False)
            print("==> Local tracking file gumroad_published.json updated!")
        except Exception as e:
            print(f"Warning updating tracking JSON: {e}", file=sys.stderr)

    print("\n🎉 PhotonCAM v1.2.8 is now LIVE on Gumroad!")

if __name__ == "__main__":
    main()
