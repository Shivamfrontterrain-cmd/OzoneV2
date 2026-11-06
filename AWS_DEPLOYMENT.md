# AWS Deployment Guide for Ozone Pokémon Bot

This guide walks you through deploying the Ozone Pokémon Telegram Bot to Amazon Web Services (AWS).

---

## 1. Recommended AWS Architecture

| Component | Recommendation | Notes |
| :--- | :--- | :--- |
| **Service** | **AWS EC2** or **AWS Lightsail** | Long-polling Telegram bot with stateful SQLite database. |
| **Instance Type** | **t3.small** (2 vCPU, 2GB RAM) or **t4g.small** (ARM) | Minimum 2GB RAM recommended. A 2GB swapfile is configured automatically. |
| **Operating System** | **Ubuntu 24.04 LTS** (or 22.04 LTS) | Debian-compatible, stable, fully supported. |
| **Storage (EBS)** | **20 GB gp3** | Provides ample space for OS, Docker images, assets (~1.2 GB), and database backups. |
| **Monthly Cost** | ~$5 - $15 / month | (AWS Lightsail offers a flat $5/month instance). |

---

## 2. AWS Security Group Configuration

In the AWS EC2 Console, configure your Instance **Security Group**:

### Inbound Rules
| Type | Protocol | Port Range | Source | Purpose |
| :--- | :--- | :--- | :--- | :--- |
| **SSH** | TCP | 22 | `My IP` (or trusted admin IP) | Secure remote server management. |
| **Custom TCP** | TCP | 5000 | `My IP` (Optional) | Gym Admin Web Panel. *(Recommended: Keep closed and access via SSH tunnel).* |

### Outbound Rules
| Type | Protocol | Port Range | Destination | Purpose |
| :--- | :--- | :--- | :--- | :--- |
| **All Traffic** | All | All | `0.0.0.0/0` | Connect to Telegram Bot API (`api.telegram.org:443`). |

---

## 3. Server Provisioning & Setup (Automated)

### 3.1 Connect to your EC2 instance via SSH
```bash
ssh -i /path/to/your-key.pem ubuntu@<YOUR_EC2_PUBLIC_IP>
```

### 3.2 Upload Project Files (Without GitHub - Recommended)

#### Option A: One-Click PowerShell Script (From your local Windows PC)
We provide an automated upload script that packages all source files, assets, and database into a compressed archive (excluding the multi-gigabyte Windows `target/` build directory) and uploads it directly to your EC2 instance:

```powershell
# Run in PowerShell from the project root folder:
.\deploy\upload_to_ec2.ps1 -Ec2Ip "<YOUR_EC2_PUBLIC_IP>" -KeyPath "C:\path\to\your-key.pem"
```

#### Option B: Manual Command (From your local Windows PC)
```powershell
# 1. Package files into a single compressed archive (skipping target/ and build cache):
tar --exclude="target" --exclude=".git" --exclude="*.zip" --exclude="*.bak*" -czf ozone_deploy.tar.gz src assets deploy Cargo.toml Cargo.lock rust-toolchain.toml pokedex.json Dockerfile docker-compose.yml docker-entrypoint.sh pokemon_bot.db .env

# 2. Upload the archive to your EC2 home directory:
scp -i "C:\path\to\your-key.pem" ozone_deploy.tar.gz ubuntu@<YOUR_EC2_PUBLIC_IP>:/home/ubuntu/
```

#### Option C: WinSCP (Graphical Drag & Drop)
1. Download and launch [WinSCP](https://winscp.net/).
2. Enter your EC2 Public IP, username `ubuntu`, and select your `.pem` key file in **Advanced > SSH > Authentication**.
3. Drag and drop project files to `/opt/ozone_bot` on EC2 (be sure not to copy the `target/` folder).

---

### 3.3 Extract and Install on EC2

In your EC2 SSH terminal:
```bash
# 1. Create target bot directory
sudo mkdir -p /opt/ozone_bot
sudo chown -R ubuntu:ubuntu /opt/ozone_bot

# 2. Extract the uploaded package
tar -xzf /home/ubuntu/ozone_deploy.tar.gz -C /opt/ozone_bot/
cd /opt/ozone_bot

# 3. Ensure persistent data and backup directories exist
mkdir -p data backups
if [ -f pokemon_bot.db ]; then
  cp pokemon_bot.db data/pokemon_bot.db
fi

# 4. Run automated EC2 configuration script
chmod +x deploy/setup_aws_ec2.sh
./deploy/setup_aws_ec2.sh
```

---

## 4. Environment & Data Configuration

### 4.1 Create production `.env`
Copy `.env.example` to `.env`:
```bash
cp .env.example .env
nano .env
```

Ensure the following variables are set:
```ini
TELOXIDE_TOKEN=123456789:ABCdefGhIJKlmNoPQRsTUVwxyZ
DATABASE_URL=sqlite:///app/data/pokemon_bot.db?mode=rwc
ADMIN_IDS=123456789
ADMIN_HOST=0.0.0.0
ADMIN_PORT=5000
ADMIN_PANEL_KEY=your_secure_admin_key_here
POKEDEX_PATH=/app/pokedex.json
ASSETS_DIR=/app/assets
RUST_LOG=info,ozone_pokemon_bot=debug,teloxide=info
RUST_BACKTRACE=1
```

### 4.2 Migrate existing player database (Optional)
If you have an existing `pokemon_bot.db` with active player accounts and Pokémon:
```bash
# On your local machine (PowerShell / Terminal):
scp -i /path/to/your-key.pem pokemon_bot.db ubuntu@<YOUR_EC2_PUBLIC_IP>:/opt/ozone_bot/data/pokemon_bot.db
```
The Docker volume maps `./data:/app/data`, guaranteeing player data persists across container restarts, image rebuilds, and server reboots.

---

## 5. Deployment with Docker Compose (Recommended)

### 5.1 Build and launch the container
```bash
docker compose up -d --build
```

### 5.2 Verify bot status and stream logs
```bash
# Check container status
docker compose ps

# Stream live bot logs
docker compose logs -f
```

### 5.3 Common Docker management commands
```bash
# Stop the bot
docker compose stop

# Restart the bot
docker compose restart

# View recent log output
docker compose logs --tail=100
```

---

## 6. Accessing the Web Admin Panel Securely

The Gym Admin Panel binds to `127.0.0.1:5000` on the EC2 host for security. You can access it securely from your local browser via an SSH tunnel without opening public ports to the Internet:

### 6.1 Create SSH Tunnel (Run on your local computer)
```bash
ssh -i /path/to/your-key.pem -N -L 5000:127.0.0.1:5000 ubuntu@<YOUR_EC2_PUBLIC_IP>
```

### 6.2 Open in browser
Navigate to:
```
http://localhost:5000
```
Enter the key configured in `ADMIN_PANEL_KEY` to manage gyms, leader rosters, and rewards.

---

## 7. Automated Daily Database Backups

A production SQLite backup script is included at `deploy/backup_db.sh`. It performs safe online backups using SQLite's backup API without locking the database or interrupting player battles.

### 7.1 Test the backup script
```bash
chmod +x deploy/backup_db.sh
./deploy/backup_db.sh
```
Backups are saved to `/opt/ozone_bot/backups/pokemon_bot_backup_<timestamp>.sqlite.gz`.

### 7.2 Configure automated daily Cron Job
Edit crontab:
```bash
crontab -e
```
Add the following line to run daily backups at 3:00 AM UTC:
```cron
0 3 * * * /opt/ozone_bot/deploy/backup_db.sh >> /opt/ozone_bot/backups/backup.log 2>&1
```

### 7.3 Optional: Upload backups to AWS S3
To store backups offsite in an AWS S3 bucket:
1. Attach an IAM role with S3 write permissions to your EC2 instance.
2. In `.env`, add:
   ```ini
   S3_BACKUP_BUCKET=my-pokemon-bot-backups
   ```
The backup script will automatically upload new compressed backups to your S3 bucket.

---

## 8. Alternative: Native Systemd Deployment (Without Docker)

If you prefer to compile and run the Rust binary directly on EC2:

### 8.1 Build the release binary
```bash
# Install Rust toolchain on EC2
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"

# Build optimized binary
cargo build --release --bin ozone_pokemon_bot
cp target/release/ozone_pokemon_bot /opt/ozone_bot/ozone_pokemon_bot
```

### 8.2 Install and enable the Systemd service
```bash
sudo cp deploy/ozone_bot.service /etc/systemd/system/ozone_bot.service
sudo systemctl daemon-reload
sudo systemctl enable ozone_bot
sudo systemctl start ozone_bot
```

### 8.3 Check service status & logs
```bash
sudo systemctl status ozone_bot
sudo journalctl -u ozone_bot -f
```

---

## 9. Updating the Bot (Zero Data Loss)

When pushing code updates to GitHub:

```bash
cd /opt/ozone_bot

# 1. Pull latest code
git pull origin master

# 2. Rebuild and restart container seamlessly
docker compose up -d --build

# 3. Verify logs
docker compose logs -f --tail=50
```
Your SQLite database in `./data/pokemon_bot.db` is completely decoupled from the container and remains intact.

---

## 10. Troubleshooting

| Issue | Diagnosis | Solution |
| :--- | :--- | :--- |
| **`401 Unauthorized` / Bot doesn't respond** | Invalid Telegram token in `.env`. | Verify `TELOXIDE_TOKEN` matches token from `@BotFather`. |
| **Container stops immediately** | Missing `pokedex.json` or database path error. | Check `docker compose logs`. Ensure `DATABASE_URL` path has write permissions. |
| **Out of memory during `cargo build`** | Instance exhausted physical RAM. | Verify swap is active with `free -h`. Run `./deploy/setup_aws_ec2.sh` to enable swap. |
| **Trainer cards not generating** | Python Pillow missing. | The runner container includes `python3-pil`. In native mode, run `sudo apt install python3-pil`. |
| **Admin panel unreachable** | Port 5000 not bound or blocked. | Access via SSH tunnel (`ssh -L 5000:127.0.0.1:5000 ubuntu@ip`) and verify `ADMIN_HOST=0.0.0.0`. |

