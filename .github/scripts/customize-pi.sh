#!/bin/bash
set -euo pipefail

echo "=========================================="
echo "  Configuring PhotonCAM OS on Raspberry Pi"
echo "=========================================="

# 1. Install prerequisites (avahi-daemon for .local mDNS discovery)
export DEBIAN_FRONTEND=noninteractive
apt-get update
apt-get install -y --no-install-recommends avahi-daemon udev libudev1

# 2. Set Hostname to 'photoncam'
echo "photoncam" > /etc/hostname
sed -i 's/raspberrypi/photoncam/g' /etc/hosts || true

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# 3. Install photoncam-bridge binary
if [ -f "$SCRIPT_DIR/photoncam-bridge" ]; then
    cp "$SCRIPT_DIR/photoncam-bridge" /usr/local/bin/photoncam-bridge
    chmod +x /usr/local/bin/photoncam-bridge
elif [ -f "$SCRIPT_DIR/laserburn-bridge" ]; then
    cp "$SCRIPT_DIR/laserburn-bridge" /usr/local/bin/photoncam-bridge
    chmod +x /usr/local/bin/photoncam-bridge
elif [ -f "/photoncam-bridge" ]; then
    cp /photoncam-bridge /usr/local/bin/photoncam-bridge
    chmod +x /usr/local/bin/photoncam-bridge
fi

ln -sf /usr/local/bin/photoncam-bridge /usr/local/bin/laserburn-bridge

# 4. Create systemd service
cat << 'EOF' > /etc/systemd/system/photoncam-bridge.service
[Unit]
Description=PhotonCAM Bridge & Web Server for Laser CAM & CNC
After=network.target avahi-daemon.service

[Service]
Type=simple
User=root
ExecStart=/usr/local/bin/photoncam-bridge --port 8080 --tcp-bridge 2323
Restart=always
RestartSec=3

[Install]
WantedBy=multi-user.target
EOF

# 5. Enable services
# In a container/chroot build environment, systemd is not PID 1.
mkdir -p /etc/systemd/system/multi-user.target.wants
ln -sf /etc/systemd/system/photoncam-bridge.service /etc/systemd/system/multi-user.target.wants/photoncam-bridge.service
ln -sf /lib/systemd/system/avahi-daemon.service /etc/systemd/system/multi-user.target.wants/avahi-daemon.service || true
systemctl enable avahi-daemon 2>/dev/null || true
systemctl enable photoncam-bridge.service 2>/dev/null || true

echo "PhotonCAM OS customization complete!"
