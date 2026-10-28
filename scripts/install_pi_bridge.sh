#!/usr/bin/env bash
# PhotonCAM Bridge - 1-Line Installer for Raspberry Pi & Linux
# Installs photoncam-bridge as a persistent systemd service.
set -euo pipefail

echo "=========================================================="
echo "  🔥 PhotonCAM Bridge & Web Server Installer (Raspberry Pi)"
echo "=========================================================="

INSTALL_DIR="/usr/local/bin"
SERVICE_FILE="/etc/systemd/system/photoncam-bridge.service"

# Check if root
if [ "$EUID" -ne 0 ]; then
  echo "==> Re-running installer with sudo..."
  exec sudo bash "$0" "$@"
fi

# Detect Architecture
ARCH=$(uname -m)
echo "==> Detected system architecture: $ARCH"

# Add current user to dialout group for USB serial laser access
CURRENT_USER="${SUDO_USER:-$USER}"
echo "==> Adding user '$CURRENT_USER' to dialout group for USB laser control..."
usermod -aG dialout "$CURRENT_USER" || true

# If photoncam-bridge already exists in current folder, install it
if [ -f "./target/release/photoncam-bridge" ]; then
  echo "==> Installing locally compiled binary..."
  cp "./target/release/photoncam-bridge" "$INSTALL_DIR/photoncam-bridge"
elif [ -f "./photoncam-bridge" ]; then
  echo "==> Installing local binary..."
  cp "./photoncam-bridge" "$INSTALL_DIR/photoncam-bridge"
elif [ -f "./target/release/laserburn-bridge" ]; then
  echo "==> Installing locally compiled binary..."
  cp "./target/release/laserburn-bridge" "$INSTALL_DIR/photoncam-bridge"
else
  BIN_NAME=""
  case "$ARCH" in
    aarch64|arm64)
      BIN_NAME="photoncam-bridge-linux-arm64"
      ;;
    armv7*|armhf)
      BIN_NAME="photoncam-bridge-linux-armv7"
      ;;
    x86_64|amd64)
      BIN_NAME="photoncam-bridge-linux-x86_64"
      ;;
    *)
      BIN_NAME=""
      ;;
  esac

  if [ -n "$BIN_NAME" ]; then
    DOWNLOAD_URL="https://github.com/pietini69/photoncam/releases/download/v1.2.6/${BIN_NAME}"
    echo "==> Downloading precompiled $BIN_NAME from GitHub Releases ($DOWNLOAD_URL)..."
    if curl -sSL -f -o "$INSTALL_DIR/photoncam-bridge" "$DOWNLOAD_URL"; then
      echo "==> Successfully downloaded $BIN_NAME!"
    else
      echo "==> Release download failed, trying fallback..."
      FALLBACK_URL="https://github.com/pietini69/photoncam/releases/latest/download/${BIN_NAME}"
      curl -sSL -f -o "$INSTALL_DIR/photoncam-bridge" "$FALLBACK_URL" || true
    fi
  fi

  if [ ! -f "$INSTALL_DIR/photoncam-bridge" ] || [ ! -s "$INSTALL_DIR/photoncam-bridge" ]; then
    if command -v cargo &> /dev/null; then
      echo "==> Compiling from source with cargo..."
      cargo build --release --bin photoncam-bridge
      cp "./target/release/photoncam-bridge" "$INSTALL_DIR/photoncam-bridge"
    else
      echo "Error: Could not download precompiled binary and cargo is not installed."
      echo "Please download ${BIN_NAME:-photoncam-bridge} manually from https://github.com/pietini69/photoncam/releases"
      exit 1
    fi
  fi
fi

chmod +x "$INSTALL_DIR/photoncam-bridge"
ln -sf "$INSTALL_DIR/photoncam-bridge" "$INSTALL_DIR/laserburn-bridge"

echo "==> Creating systemd service: $SERVICE_FILE..."
cat << EOF > "$SERVICE_FILE"
[Unit]
Description=PhotonCAM Bridge & Web Server for Laser CAM & CNC
After=network.target network-online.target
Wants=network-online.target

[Service]
Type=simple
User=root
ExecStart=$INSTALL_DIR/photoncam-bridge --port 8080 --tcp-bridge 2323
Restart=always
RestartSec=3

[Install]
WantedBy=multi-user.target
EOF

echo "==> Enabling and starting photoncam-bridge service..."
systemctl daemon-reload
systemctl enable photoncam-bridge.service
systemctl restart photoncam-bridge.service

IP_ADDR=$(hostname -I 2>/dev/null | awk '{print $1}' || echo "127.0.0.1")

echo ""
echo "╔═══════════════════════════════════════════════════════════════════╗"
echo "║   🎉  PhotonCAM Bridge is now INSTALLED and RUNNING!             ║"
echo "╠═══════════════════════════════════════════════════════════════════╣"
echo "║   📱  Open Safari on iPad or phone:                              ║"
echo "║       http://$IP_ADDR:8080                                      ║"
echo "║       http://photoncam.local:8080                               ║"
echo "║                                                                   ║"
echo "║   💡  Tip: Tap 'Share' -> 'Add to Home Screen' in Safari         ║"
echo "║       for full-screen app experience on iPad!                    ║"
echo "╚═══════════════════════════════════════════════════════════════════╝"
echo ""
