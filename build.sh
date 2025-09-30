#!/bin/bash

# Build script for SecretFS
set -e

echo "🔧 Building SecretFS..."

# Build the Rust binary
echo "📦 Building Rust binary..."
cargo build --release

echo "✅ Rust binary built successfully"

# Check if Docker is available
if command -v docker &> /dev/null; then
    echo "🐳 Building Docker image..."
    docker build -t secret-fuse:latest .
    echo "✅ Docker image built successfully"
    
    # Tag for different registries (optional)
    echo "🏷️  Tagging image..."
    docker tag secret-fuse:latest ghcr.io/yourorg/secret-fuse:latest
    docker tag secret-fuse:latest secret-fuse:$(date +%Y%m%d)
    
    echo "📋 Available Docker images:"
    docker images | grep secret-fuse
else
    echo "⚠️  Docker not available, skipping Docker build"
fi

echo ""
echo "🎉 Build complete!"
echo ""
echo "Usage examples:"
echo "  # Standalone:"
echo "  ./target/release/ephemfs /tmp/secrets"
echo ""
echo "  # With environment variables:"
echo "  DATABASE_PASSWORD=secret123 ./target/release/ephemfs /tmp/secrets"
echo ""
echo "  # Docker Compose:"
echo "  docker-compose up --build"
echo ""
echo "  # Kubernetes:"
echo "  kubectl apply -f k8s-secret-fuse-pod.yaml"
