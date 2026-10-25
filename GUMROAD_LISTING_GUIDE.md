# 🔥 LaserBurn – Gumroad Product Listing & Multi-Platform Launch Guide

> **Tuote:** LaserBurn – Modern Laser CAM & CNC Software  
> **Versio:** v1.0.0-beta.1  
> **Kauppapaikka:** Gumroad (`shop.pietiniart.com / nikopietinen.gumroad.com`)  
> **Kohdeosoite:** `https://shop.pietiniart.com/l/laserburn`  
> **Hinnoittelumalli:** Pay What You Want (0 €+ suositus 15 €) tai kiinteä beta-lanseeraushinta (esim. 19 €)

---

## 1. Saako kaikki käyttöjärjestelmät samalle sivulle?

**KYLLÄ! Juuri näin kannattaakin tehdä.**  
Gumroad tukee useita rinnakkaisia ladattavia tiedostoja samalla tuotesivulla. Kun ostaja hankkii LaserBurnin, hänen lataussivulleen ilmestyvät selkeät latauspainikkeet:

| Tiedosto | Koko | Alusta / Arkkitehtuuri | Kuvaus lataajalle |
| :--- | :--- | :--- | :--- |
| `LaserBurn-1.0.0-beta.1-universal.dmg` | 11 Mt | **macOS Universal 2** | Kaikki Macit (Apple Silicon M1/M2/M3/M4 & Intel Core i5/i7/i9/Xeon). macOS 11.0+. |
| `LaserBurn-1.0.0-beta.1-windows-x64.zip` | 3,4 Mt | **Windows 10 / 11** | Kannettava (portable) 64-bittinen standalone `.exe`. Ei asennusta. |
| `LaserBurn-1.0.0-beta.1-linux-x86_64.tar.gz` | 7,1 Mt | **Linux 64-bit** | Natiivi ELF-binääri, `.desktop`-pikakuvake ja 512px ikoni (Ubuntu, Debian, Fedora, Arch, Mint). |

Ostaja voi ladata suoraan oman koneensa version tai vaikka kaikki kolme (esimerkiksi jos suunnittelee Macilla toimistossa ja käyttää Windows-minipöytäkonetta verstaan laserleikkurilla).

---

## 2. Kuvat ja visuaaliset materiaalit

Kuvat on generoitu valmiiksi kansioon:  
👉 [`dist/gumroad_assets/`](file:///Volumes/Dataa/Ohjelmointi/ProjektitRust/laser_burn/dist/gumroad_assets)

1. **Cover-kuva (1280 × 720 px, 16:9):**  
   [`dist/gumroad_assets/gumroad_cover.jpg`](file:///Volumes/Dataa/Ohjelmointi/ProjektitRust/laser_burn/dist/gumroad_assets/gumroad_cover.jpg)  
   *Tumma moderni terästeema, hehkuva laser-suutinkuvake, arkkitehtuuribadget (macOS, Windows, Linux) ja keskeiset ominaisuudet lueteltuna.*
2. **Neliöikoni / Thumbnail (1000 × 1000 px, 1:1):**  
   [`dist/gumroad_assets/gumroad_thumb_square.jpg`](file:///Volumes/Dataa/Ohjelmointi/ProjektitRust/laser_burn/dist/gumroad_assets/gumroad_thumb_square.jpg)  
   *Selkeä, katseen vangitseva sovellusikoni Gumroad-kauppasi ja profiilisi tuoteristikkoon.*

---

## 3. Tuotesivun tekstit ja esittely (Sales Copy)

### Otsikko & Lyhytkuvaus
* **Product Title:**  
  `LaserBurn – Modern Laser CAM & CNC Software (macOS, Windows, Linux)`
* **Short Summary (Korttikuvaus):**  
  `High-performance, modern laser cutting & engraving CAM software built in Rust. LightBurn .lbrn2 compatible, advanced dithering, camera overlay & path optimizer for macOS, Windows, and Linux.`
* **Custom URL Slug:** `laserburn` ➔ `https://shop.pietiniart.com/l/laserburn`
* **Avainsanat (Tags):** `laser cutting`, `laser engraving`, `cam software`, `lightburn`, `grbl`, `ruida`, `cnc`, `macos`, `windows`, `linux`

---

### Tuotesivun leipäteksti (HTML / Gumroad Description)

```html
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
  <li><strong>macOS Universal 2 Installer (LaserBurn-1.0.0-beta.1-universal.dmg):</strong> Native performance on all Macs — Apple Silicon (M1/M2/M3/M4) and Intel (x86_64). Code-signed with Apple Developer certificate.</li>
  <li><strong>Windows 64-bit Standalone (LaserBurn-1.0.0-beta.1-windows-x64.zip):</strong> Portable standalone executable for Windows 10 & Windows 11. No complex installation or registry keys required.</li>
  <li><strong>Linux 64-bit Package (LaserBurn-1.0.0-beta.1-linux-x86_64.tar.gz):</strong> Native 64-bit ELF binary, system icon, and .desktop launcher for Ubuntu, Debian, Fedora, Arch, and Linux Mint.</li>
  <li><strong>Documentation & Quick Start Guide:</strong> Setup and controller connection guide.</li>
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
Because LaserBurn is freshly packaged, macOS may show a standard security prompt on first launch. To open it smoothly:<br>
• <strong>Right-click (or Control-click)</strong> <code>LaserBurn.app</code> in your Applications folder and select <strong>Open</strong>, then click <strong>Open</strong> in the prompt.<br>
• <em>Alternatively:</em> Open <strong>System Settings → Privacy & Security</strong>, scroll down to the Security section, and click <strong>"Open Anyway"</strong>.
</p>

<h4>🪟 Windows:</h4>
<p>
1. Extract <code>LaserBurn-1.0.0-beta.1-windows-x64.zip</code> into any folder (e.g. <code>C:\\LaserBurn</code>).<br>
2. Double-click <code>LaserBurn.exe</code> to launch.<br>
3. <strong>First Launch (Windows Defender SmartScreen):</strong><br>
If the blue <em>"Windows protected your PC"</em> screen appears:<br>
• Click the link <strong>"More info"</strong>.<br>
• Click the button <strong>"Run anyway"</strong>.<br>
<em>(This only needs to be clicked once; Windows will remember your choice).</em>
</p>

<h4>🐧 Linux:</h4>
<p>
1. Extract <code>LaserBurn-1.0.0-beta.1-linux-x86_64.tar.gz</code> and run <code>./LaserBurn</code>.<br>
2. <strong>USB Serial Port Permissions (GRBL & Ruida):</strong> To allow LaserBurn to communicate with your laser via USB, ensure your user belongs to the serial group:<br>
• Ubuntu / Debian / Linux Mint: <code>sudo usermod -aG dialout $USER</code><br>
• Arch Linux / Manjaro / Fedora: <code>sudo usermod -aG uucp $USER</code><br>
<em>(Log out and back in once for the group permission to take effect).</em><br>
3. <strong>File Dialogs:</strong> Ensure <code>xdg-desktop-portal-gtk</code> (or your desktop environment's portal) is installed for native open/save file pickers.
</p>

<hr>

<p><em>LaserBurn v1.0.0 Beta. Developed by Niko Pietinen. Free updates and continuous improvements included.</em></p>
```

---

## 4. Huomisaamun julkaisusuunnitelma (10 tuotteen erä)

Huomenna maanantaina 26.10. klo **08:00** ajetaan 10 tuotteen erä, josta napsaistaan #58 pois ja otetaan LaserBurn tilalle:

1. **#49:** Victorian Pastoral Manor Wall Art Set
2. **#50:** Provence Lavender Fields Wall Art Set
3. **#51:** Japandi Neutral Arches Wall Art Set
4. **#52:** Minimalist Botanical Shapes Wall Art Set
5. **#53:** Bauhaus Geometric Balance Wall Art Set
6. **#54:** Zen Sumi-E Ink Wash Wall Art Set
7. **#55:** Textured Plaster Relief Wall Art Set
8. **#56:** Terracotta Sunburst Boho Wall Art Set
9. **#57:** Mid Century Retro Palms Wall Art Set
10. 🎯 **LaserBurn – Modern Laser CAM & CNC Software (macOS, Windows, Linux)**

*(#58 Scandinavian Abstract siirtyy automaattisesti seuraavan erän ensimmäiseksi).*

LaserBurn voidaan julkaista huomenna aamulla yhdellä komennolla:
```bash
python3 /Volumes/Dataa/Ohjelmointi/ProjektitRust/laser_burn/scripts/publish_to_gumroad.py
```
*(Tai `--draft`-valitsimella, jos haluat ensin esikatsella sivua Gumroadin hallintapaneelissa ennen julkistamista).*
