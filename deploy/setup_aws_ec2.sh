#!/usr/bin/env bash
set -euo pipefail

# ==============================================================================
# AWS EC2 Initial Setup Script for Ozone Pokémon Telegram Bot
# Compatible with Ubuntu 22.04 / 24.04 LTS and Debian 12
# ==============================================================================

echo "=== [1/6] Updating system packages ==="
sudo apt-get update -y
sudo apt-get upgrade -y
sudo apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    gnupg \
    lsb-release \
    git \
    sqlite3 \
    python3 \
    python3-pil \
    tar \
    gzip

echo "=== [2/6] Configuring Swap Space (2GB) ==="
# Swap prevents out-of-memory errors on small EC2 instances (e.g., t3.micro/t3.small)
if [ ! -f /swapfile ]; then
    echo "Creating 2GB swapfile..."
    sudo fallocate -l 2G /swapfile || sudo dd if=/dev/zero of=/swapfile bs=1M count=2048
    sudo chmod 600 /swapfile
    sudo mkswap /swapfile
    sudo swapon /swapfile
    echo '/swapfile none swap sw 0 0' | sudo tee -a /etc/fstab
    echo "Swap created successfully."
else
    echo "Swapfile already exists."
fi

echo "=== [3/6] Installing Docker & Docker Compose Plugin ==="
if ! command -v docker &> /dev/null; then
    sudo install -m 0755 -d /etc/apt/keyrings
    curl -fsSL https://download.docker.com/linux/ubuntu/gpg | sudo gpg --dearmor -o /etc/apt/keyrings/docker.gpg || \
    curl -fsSL https://download.docker.com/linux/debian/gpg | sudo gpg --dearmor -o /etc/apt/keyrings/docker.gpg
    sudo chmod a+r /etc/apt/keyrings/docker.gpg

    echo \
      "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/ubuntu \
      $(. /etc/os-release && echo "$VERSION_CODENAME") stable" | \
      sudo tee /etc/apt/sources.list.d/docker.list > /dev/null || true

    sudo apt-get update -y
    sudo apt-get install -y docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin

    # Enable and start Docker
    sudo systemctl enable docker
    sudo systemctl start docker

    # Add current user to docker group
    sudo usermod -aG docker "$USER" || true
    echo "Docker installed. (Note: log out and back in to use docker without sudo)"
else
    echo "Docker is already installed."
fi

echo "=== [4/6] Creating deployment directories ==="
APP_DIR="/opt/ozone_bot"
sudo mkdir -p "${APP_DIR}/data" "${APP_DIR}/backups"
sudo chown -R "$USER:$USER" "$APP_DIR"
chmod 755 "$APP_DIR"

echo "=== [5/6] Setting permissions for scripts ==="
if [ -d "./deploy" ]; then
    chmod +x ./deploy/*.sh 2>/dev/null || true
fi

echo "=== [6/6] Setup Completed Successfully ==="
echo ""
echo "Next steps:"
echo "1. Copy your .env file into ${APP_DIR}/.env (see .env.example)"
echo "2. Copy pokemon_bot.db into ${APP_DIR}/data/pokemon_bot.db if migrating existing player data"
echo "3. Run 'docker compose up -d --build' to start the bot"
echo "4. View logs with: 'docker compose logs -f'"
echo ""

