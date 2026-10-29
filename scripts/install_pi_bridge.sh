#!/usr/bin/env bash
# PhotonCAM Bridge - 1-Line Installer for Raspberry Pi & Linux
# Installs photoncam-bridge as a persistent, auto-starting systemd service on boot.
set -euo pipefail

echo "=========================================================="
echo "  🔥 PhotonCAM Bridge & Web Server Installer (Raspberry Pi)"
echo "=========================================================="

INSTALL_DIR="/usr/local/bin"
SERVICE_FILE="/etc/systemd/system/photoncam-bridge.service"

# Check if running as root / sudo; if not, escalate privileges automatically
if [ "$EUID" -ne 0 ]; then
  echo "==> Sudo-oikeudet vaaditaan systemd-automaattikäynnistyksen asentamiseen."
  if command -v sudo &> /dev/null; then
    if [ -f "$0" ] && [ "$0" != "bash" ] && [ "$0" != "sh" ]; then
      exec sudo bash "$0" "$@"
    else
      # If executed directly via pipe: curl ... | bash
      exec sudo bash -c "$(curl -sSL https://raw.githubusercontent.com/pietini69/photoncam/master/scripts/install_pi_bridge.sh)" "$@"
    fi
  else
    echo "Virhe: Aja tämä skripti root-käyttäjänä tai sudolla:"
    echo "  curl -sSL https://raw.githubusercontent.com/pietini69/photoncam/master/scripts/install_pi_bridge.sh | sudo bash"
    exit 1
  fi
fi

# Detect Architecture
ARCH=$(uname -m)
echo "==> Tunnistettu prosessoriarkkitehtuuri: $ARCH"

# Add target user to dialout group for USB serial laser access
CURRENT_USER="${SUDO_USER:-${USER:-pi}}"
if id "$CURRENT_USER" &>/dev/null; then
  echo "==> Lisätään käyttäjä '$CURRENT_USER' dialout-ryhmään USB-sarjaporttioikeuksia varten..."
  usermod -aG dialout "$CURRENT_USER" || true
fi

# Check if locally compiled binary is available in current or standard paths
INSTALLED=false
if [ -f "./target/release/photoncam-bridge" ]; then
  echo "==> Asennetaan paikallisesti käännetty binääri..."
  cp "./target/release/photoncam-bridge" "$INSTALL_DIR/photoncam-bridge"
  INSTALLED=true
elif [ -f "./photoncam-bridge" ]; then
  echo "==> Asennetaan nykyisessä hakemistossa oleva binääri..."
  cp "./photoncam-bridge" "$INSTALL_DIR/photoncam-bridge"
  INSTALLED=true
elif [ -f "./target/release/laserburn-bridge" ]; then
  echo "==> Asennetaan laserburn-bridge binääri..."
  cp "./target/release/laserburn-bridge" "$INSTALL_DIR/photoncam-bridge"
  INSTALLED=true
elif [ -f "$INSTALL_DIR/photoncam-bridge" ] && [ -s "$INSTALL_DIR/photoncam-bridge" ]; then
  echo "==> Käytetään jo asennettua binääriä: $INSTALL_DIR/photoncam-bridge"
  INSTALLED=true
fi

if [ "$INSTALLED" = false ]; then
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
    echo "==> Ladataan esikäännetty $BIN_NAME GitHub Releases -jakelusta..."
    LATEST_URL="https://github.com/pietini69/photoncam/releases/latest/download/${BIN_NAME}"
    if curl -sSL -f -o "$INSTALL_DIR/photoncam-bridge" "$LATEST_URL"; then
      echo "==> Ladattu onnistuneesti: $BIN_NAME"
      INSTALLED=true
    else
      V126_URL="https://github.com/pietini69/photoncam/releases/download/v1.2.6/${BIN_NAME}"
      if curl -sSL -f -o "$INSTALL_DIR/photoncam-bridge" "$V126_URL"; then
        echo "==> Ladattu v1.2.6 fallback: $BIN_NAME"
        INSTALLED=true
      fi
    fi
  fi

  if [ "$INSTALLED" = false ]; then
    if command -v cargo &> /dev/null; then
      echo "==> Käännetään lähdekoodista cargolla..."
      cargo build --release --bin photoncam-bridge
      cp "./target/release/photoncam-bridge" "$INSTALL_DIR/photoncam-bridge"
      INSTALLED=true
    else
      echo "Virhe: Esikäännettyä binääriä ei voitu ladata eikä cargoa ole asennettu."
      echo "Lataa binääri ${BIN_NAME:-photoncam-bridge} käsin osoitteesta: https://github.com/pietini69/photoncam/releases"
      exit 1
    fi
  fi
fi

chmod +x "$INSTALL_DIR/photoncam-bridge"
ln -sf "$INSTALL_DIR/photoncam-bridge" "$INSTALL_DIR/laserburn-bridge"

# -------------------------------------------------------------
# SYSTEMD AUTOMAATTIKÄYNNISTYS (BOOT STARTUP)
# -------------------------------------------------------------
echo "==> Luodaan automaattisesti käynnistyvä systemd-palvelu: $SERVICE_FILE..."
cat << EOF > "$SERVICE_FILE"
[Unit]
Description=PhotonCAM Bridge & Web Remote Server
Documentation=https://github.com/pietini69/photoncam
After=network.target network-online.target
Wants=network-online.target

[Service]
Type=simple
User=root
WorkingDirectory=/usr/local/bin
ExecStart=$INSTALL_DIR/photoncam-bridge --port 8080 --tcp-bridge 2323
Restart=always
RestartSec=3
KillMode=process

[Install]
WantedBy=multi-user.target
EOF

echo "==> Aktivoidaan automaattikäynnistys käynnistyksen (boot) yhteydessä..."
systemctl daemon-reload
systemctl enable photoncam-bridge.service
systemctl restart photoncam-bridge.service

# Verify service status
if systemctl is-active --quiet photoncam-bridge.service; then
  echo "==> Palvelu on nyt KÄYNNISSÄ ja AKTIVOITU käynnistymään automaattisesti aina kun Pi saa virtaa!"
else
  echo "==> Varoitus: Palvelu aktivoitiin, mutta se ei vielä käynnistynyt. Tarkista lokit: journalctl -u photoncam-bridge -n 20"
fi

IP_ADDR=$(hostname -I 2>/dev/null | awk '{print $1}' || echo "127.0.0.1")

echo ""
echo "╔═══════════════════════════════════════════════════════════════════╗"
echo "║   🎉  PhotonCAM Bridge on ASENNETTU ja KÄYNNISSÄ!                ║"
echo "╠═══════════════════════════════════════════════════════════════════╣"
echo "║   ⚡  Automaattikäynnistys: PÄÄLLÄ (käynnistyy aina bootissa)    ║"
echo "║   🌐  Avaa iPadin tai tietokoneen selaimessa:                     ║"
echo "║       http://$IP_ADDR:8080                                      ║"
echo "║       http://photoncam.local:8080                               ║"
echo "║                                                                   ║"
echo "║   📱  Vinkki: Safarissa 'Jaa' -> 'Lisää Koti-valikkoon'          ║"
echo "║       avaa PhotonCAMin itsenäisenä koko ruudun sovelluksena!     ║"
echo "╚═══════════════════════════════════════════════════════════════════╝"
echo ""
