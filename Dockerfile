# Stage 1: Build
FROM rust:1.94-slim-trixie AS builder

RUN apt-get update && \
    apt-get install -y --no-install-recommends pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy manifests first for dependency caching
COPY Cargo.toml Cargo.lock ./

# Create a dummy src to pre-build dependencies
# Errors are shown (not suppressed) so real failures are visible
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    echo "" > src/lib.rs && \
    cargo build --release 2>&1 | tail -5 || true && \
    rm -rf src && \
    rm -f target/release/deps/lunar_frontiers* && \
    rm -f target/release/deps/liblunar_frontiers* && \
    rm -f target/release/lunar-frontiers

# Copy actual source code and SQLx offline data
COPY src/ src/
COPY migrations/ migrations/
COPY .sqlx/ .sqlx/

# Build the real application with offline SQLx checks
ENV SQLX_OFFLINE=true
RUN cargo build --release

# Stage 2: Runtime
FROM debian:trixie-slim@sha256:26f98ccd92fd0a44d6928ce8ff8f4921b4d2f535bfa07555ee5d18f61429cf0c

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        ca-certificates \
        libssl3 \
        curl \
    && rm -rf /var/lib/apt/lists/* \
    && useradd -r -s /bin/false appuser

COPY --from=builder /app/target/release/lunar-frontiers /usr/local/bin/lunar-frontiers
COPY migrations/ /app/migrations/

WORKDIR /app
RUN chown -R appuser:appuser /app

USER appuser

ENV RUST_LOG=info

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

ENTRYPOINT ["lunar-frontiers"]
