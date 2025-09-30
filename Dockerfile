FROM ubuntu:22.04

# Install system dependencies
RUN apt-get update && apt-get install -y \
    curl \
    build-essential \
    pkg-config \
    libssl-dev \
    libfuse3-dev \
    fuse3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Install Rust nightly via rustup
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain nightly
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /app

# Copy source
COPY Cargo.toml .
COPY src/ ./src/

# Build release binary with nightly (edition2024 support)
RUN cargo +nightly build --release

CMD ["/app/target/release/secretfs", "/secrets"]
