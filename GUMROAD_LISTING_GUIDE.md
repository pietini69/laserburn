# 🔥 LaserBurn – Gumroad Product Listing & Multi-Platform Launch Guide

> **Tuote:** LaserBurn – Modern Laser CAM & CNC Software  
> **Versio:** v1.0.0 (Virallinen julkaisuversio)  
> **Kauppapaikka:** Gumroad (`shop.pietiniart.com / nikopietinen.gumroad.com`)  
> **Kohdeosoite:** `https://shop.pietiniart.com/l/laserburn`  
> **Hinnoittelumalli:** Pay What You Want (`0 €+`, suositushinta kahvikupin verran: **4 €** tai **5 €**)

---

## 1. Miksi "Kahvikupin suositushinta" (3 € – 5 €) toimii parhaiten 0 €+ -mallissa?

Kun tuotteen perushinta on **0 €+ (Maksa mitä haluat / Pay What You Want)**, liian korkea suositushinta (kuten 14 € tai 15 €) luo psykologisen kynnyksen: moni lataaja ajattelee *"En halua maksaa 14 euroa tuntemattomasta ohjelmasta, joten kirjoitan kenttään 0 €"*.

Kun taas suositushinnaksi asetetaan **kahvikupin hinta (esim. 4 € tai 5 €)**:
- **Impulssikynnys katoaa:** 3 € – 5 € tuntuu pieneltä, reilulta "tarjoa kehittäjälle kahvit" -kiitokselta, jonka moni maksaa mielellään heti ladatessaan.
- **Konversio maksaviin lataajiin nousee merkittävästi:** Useampi lataaja jättää suositellun 4 € tai 5 € summan kenttään sen sijaan, että nollaisi sen.
- **Silti 100 % saavutettava:** Jokainen harrastaja pääsee lataamaan ja kokeilemaan ohjelmaa myös 0 eurolla.

---

## 2. Ladattavat paketit samalla tuotesivulla (`dist/`)

| Tiedosto | Koko | Alusta / Arkkitehtuuri | Kuvaus lataajalle |
| :--- | :--- | :--- | :--- |
| `LaserBurn-1.0.0-universal.dmg` | ~11 Mt | **macOS Universal 2** | Kaikki Macit (Apple Silicon M1/M2/M3/M4 & Intel Core i5/i7/i9/Xeon). macOS 11.0+. |
| `LaserBurn-1.0.0-windows-x64.zip` | ~3,5 Mt | **Windows 10 / 11** | Kannettava (portable) 64-bittinen standalone `.exe`. Ei raskasta asennusta. |
| `LaserBurn-1.0.0-linux-x86_64.tar.gz` | ~7,2 Mt | **Linux 64-bit** | Natiivi ELF-binääri, `.desktop`-pikakuvake ja 512px ikoni (Ubuntu, Debian, Fedora, Arch, Mint). |

---

## 3. Kuvat ja visuaaliset materiaalit

Kuvat sijaitsevat kansiossa:  
👉 [`dist/gumroad_assets/`](file:///Volumes/Dataa/Ohjelmointi/ProjektitRust/laser_burn/dist/gumroad_assets)

1. **Cover-kuva (1280 × 720 px, 16:9):**  
   [`dist/gumroad_assets/gumroad_cover.jpg`](file:///Volumes/Dataa/Ohjelmointi/ProjektitRust/laser_burn/dist/gumroad_assets/gumroad_cover.jpg)
2. **Neliöikoni / Thumbnail (1000 × 1000 px, 1:1):**  
   [`dist/gumroad_assets/gumroad_thumb_square.jpg`](file:///Volumes/Dataa/Ohjelmointi/ProjektitRust/laser_burn/dist/gumroad_assets/gumroad_thumb_square.jpg)

---

## 4. Tuotesivun tekstit ja esittely (Sales Copy – v1.0.0)

### Otsikko & Lyhytkuvaus
* **Product Title:**  
  `LaserBurn – Modern Laser CAM & CNC Software (macOS, Windows, Linux)`
* **Short Summary (Korttikuvaus):**  
  `High-performance, modern laser cutting & engraving CAM software built in Rust. LightBurn .lbrn2 compatible, true grayscale power modulation, ceramic paint layer simulator, live camera auto-fit & path optimizer for macOS, Windows, and Linux.`
* **Custom URL Slug:** `laserburn` ➔ `https://shop.pietiniart.com/l/laserburn`
* **Price:** `0` (Allow customers to pay what they want: **ON**, Minimum: `0`, Suggested amount: **`5`**)
* **Avainsanat (Tags):** `laser cutting`, `laser engraving`, `cam software`, `lightburn`, `grbl`, `ruida`, `cnc`, `macos`, `windows`, `linux`

---

### Tuotesivun leipäteksti (HTML / Gumroad Description)

```html
<h2>🔥 Meet LaserBurn v1.0.0: High-Performance Laser CAM & CNC Software</h2>
<p>
LaserBurn is a modern, blazing-fast laser cutting and engraving CAM application engineered from the ground up in <strong>Rust</strong>. Designed for hobbyists, craft businesses, ceramic tile engravers, and high-precision fabrication workshops, LaserBurn delivers sub-second vector path optimization, instant project loading, and full multi-controller hardware control — with <strong>no subscriptions and no bloat</strong>.
</p>
<p>
<strong>Free / Pay What You Want (0 €+):</strong> Download and use LaserBurn completely free, or support independent development with the price of a cup of coffee (3 € – 5 €)!<br>
<strong>All 3 Desktop Platforms Included:</strong> macOS (Universal 2 for Apple Silicon &amp; Intel), Windows 10/11 (64-bit), and Linux (x86_64).
</p>

<hr>

<h3>⚡ Key Features in v1.0.0</h3>
<ul>
  <li><strong>📂 LightBurn .lbrn2 &amp; SVG Compatibility:</strong> Open and save modern LightBurn project files directly, including vector paths, layer assignments (C00–C29), cut speeds, power settings, and multi-pass rules.</li>
  <li><strong>🎨 True Analog Grayscale Power Modulation &amp; Ceramic Paint Layer Simulator:</strong> Real dynamic PWM power modulation (<code>M4 S...</code>) between Min % and Max % power with automatic overscan lead-in/lead-out and negative inversion. Includes a specialized <strong>Multi-Color Ceramic Tile Paint Layer Simulator</strong> to preview how different laser power levels burn through stacked spray-paint layers.</li>
  <li><strong>📷 Live Camera Bed Overlay &amp; 1-Click Workpiece Auto-Fit:</strong> Connect any USB webcam or overhead camera with 4-point homography perspective calibration and sub-pixel zoom loupe. Automatically detect rectangular material blanks (tiles, coasters, plywood scraps) on the bed and center/fit your artwork with a single click.</li>
  <li><strong>🛠️ Vector CAD, 2D Boolean Tools &amp; SDF Contour Offset:</strong> Built-in vector drawing, multi-step Undo/Redo (<code>Cmd+Z</code>), 2D Boolean operations (Weld/Union, Subtract, Intersect), true Signed Distance Field (SDF) inner/outer contour offsetting, and automatic beam kerf compensation.</li>
  <li><strong>🧪 Built-in Material Speed × Power Test Matrix Generator:</strong> Generate ready-to-burn calibration grids with automatic vector labels in one click to find the sweet spot for any wood, acrylic, leather, slate, or painted ceramic tile.</li>
  <li><strong>🖼️ Studio-Grade Image Dithering (9 Modes):</strong> Atkinson, Floyd-Steinberg, Jarvis-Judice-Ninke, Stucki, Burkes, Sierra3, Ordered Bayer 8x8, Halftone, and True Grayscale Power Modulation.</li>
  <li><strong>📡 Real-Time Data Traffic Log (TX / RX) with Plain-English Explanations:</strong> Dedicated live G-code and GRBL serial inspector that explains every transmitted command and controller response in plain English, with optional real-time file logging.</li>
  <li><strong>🌗 Dark &amp; Light Workspace Themes + Dynamic Origin Jogging:</strong> Switch seamlessly between crisp Dark Mode and high-contrast Light Mode. Jog D-pad directions (<code>X+</code>/<code>X-</code>/<code>Y+</code>/<code>Y-</code>) dynamically adapt to your machine's configured <code>0,0</code> origin corner.</li>
  <li><strong>🍎 macOS Apple Shortcuts Automation:</strong> Trigger macOS Shortcuts automatically when a burn job starts or finishes (e.g., turn on workshop exhaust fans or smart lights, and send an iPhone/Apple Watch notification when cutting completes).</li>
  <li><strong>🖨️ Multi-Controller Support:</strong> Direct USB serial communication with <strong>GRBL</strong> (1.1+) diode/CO2 lasers (Ortur, xTool, Sculpfun, Atomstack, Creality, Two Trees, Neje, generic 3018) and network/USB support for <strong>Ruida</strong> DSP controllers.</li>
</ul>

<hr>

<h3>📦 What’s Included in Your Download</h3>
<p>When you download LaserBurn, you receive immediate access to all three platform builds:</p>
<ol>
  <li><strong>macOS Universal 2 Installer (<code>LaserBurn-1.0.0-universal.dmg</code>):</strong> Native performance on all Macs — Apple Silicon (M1/M2/M3/M4) and Intel (x86_64). Code-signed with Apple Developer certificate.</li>
  <li><strong>Windows 64-bit Standalone (<code>LaserBurn-1.0.0-windows-x64.zip</code>):</strong> Portable standalone executable for Windows 10 &amp; Windows 11. No complex installation or registry keys required.</li>
  <li><strong>Linux 64-bit Package (<code>LaserBurn-1.0.0-linux-x86_64.tar.gz</code>):</strong> Native 64-bit ELF binary, system icon, and <code>.desktop</code> launcher for Ubuntu, Debian, Fedora, Arch, and Linux Mint.</li>
</ol>

<hr>

<h3>🚀 Quick Start Guide</h3>
<ol>
  <li><strong>Launch LaserBurn</strong> on your machine (macOS, Windows, or Linux).</li>
  <li><strong>Connect your Laser:</strong> Connect the USB cable from your laser cutter to your computer. In LaserBurn, select your serial COM / tty port and click <strong>Connect</strong>.</li>
  <li><strong>Import or Draw:</strong> Use <strong>File → Open</strong> to load a <code>.lbrn2</code>, <code>.svg</code>, or image file (PNG/JPG/BMP), or use the built-in shape, text, boolean, and Material Test Grid tools.</li>
  <li><strong>Configure Layers:</strong> Assign shapes to layers (C00–C29), and set speed (mm/min), laser power %, dither mode, and number of passes.</li>
  <li><strong>Frame &amp; Run:</strong> Use the <strong>Frame (Bounding Box)</strong> tool with Weak Beam enabled to verify placement on your material blank, then press <strong>Start Job (Cut)</strong>.</li>
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
1. Open <code>LaserBurn-1.0.0-universal.dmg</code> and drag the <strong>LaserBurn</strong> icon into your <em>Applications</em> folder.<br>
2. <strong>First Launch (Gatekeeper Security):</strong><br>
Because LaserBurn is distributed directly outside the Mac App Store, macOS may show a standard security prompt on first launch. To open it smoothly:<br>
• <strong>Right-click (or Control-click)</strong> <code>LaserBurn.app</code> in your Applications folder and select <strong>Open</strong>, then click <strong>Open</strong> in the prompt.<br>
• <em>Alternatively:</em> Open <strong>System Settings → Privacy &amp; Security</strong>, scroll down to the Security section, and click <strong>"Open Anyway"</strong>.
</p>

<h4>🪟 Windows:</h4>
<p>
1. Extract <code>LaserBurn-1.0.0-windows-x64.zip</code> into any folder (e.g. <code>C:\\LaserBurn</code>).<br>
2. Double-click <code>LaserBurn.exe</code> to launch.<br>
3. <strong>First Launch (Windows Defender SmartScreen):</strong><br>
If the blue <em>"Windows protected your PC"</em> screen appears:<br>
• Click the link <strong>"More info"</strong>.<br>
• Click the button <strong>"Run anyway"</strong>.<br>
<em>(This only needs to be clicked once; Windows will remember your choice).</em>
</p>

<h4>🐧 Linux:</h4>
<p>
1. Extract <code>LaserBurn-1.0.0-linux-x86_64.tar.gz</code> and run <code>./LaserBurn</code>.<br>
2. <strong>USB Serial Port Permissions (GRBL &amp; Ruida):</strong> To allow LaserBurn to communicate with your laser via USB, ensure your user belongs to the serial group:<br>
• Ubuntu / Debian / Linux Mint: <code>sudo usermod -aG dialout $USER</code><br>
• Arch Linux / Manjaro / Fedora: <code>sudo usermod -aG uucp $USER</code><br>
<em>(Log out and back in once for the group permission to take effect).</em><br>
3. <strong>File Dialogs:</strong> Ensure <code>xdg-desktop-portal-gtk</code> (or your desktop environment's portal) is installed for native open/save file pickers.
</p>

<hr>

<p><em>LaserBurn v1.0.0. Developed by Niko Pietinen. Free updates and continuous improvements included.</em></p>
```
