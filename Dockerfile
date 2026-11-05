# ==============================================================================
# Multi-Stage Production Dockerfile for Ozone Pokémon Telegram Bot
# ==============================================================================

# ------------------------------------------------------------------------------
# Stage 1: Build stage
# ------------------------------------------------------------------------------
FROM rust:bookworm AS builder

WORKDIR /usr/src/ozone_pokemon_bot

# Install native compilation dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    libsqlite3-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy dependency manifests
COPY Cargo.toml Cargo.lock* ./

# Cache cargo dependencies with dummy main.rs
RUN mkdir -p src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy actual source code
COPY src ./src

# Build the release binary
RUN touch src/main.rs && cargo build --release --bin ozone_pokemon_bot

# ------------------------------------------------------------------------------
# Stage 2: Runtime stage
# ------------------------------------------------------------------------------
FROM debian:bookworm-slim AS runner

WORKDIR /app

# Install runtime dependencies:
# - ca-certificates: Required for HTTPS requests to Telegram Bot API
# - libssl3: Required by OpenSSL/HTTPS
# - libsqlite3-0: Native SQLite3 runtime library
# - python3 & python3-pil: Required by assets/generate_trainer_card.py
# - tzdata: For accurate local timezones
# - curl: For container health checks
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    libsqlite3-0 \
    python3 \
    python3-pil \
    tzdata \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Copy release binary from builder
COPY --from=builder /usr/src/ozone_pokemon_bot/target/release/ozone_pokemon_bot /usr/local/bin/ozone_pokemon_bot

# Copy static game data and assets
COPY pokedex.json ./pokedex.json
COPY assets ./assets
COPY docker-entrypoint.sh /usr/local/bin/docker-entrypoint.sh
RUN chmod +x /usr/local/bin/docker-entrypoint.sh

# Create persistent storage directories
RUN mkdir -p /app/data /tmp/ozone_trainer_cards

# Default environment configuration
ENV RUST_LOG="info,ozone_pokemon_bot=debug,teloxide=info" \
    RUST_BACKTRACE="1" \
    DATABASE_URL="sqlite:///app/data/pokemon_bot.db?mode=rwc" \
    POKEDEX_PATH="/app/pokedex.json" \
    ASSETS_DIR="/app/assets" \
    ADMIN_HOST="0.0.0.0" \
    ADMIN_PORT="5000"

# Expose Gym Admin Panel
EXPOSE 5000

# Persistent volume for SQLite database
VOLUME ["/app/data"]

ENTRYPOINT ["docker-entrypoint.sh"]
CMD ["ozone_pokemon_bot"]

