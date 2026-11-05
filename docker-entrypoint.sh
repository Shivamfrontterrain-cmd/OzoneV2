#!/bin/sh
set -e

# Ensure persistent data directory and temporary scratch paths exist
mkdir -p /app/data /tmp/ozone_trainer_cards

# Safe migration: If existing pokemon_bot.db exists in /app but not in /app/data, copy safely
if [ -f "/app/pokemon_bot.db" ] && [ ! -f "/app/data/pokemon_bot.db" ]; then
    echo "[INFO] Migrating existing pokemon_bot.db to persistent /app/data/pokemon_bot.db..."
    cp -n /app/pokemon_bot.db /app/data/pokemon_bot.db || true
fi

# Run the provided command (default: ozone_pokemon_bot)
exec "$@"

