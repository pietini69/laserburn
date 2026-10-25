#!/usr/bin/env bash
set -euo pipefail

# Build all release packages for macOS (Universal 2: Apple Silicon + Intel), Windows x64, and Linux x86_64

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo "=========================================="
echo " Building All LaserBurn Distribution Packages"
echo " (macOS Universal 2, Windows x64, Linux x86_64)"
echo "=========================================="

"$PROJECT_DIR/scripts/build_dmg.sh" universal
"$PROJECT_DIR/scripts/build_windows.sh"
"$PROJECT_DIR/scripts/build_linux.sh"

echo "=========================================="
echo " All distribution packages built successfully!"
ls -lh "$PROJECT_DIR/dist"
echo "=========================================="
