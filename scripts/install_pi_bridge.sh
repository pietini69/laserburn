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

# Add current user to dialout group for serial port access
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
  cp "./target/release/laserburn-bridge" "$INSTALL_DIR/photoncam-bridge"
else
  echo "==> Compiling or downloading latest release binary..."
  if command -v cargo &> /dev/null; then
    cargo build --release --bin photoncam-bridge
    cp "./target/release/photoncam-bridge" "$INSTALL_DIR/photoncam-bridge"
  else
    echo "Error: photoncam-bridge binary not found and cargo is not installed."
    echo "Please build with 'cargo build --release --bin photoncam-bridge' or copy the binary here."
    exit 1
  fi
fi

chmod +x "$INSTALL_DIR/photoncam-bridge"
ln -sf "$INSTALL_DIR/photoncam-bridge" "$INSTALL_DIR/laserburn-bridge"

echo "==> Creating systemd service: $SERVICE_FILE..."
cat << EOF > "$SERVICE_FILE"
[Unit]
Description=PhotonCAM Bridge & Web Server for Laser CAM & CNC
After=network.target

[Service]
Type=simple
User=$CURRENT_USER
WorkingDirectory=/home/$CURRENT_USER
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

IP_ADDR=$(hostname -I | awk '{print $1}')

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
