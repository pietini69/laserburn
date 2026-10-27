#!/usr/bin/env bash
set -euo pipefail

# Build PhotonCAM for Linux (x86_64-unknown-linux-gnu) using cargo-zigbuild

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_DIR"

VERSION=$(grep '^version =' Cargo.toml | head -n1 | cut -d '"' -f 2)
DIST_DIR="$PROJECT_DIR/dist"
mkdir -p "$DIST_DIR"

echo "=========================================="
echo " Building PhotonCAM v${VERSION} for Linux (x86_64-unknown-linux-gnu)"
echo "=========================================="

export PATH="$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"
export BINDGEN_EXTRA_CLANG_ARGS="-I/tmp/linux_headers/usr/include -I/tmp/linux_headers/usr/include/x86_64-linux-gnu"

cargo zigbuild --release --bin photoncam --target x86_64-unknown-linux-gnu

STAGE_DIR="$DIST_DIR/PhotonCAM-${VERSION}-linux-x86_64"
rm -rf "$STAGE_DIR"
mkdir -p "$STAGE_DIR"

cp "$PROJECT_DIR/target/x86_64-unknown-linux-gnu/release/photoncam" "$STAGE_DIR/photoncam"
chmod +x "$STAGE_DIR/photoncam"
cp "$PROJECT_DIR/assets/laserburn.iconset/icon_512x512.png" "$STAGE_DIR/photoncam.png"

cat << 'EOF' > "$STAGE_DIR/PhotonCAM.desktop"
[Desktop Entry]
Name=PhotonCAM
Comment=Modern Laser Cutting & Engraving CAM Software
Exec=photoncam
Icon=photoncam
Terminal=false
Type=Application
Categories=Graphics;Engineering;Development;
EOF

cat << EOF > "$STAGE_DIR/README.txt"
PhotonCAM v${VERSION} (Linux x86_64)
====================================
Modern, high-performance laser cutting & engraving CAM software.

Running PhotonCAM:
  1. Ensure execution permissions:
     chmod +x photoncam
  2. Launch directly from terminal or file manager:
     ./photoncam

USB Serial Port Permissions (GRBL & Ruida):
  To allow PhotonCAM to communicate with USB serial ports (/dev/ttyUSB0, /dev/ttyACM0),
  ensure your user belongs to the 'dialout' (Ubuntu/Debian) or 'uucp' (Arch/Fedora) group:
     sudo usermod -aG dialout \$USER   # Ubuntu / Debian / Linux Mint
     sudo usermod -aG uucp \$USER      # Arch Linux / Manjaro / Fedora
  (Log out and back in once for permissions to take effect).

File Dialogs on Linux:
  Ensure an XDG Desktop Portal backend is installed for native open/save dialogs:
     sudo apt install xdg-desktop-portal-gtk    # Ubuntu / Debian
     sudo pacman -S xdg-desktop-portal-gtk     # Arch Linux
EOF

TAR_PATH="$DIST_DIR/PhotonCAM-${VERSION}-linux-x86_64.tar.gz"
rm -f "$TAR_PATH"
(cd "$DIST_DIR" && tar -czvf "$TAR_PATH" "PhotonCAM-${VERSION}-linux-x86_64")
rm -rf "$STAGE_DIR"

echo "==> Linux package created: $TAR_PATH"
