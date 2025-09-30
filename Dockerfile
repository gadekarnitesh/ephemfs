# Multi-stage build for Rust FUSE secrets filesystem
FROM rust:1.75-slim as builder

# Install system dependencies for building
RUN apt-get update && apt-get install -y \
    pkg-config \
    libfuse3-dev \
    fuse3 \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy Cargo files first for better caching
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src/ ./src/

# Build the application in release mode
RUN cargo build --release --verbose

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    fuse3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && mkdir -p /mnt/secrets

# Copy the binary from builder stage
COPY --from=builder /app/target/release/ephemfs /usr/local/bin/secretfs
COPY --from=builder /app/target/release/secretfs-keygen /usr/local/bin/secretfs-keygen

# Create user (UID 1000 to match common K8s setups)
RUN useradd -u 1000 -r -s /bin/bash -m -d /home/secretfs secretfs

# Set default environment variables
ENV FUSE_MOUNTPOINT=/secrets
ENV RUST_LOG=info

# Expose the mount point as a volume
VOLUME ["/secrets"]

# Default command (will be overridden by K8s if needed)
CMD ["secretfs", "/secrets"]
