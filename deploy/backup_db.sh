#!/usr/bin/env bash
set -euo pipefail

# ==============================================================================
# Automated SQLite Database Backup Script for Ozone Pokémon Bot
# Uses SQLite Online Backup API to safely backup live database without downtime
# ==============================================================================

# Directory where persistent DB lives (default: ./data/pokemon_bot.db or /opt/ozone_bot/data/pokemon_bot.db)
APP_DIR="${APP_DIR:-/opt/ozone_bot}"
DB_PATH="${DB_PATH:-${APP_DIR}/data/pokemon_bot.db}"

# If data/pokemon_bot.db does not exist, check root pokemon_bot.db
if [ ! -f "$DB_PATH" ] && [ -f "${APP_DIR}/pokemon_bot.db" ]; then
    DB_PATH="${APP_DIR}/pokemon_bot.db"
fi

BACKUP_DIR="${APP_DIR}/backups"
TIMESTAMP="$(date +'%Y%m%d_%H%M%S')"
BACKUP_FILE="${BACKUP_DIR}/pokemon_bot_backup_${TIMESTAMP}.sqlite"
GZ_FILE="${BACKUP_FILE}.gz"

# Create backup directory
mkdir -p "$BACKUP_DIR"

if [ ! -f "$DB_PATH" ]; then
    echo "[ERROR] Database not found at ${DB_PATH} - skipping backup."
    exit 1
fi

echo "[INFO] Starting safe online backup of ${DB_PATH}..."

# Safe online backup using SQLite VACUUM INTO or .backup command
if command -v sqlite3 &>/dev/null; then
    sqlite3 "$DB_PATH" ".backup '${BACKUP_FILE}'"
else
    # Fallback to copy if sqlite3 CLI is not installed
    cp "$DB_PATH" "$BACKUP_FILE"
fi

# Compress the backup file to save disk space
gzip -f "$BACKUP_FILE"
echo "[INFO] Backup created successfully: ${GZ_FILE} ($(du -h "${GZ_FILE}" | cut -f1))"

# Retention policy: Delete backups older than 14 days
find "$BACKUP_DIR" -type f -name "pokemon_bot_backup_*.sqlite.gz" -mtime +14 -exec rm -f {} \;
echo "[INFO] Cleaned up backups older than 14 days."

# Optional: Upload to AWS S3 if S3_BACKUP_BUCKET environment variable is defined
if [ -n "${S3_BACKUP_BUCKET:-}" ] && command -v aws &>/dev/null; then
    echo "[INFO] Uploading backup to s3://${S3_BACKUP_BUCKET}/backups/..."
    aws s3 cp "${GZ_FILE}" "s3://${S3_BACKUP_BUCKET}/backups/$(basename "${GZ_FILE}")"
    echo "[INFO] S3 upload completed."
fi

