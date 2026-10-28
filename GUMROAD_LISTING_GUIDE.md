# 🔥 PhotonCAM – Gumroad Product Listing & Multi-Platform Launch Guide (v1.2.5)

> **Product:** PhotonCAM – Modern Laser CAM & CNC Software + Wireless Web Remote  
> **Version:** v1.2.5 (Official Release)  
> **Store:** Gumroad (`shop.pietiniart.com / nikopietinen.gumroad.com`)  
> **Target URL:** `https://nikopietinen.gumroad.com/l/photoncam`  
> **Pricing Model:** Freeware / Pay What You Want (`0 €+`, Suggested coffee tip: **5 €**)

---

## 1. Pricing Strategy: "Pay What You Want (0 €+ / Suggested 5 €)"

PhotonCAM is distributed as **100% Freeware**:
- Anyone can enter **0 €** and download the complete software package immediately with zero restrictions.
- The suggested price of **5 €** invites satisfied makers and workshops to support continuous independent development with a cup of coffee.
- This zero-friction model maximizes community adoption, word-of-mouth recommendations, and email list growth on Gumroad.

---

## 2. Files to Upload to Gumroad (`dist/`)

Upload these files to your Gumroad product download section:

| File Name | Size | Target Platform | Description for Downloaders |
| :--- | :--- | :--- | :--- |
| `PhotonCAM-1.2.5-universal.dmg` | ~12 MB | **macOS Universal 2** | Native build for all Apple Macs (Apple Silicon M1/M2/M3/M4 & Intel Core i5/i7/i9/Xeon). macOS 11.0+. |
| `PhotonCAM-1.2.5-windows-x64.zip` | ~4.0 MB | **Windows 10 / 11** | Portable standalone 64-bit executable (`.exe`). No complex installer needed. |
| `PhotonCAM-1.2.5-linux-x86_64.tar.gz` | ~7.7 MB | **Linux 64-bit** | Native ELF binary, desktop shortcut, and system icon for Ubuntu, Debian, Fedora, Arch, Mint. |
| `PhotonCAM-Pi-OS-arm64.img.xz` | ~567 MB | **Raspberry Pi OS (Pre-built)** | Ready-to-flash SD card image for Raspberry Pi 3/4/5. Wireless Web Remote server pre-installed (`pi:raspberry`). |
| `photoncam-bridge-linux-arm64` | ~3.0 MB | **Raspberry Pi / Linux ARM64** | Standalone headless bridge binary for existing Linux/Pi installations. |

---

## 3. Product Visuals & Cover Assets

Cover graphics and screenshots are located in:  
👉 [`assets/gumroad/`](file:///Volumes/Dataa/Ohjelmointi/ProjektitRust/laser_burn/assets/gumroad)

1. **Cover Image (1280 × 720 px, 16:9):**  
   [`assets/gumroad/gumroad_cover.jpg`](file:///Volumes/Dataa/Ohjelmointi/ProjektitRust/laser_burn/assets/gumroad/gumroad_cover.jpg)  
   *Raw URL:* `https://raw.githubusercontent.com/pietini69/photoncam/master/assets/gumroad/gumroad_cover.jpg`
2. **Square Thumbnail (1000 × 1000 px, 1:1):**  
   [`assets/gumroad/gumroad_thumb_square.jpg`](file:///Volumes/Dataa/Ohjelmointi/ProjektitRust/laser_burn/assets/gumroad/gumroad_thumb_square.jpg)  
   *Raw URL:* `https://raw.githubusercontent.com/pietini69/photoncam/master/assets/gumroad/gumroad_thumb_square.jpg`
3. **App Showcase (Dark & Light Mode):**  
   [`assets/gumroad/screenshot_dark_and_light.png`](file:///Volumes/Dataa/Ohjelmointi/ProjektitRust/laser_burn/assets/gumroad/screenshot_dark_and_light.png)  
   *Raw URL:* `https://raw.githubusercontent.com/pietini69/photoncam/master/assets/gumroad/screenshot_dark_and_light.png`

---

## 4. Product Listing Settings (Gumroad Dashboard)

* **Product Title:**  
  `PhotonCAM – Modern Laser CAM, CNC Software & Wireless Web Remote (macOS, Windows, Linux, Raspberry Pi)`
* **Short Summary (Card Description):**  
  `High-performance, modern laser cutting & engraving CAM software built in Rust. LightBurn .lbrn2 compatible, wireless Raspberry Pi web remote, true grayscale modulation, ceramic tile paint layer simulator & path optimizer for macOS, Windows, Linux, and Raspberry Pi.`
* **Custom URL Slug:** `photoncam` ➔ `https://nikopietinen.gumroad.com/l/photoncam`
* **Price:** `0` (Allow customers to pay what they want: **ON**, Minimum: `0`, Suggested amount: **`5`**)
* **Tags (Avainsanat):** `laser cutting`, `laser engraving`, `cam software`, `lightburn`, `grbl`, `ruida`, `raspberry pi`, `cnc`, `macos`, `windows`, `linux`, `maker`

---

## 5. Product Page Body (HTML / Gumroad Description)

Copy and paste the HTML below into the main Gumroad description box:

```html
<h2>🔥 Meet PhotonCAM v1.2.5: High-Performance Laser CAM & Wireless Web Remote</h2>
<p>
<strong>PhotonCAM</strong> is a modern, blazing-fast laser cutting and engraving CAM application engineered from the ground up in <strong>Rust</strong>. Designed for hobbyists, maker businesses, ceramic tile engravers, and high-precision workshops, PhotonCAM delivers sub-second vector path optimization, instant project loading, and full hardware control — with <strong>no subscriptions, no telemetry, and zero bloat</strong>.
</p>
<p>
<strong>✨ Freeware / Pay What You Want (0 €+):</strong> Download and use PhotonCAM completely free, or support independent maker software with a cup of coffee (5 €)!<br>
<strong>💻 All Platforms Included:</strong> macOS (Universal 2 for Apple Silicon &amp; Intel), Windows 10/11 (64-bit), Linux (x86_64), and Raspberry Pi (ARM64 / ARMv7).
</p>

<hr>

<h3>🌟 What's New in v1.2.5</h3>
<ul>
  <li><strong>🌐 Wireless Web Remote &amp; Raspberry Pi OS Image:</strong> Control your laser cutter over Wi-Fi from any smartphone, tablet, iPad, or laptop browser. Untether your computer from the workshop! Includes a pre-configured ready-to-flash Raspberry Pi OS Bookworm image.</li>
  <li><strong>🎯 4-Quadrant Machine Origin &amp; Mirror Calibration:</strong> Full support for machines with origins at Top-Left, Top-Right (Takaoikea), Bottom-Left, or Bottom-Right. Features live visual canvas mapping, independent X/Y mirroring, and inverted jog D-pad controls for brands like Ortur, xTool, Sculpfun, Atomstack, Creality, Two Trees, and Neje.</li>
  <li><strong>⚡ Sub-Second Path Optimizer:</strong> Re-orders cutting vectors with a greedy nearest-neighbor TSP algorithm and inner-before-outer contour logic. Cuts job times by 20% to 40% on intricate designs.</li>
  <li><strong>📂 LightBurn .lbrn2 &amp; SVG Compatibility:</strong> Open, view, and process modern LightBurn project files directly, including vector geometries, layer assignments (C00–C29), feed rates, power levels, and multi-pass settings.</li>
  <li><strong>🎨 Analog Grayscale PWM Modulation &amp; Ceramic Paint Simulator:</strong> Real dynamic power modulation (<code>M4 S...</code>) with automatic overscan lead-in/out. Includes a <strong>Multi-Color Ceramic Tile Paint Layer Simulator</strong> to preview how laser intensity removes stacked paint coats.</li>
  <li><strong>📷 Live Camera Workpiece Auto-Fit:</strong> Connect any overhead webcam with 4-point perspective homography calibration and sub-pixel zoom loupe to auto-center designs onto blank materials (tiles, coasters, wood offcuts).</li>
  <li><strong>🧪 Built-in Material Test Matrix Generator:</strong> Generate ready-to-burn Speed × Power calibration grids with one click to dial in parameters for any wood, acrylic, leather, slate, or tile.</li>
  <li><strong>🖼️ Studio-Grade Dithering (9 Modes):</strong> Atkinson, Floyd-Steinberg, Jarvis-Judice-Ninke, Stucki, Burkes, Sierra3, Ordered Bayer 8x8, Halftone, and True Grayscale Power Modulation.</li>
  <li><strong>🍎 macOS Apple Shortcuts Automation:</strong> Trigger Apple Shortcuts when jobs start or finish (turn on workshop exhaust systems, toggle lights, or send notifications).</li>
</ul>

<hr>

<h3>📦 What’s Included in Your Download</h3>
<ol>
  <li><strong>macOS Universal 2 DMG (<code>PhotonCAM-1.2.5-universal.dmg</code>):</strong> Native performance on all Macs — Apple Silicon (M1/M2/M3/M4) and Intel (x86_64). macOS 11.0+.</li>
  <li><strong>Windows 64-bit Portable (<code>PhotonCAM-1.2.5-windows-x64.zip</code>):</strong> Standalone executable for Windows 10 &amp; 11. No installer or administrative privileges required.</li>
  <li><strong>Linux 64-bit Desktop (<code>PhotonCAM-1.2.5-linux-x86_64.tar.gz</code>):</strong> Native 64-bit binary, desktop launcher, and icon for Ubuntu, Debian, Fedora, Arch, and Mint.</li>
  <li><strong>Raspberry Pi OS Image (<code>PhotonCAM-Pi-OS-arm64.img.xz</code>):</strong> Complete SD card image with pre-installed headless PhotonCAM Bridge &amp; Web Remote. Plug in your laser USB, power up the Pi, and access <code>http://photoncam.local:8080</code> from any device. Default user: <code>pi</code> / password: <code>raspberry</code>.</li>
  <li><strong>Standalone Bridge Binaries:</strong> Pre-compiled ARM64 and ARMv7 Linux bridge binaries for existing Raspberry Pi / Orange Pi setups.</li>
</ol>

<hr>

<h3>🚀 Quick Start Guide</h3>
<ol>
  <li><strong>Desktop:</strong> Launch PhotonCAM on macOS, Windows, or Linux. Plug in your laser's USB cable, select your serial COM / tty port, and click <strong>Connect</strong>.</li>
  <li><strong>Raspberry Pi Web Remote:</strong> Flash <code>PhotonCAM-Pi-OS-arm64.img.xz</code> to a micro-SD card using Raspberry Pi Imager. Insert into Raspberry Pi, connect laser via USB, and open <code>http://photoncam.local:8080</code> (or Pi's IP address) in your browser.</li>
  <li><strong>Configure Origin:</strong> Click the <strong>🎯 Origo</strong> button to select your machine's homing corner (Top-Left, Top-Right, Bottom-Left, or Bottom-Right).</li>
  <li><strong>Import &amp; Burn:</strong> Open any <code>.lbrn2</code>, <code>.svg</code>, or image file. Use the <strong>Frame (Bounding Box)</strong> tool to verify position, and click <strong>Start Cut</strong>!</li>
</ol>
```

---

## 6. Sähköpostiviesti ostajalle (Gumroad Receipt / "Thank You" Email)

Aseta Gumroadin *Email to customer* -kohtaan tämä viesti:

```text
Subject: Welcome to PhotonCAM v1.2.5! Here is your download & quick start guide 🚀

Hi there!

Thank you for downloading PhotonCAM v1.2.5! Whether you downloaded it for free or supported development with a tip, your support means the world to our independent maker project.

📂 WHAT TO DO NEXT:
1. Download the package for your platform:
   • macOS: PhotonCAM-1.2.5-universal.dmg
   • Windows: PhotonCAM-1.2.5-windows-x64.zip
   • Linux: PhotonCAM-1.2.5-linux-x86_64.tar.gz
   • Raspberry Pi: PhotonCAM-Pi-OS-arm64.img.xz (for wireless web control from iPad/phone)

2. Read the Quick Start instructions on the download page to connect your laser.

3. Need help or have feature requests?
   Join our GitHub repository or reply directly to this email:
   https://github.com/pietini69/photoncam

Happy laser making!
Niko Pietinen / PhotonCAM Team
```
