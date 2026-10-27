#!/bin/bash
set -euo pipefail

echo "=========================================="
echo "  Configuring LaserBurn OS on Raspberry Pi"
echo "=========================================="

# 1. Install prerequisites (avahi-daemon for .local mDNS discovery)
export DEBIAN_FRONTEND=noninteractive
apt-get update
apt-get install -y --no-install-recommends avahi-daemon udev libudev1

# 2. Set Hostname to 'laserburn'
echo "laserburn" > /etc/hostname
sed -i 's/raspberrypi/laserburn/g' /etc/hosts || true

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# 3. Install laserburn-bridge binary
if [ -f "$SCRIPT_DIR/laserburn-bridge" ]; then
    cp "$SCRIPT_DIR/laserburn-bridge" /usr/local/bin/laserburn-bridge
    chmod +x /usr/local/bin/laserburn-bridge
elif [ -f "/laserburn-bridge" ]; then
    cp /laserburn-bridge /usr/local/bin/laserburn-bridge
    chmod +x /usr/local/bin/laserburn-bridge
elif [ -f "/tmp/laserburn-bridge" ]; then
    cp /tmp/laserburn-bridge /usr/local/bin/laserburn-bridge
    chmod +x /usr/local/bin/laserburn-bridge
fi

# 4. Create systemd service
cat << 'EOF' > /etc/systemd/system/laserburn-bridge.service
[Unit]
Description=LaserBurn Bridge & Web Server for Laser CAM & CNC
After=network.target avahi-daemon.service

[Service]
Type=simple
User=root
ExecStart=/usr/local/bin/laserburn-bridge --port 8080 --tcp-bridge 2323
Restart=always
RestartSec=3

[Install]
WantedBy=multi-user.target
EOF

# 5. Enable services
systemctl daemon-reload
systemctl enable avahi-daemon
systemctl enable laserburn-bridge.service

echo "LaserBurn OS customization complete!"
