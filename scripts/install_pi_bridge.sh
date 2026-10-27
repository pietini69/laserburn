#!/usr/bin/env bash
# LaserBurn Bridge - 1-Line Installer for Raspberry Pi & Linux
# Installs laserburn-bridge as a persistent systemd service.
set -euo pipefail

echo "=========================================================="
echo "  🔥 LaserBurn Bridge & Web Server Installer (Raspberry Pi)"
echo "=========================================================="

INSTALL_DIR="/usr/local/bin"
SERVICE_FILE="/etc/systemd/system/laserburn-bridge.service"

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

# If laserburn-bridge already exists in current folder, install it
if [ -f "./target/release/laserburn-bridge" ]; then
  echo "==> Installing locally compiled binary..."
  cp "./target/release/laserburn-bridge" "$INSTALL_DIR/laserburn-bridge"
elif [ -f "./laserburn-bridge" ]; then
  echo "==> Installing local binary..."
  cp "./laserburn-bridge" "$INSTALL_DIR/laserburn-bridge"
else
  echo "==> Compiling or downloading latest release binary..."
  if command -v cargo &> /dev/null; then
    cargo build --release --bin laserburn-bridge
    cp "./target/release/laserburn-bridge" "$INSTALL_DIR/laserburn-bridge"
  else
    echo "Error: laserburn-bridge binary not found and cargo is not installed."
    echo "Please build with 'cargo build --release --bin laserburn-bridge' or copy the binary here."
    exit 1
  fi
fi

chmod +x "$INSTALL_DIR/laserburn-bridge"

echo "==> Creating systemd service: $SERVICE_FILE..."
cat << EOF > "$SERVICE_FILE"
[Unit]
Description=LaserBurn Bridge & Web Server for Laser CAM & CNC
After=network.target

[Service]
Type=simple
User=$CURRENT_USER
WorkingDirectory=/home/$CURRENT_USER
ExecStart=$INSTALL_DIR/laserburn-bridge --port 8080 --tcp-bridge 2323
Restart=always
RestartSec=3

[Install]
WantedBy=multi-user.target
EOF

echo "==> Enabling and starting laserburn-bridge service..."
systemctl daemon-reload
systemctl enable laserburn-bridge.service
systemctl restart laserburn-bridge.service

IP_ADDR=$(hostname -I | awk '{print $1}')

echo ""
echo "╔═══════════════════════════════════════════════════════════════════╗"
echo "║   🎉  LaserBurn Bridge is now INSTALLED and RUNNING!             ║"
echo "╠═══════════════════════════════════════════════════════════════════╣"
echo "║   📱  Open Safari on iPad or phone:                              ║"
echo "║       http://$IP_ADDR:8080                                      ║"
echo "║       http://laserburn.local:8080                               ║"
echo "║                                                                   ║"
echo "║   💡  Tip: Tap 'Share' -> 'Add to Home Screen' in Safari         ║"
echo "║       for full-screen app experience on iPad!                    ║"
echo "╚═══════════════════════════════════════════════════════════════════╝"
echo ""
