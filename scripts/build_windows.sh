#!/usr/bin/env bash
set -euo pipefail

# Build LaserBurn for Windows (x86_64-pc-windows-msvc) using cargo-xwin

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_DIR"

VERSION=$(grep '^version =' Cargo.toml | head -n1 | cut -d '"' -f 2)
DIST_DIR="$PROJECT_DIR/dist"
mkdir -p "$DIST_DIR"

echo "=========================================="
echo " Building LaserBurn v${VERSION} for Windows (x86_64-pc-windows-msvc)"
echo "=========================================="

export PATH="$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"
cargo xwin build --release --target x86_64-pc-windows-msvc

STAGE_DIR="$DIST_DIR/LaserBurn-${VERSION}-windows-x64"
rm -rf "$STAGE_DIR"
mkdir -p "$STAGE_DIR"

cp "$PROJECT_DIR/target/x86_64-pc-windows-msvc/release/laser_burn.exe" "$STAGE_DIR/LaserBurn.exe"

cat << EOF > "$STAGE_DIR/README.txt"
LaserBurn v${VERSION} (Windows x64)
===================================
Modern, high-performance laser cutting & engraving CAM software.

Usage:
  1. Extract this zip archive into any folder (e.g. C:\LaserBurn).
  2. Double-click LaserBurn.exe to start.

First Launch (Windows Defender SmartScreen):
  If the blue "Windows protected your PC" screen appears:
  - Click "More info"
  - Click "Run anyway"
  (This only needs to be clicked once; Windows will remember your choice).
EOF

ZIP_PATH="$DIST_DIR/LaserBurn-${VERSION}-windows-x64.zip"
rm -f "$ZIP_PATH"
(cd "$STAGE_DIR" && zip -r "$ZIP_PATH" LaserBurn.exe README.txt)
rm -rf "$STAGE_DIR"

echo "==> Windows package created: $ZIP_PATH"
