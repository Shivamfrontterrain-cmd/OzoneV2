#!/usr/bin/env bash
set -euo pipefail

# ==============================================================================
# AWS EC2 Clean Slate Script
# Stops old bot processes/containers and cleans files for a fresh installation
# ==============================================================================

echo "=== [1/5] Stopping and disabling running systemd services ==="
# Stop common bot service names if they exist
for svc in ozone_bot ozone_pokemon_bot pokemon_bot telegram_bot bot; do
    if systemctl is-active --quiet "$svc" 2>/dev/null; then
        echo "Stopping service: $svc"
        sudo systemctl stop "$svc" || true
    fi
    if systemctl is-enabled --quiet "$svc" 2>/dev/null; then
        echo "Disabling service: $svc"
        sudo systemctl disable "$svc" || true
    fi
    sudo rm -f "/etc/systemd/system/${svc}.service"
done
sudo systemctl daemon-reload

echo "=== [2/5] Stopping all Docker containers and cleaning Docker state ==="
if command -v docker &>/dev/null; then
    RUNNING_CONTAINERS=$(docker ps -aq 2>/dev/null || true)
    if [ -n "$RUNNING_CONTAINERS" ]; then
        echo "Stopping running Docker containers..."
        docker stop $RUNNING_CONTAINERS || true
        echo "Removing Docker containers..."
        docker rm -f $RUNNING_CONTAINERS || true
    fi
    echo "Pruning unused Docker images, volumes, and networks..."
    docker system prune -af --volumes || true
fi

echo "=== [3/5] Terminating any lingering background processes ==="
# Terminate any stray processes running in tmux, screen, nohup, or background
sudo pkill -9 -f ozone_pokemon_bot 2>/dev/null || true
sudo pkill -9 -f "python.*bot\.py" 2>/dev/null || true
sudo pkill -9 -f "cargo run" 2>/dev/null || true

# Stop PM2 processes if pm2 is present
if command -v pm2 &>/dev/null; then
    pm2 stop all 2>/dev/null || true
    pm2 delete all 2>/dev/null || true
    pm2 save --force 2>/dev/null || true
fi

echo "=== [4/5] Removing old application directory ==="
# Remove /opt/ozone_bot (adjust if your old bot was located in a different path)
TARGET_DIR="/opt/ozone_bot"
if [ -d "$TARGET_DIR" ]; then
    echo "Removing $TARGET_DIR..."
    sudo rm -rf "$TARGET_DIR"
fi

# Clean up temp trainer cards
rm -rf /tmp/ozone_trainer_cards

echo "=== [5/5] Re-creating fresh deployment directory ==="
sudo mkdir -p "${TARGET_DIR}/data" "${TARGET_DIR}/backups"
sudo chown -R "$USER:$USER" "$TARGET_DIR"
chmod 755 "$TARGET_DIR"

echo "=== Clean Slate Complete ==="
echo "Your EC2 instance is clean and ready for a fresh deployment in ${TARGET_DIR}."

