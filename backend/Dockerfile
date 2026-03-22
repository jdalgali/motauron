# Build Stage
FROM rust:1.88-slim-bookworm AS builder

RUN apt-get update && apt-get install -y pkg-config libssl-dev protobuf-compiler

WORKDIR /app
COPY . .

RUN cargo build --release

# Runtime Stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    chromium \
    fonts-liberation \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/motauron /usr/local/bin/motauron

# Fallback config with safe defaults (real values come from env vars at runtime).
COPY config.toml.example /app/config.toml

# Tell motoscout where to find Chromium.
ENV CHROME_PATH=/usr/bin/chromium

# Required env vars at runtime:
#   NTFY_URL                        — ntfy.sh topic URL for push notifications
#   NTFY_TOKEN                      — (optional) ntfy auth token
#   GOOGLE_APPLICATION_CREDENTIALS  — path to GCP service account key, OR
#                                     leave unset on Cloud Run (workload identity used automatically)
#
# Deployment: run as a Cloud Run Job triggered by Cloud Scheduler every 4 hours.
# Do NOT pass --daemon; Cloud Scheduler handles the interval.

ENTRYPOINT ["motauron"]
