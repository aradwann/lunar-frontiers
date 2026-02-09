# Stage 1: Build
FROM rust:1.93-bookworm AS builder

WORKDIR /app

# Copy manifests first for dependency caching
COPY Cargo.toml Cargo.lock ./

# Create a dummy src to build dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    echo "" > src/lib.rs && \
    cargo build --release 2>/dev/null || true && \
    rm -rf src && \
    rm -f target/release/deps/lunar_frontiers* && \
    rm -f target/release/deps/liblunar_frontiers* && \
    rm -f target/release/lunar-frontiers && \
    rm -f target/release/liblunar_frontiers*

# Copy actual source code and SQLx offline data
COPY src/ src/
COPY migrations/ migrations/
COPY .sqlx/ .sqlx/

# Build the real application with offline SQLx checks
ENV SQLX_OFFLINE=true
RUN cargo build --release

# Stage 2: Runtime
FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates libssl3 && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/lunar-frontiers /usr/local/bin/lunar-frontiers
COPY migrations/ /app/migrations/

WORKDIR /app

ENV RUST_LOG=info

ENTRYPOINT ["lunar-frontiers"]
