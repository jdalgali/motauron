# Build Stage
FROM rust:1.80-slim-bookworm AS builder

# Install dependencies needed by grpc/firestore (protoc, libssl)
RUN apt-get update && apt-get install -y pkg-config libssl-dev protobuf-compiler

WORKDIR /app
COPY . .

# Build the release binary
RUN cargo build --release

# Runtime Stage
FROM debian:bookworm-slim

# Install runtime dependencies for TLS
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the compiled binary from the builder
COPY --from=builder /app/target/release/motauron /usr/local/bin/motauron

# Copy the default configuration files (if they exist)
COPY config.toml.example /app/config.toml
# Note: Google Cloud Run passes the Service Account implicitly, so we don't copy service-account.json into the image!

ENTRYPOINT ["motauron"]
