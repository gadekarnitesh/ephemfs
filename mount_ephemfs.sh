#!/bin/bash

# SecretFS Mount Script
# Mounts secrets from environment variables as encrypted files in memory-only storage
# Usage: ./mount_ephemfs.sh [mount_point] [options]

set -e

# Default mount point
DEFAULT_MOUNT_POINT="/tmp/secrets"

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Show usage
show_usage() {
    echo "SecretFS - Encrypted Memory-Only FUSE Filesystem for Kubernetes Secrets"
    echo ""
    echo "Usage: $0 [mount_point] [options]"
    echo ""
    echo "Arguments:"
    echo "  mount_point          Directory to mount secrets (default: /tmp/secrets)"
    echo ""
    echo "Options:"
    echo "  --help, -h           Show this help message"
    echo "  --release            Use release build (optimized)"
    echo "  --build              Force rebuild before mounting"
    echo ""
    echo "Environment Variables:"
    echo "  Secrets (will be mounted as files):"
    echo "    DATABASE_PASSWORD, API_KEY, JWT_SECRET, etc."
    echo "    SECRET_<NAME> - Custom secrets (e.g., SECRET_STRIPE_KEY)"
    echo ""
    echo "  Encryption Configuration:"
    echo "    SECRETFS_CIPHER_TYPE - 'default', 'plaintext', or 'rsa'"
    echo "    SECRETFS_ENCRYPTION_KEY - Encryption key (for default cipher)"
    echo "    SECRETFS_PUBLIC_KEY_FILE - RSA public key file (for RSA mode)"
    echo ""
    echo "  External Secret Fetching:"
    echo "    SECRETFS_URLS - Comma-separated URLs to fetch secrets from"
    echo "    SECRETFS_AUTH_TOKEN - Bearer token for API authentication"
    echo ""
    echo "Examples:"
    echo "  # Basic usage with default encryption"
    echo "  DATABASE_PASSWORD='secret123' API_KEY='sk-test' $0"
    echo ""
    echo "  # With custom mount point"
    echo "  DATABASE_PASSWORD='secret123' $0 /mnt/secrets"
    echo ""
    echo "  # With RSA encryption (production)"
    echo "  SECRETFS_CIPHER_TYPE=rsa \\"
    echo "  SECRETFS_PUBLIC_KEY_FILE=public.pem \\"
    echo "  DATABASE_PASSWORD='secret123' $0 /mnt/secrets --release"
    echo ""
    echo "  # With external secret fetching"
    echo "  SECRETFS_URLS='https://vault.example.com/v1/secret' \\"
    echo "  SECRETFS_AUTH_TOKEN='hvs.token123' $0"
    echo ""
    echo "Documentation:"
    echo "  README.md - General documentation"
    echo "  ENCRYPTION_OPTIONS.md - Encryption setup guide"
    echo "  RSA_ENCRYPTION.md - RSA encryption details"
    echo ""
}

# Parse arguments
MOUNT_POINT="$DEFAULT_MOUNT_POINT"
USE_RELEASE=false
FORCE_BUILD=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --help|-h)
            show_usage
            exit 0
            ;;
        --release)
            USE_RELEASE=true
            shift
            ;;
        --build)
            FORCE_BUILD=true
            shift
            ;;
        -*)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
        *)
            MOUNT_POINT="$1"
            shift
            ;;
    esac
done

# Determine binary path
if [ "$USE_RELEASE" = true ]; then
    BINARY="./target/release/ephemfs"
    BUILD_CMD="cargo build --release"
    BUILD_TYPE="release"
else
    BINARY="./target/debug/ephemfs"
    BUILD_CMD="cargo build"
    BUILD_TYPE="debug"
fi

# Build the project if needed
if [ "$FORCE_BUILD" = true ] || [ ! -f "$BINARY" ] || [ "src/main.rs" -nt "$BINARY" ]; then
    echo -e "${BLUE}🔨 Building SecretFS ($BUILD_TYPE)...${NC}"
    $BUILD_CMD
    echo -e "${GREEN}✅ Build complete${NC}"
    echo ""
fi

# Create mount point if it doesn't exist
if [ ! -d "$MOUNT_POINT" ]; then
    echo -e "${BLUE}📁 Creating mount point: $MOUNT_POINT${NC}"
    mkdir -p "$MOUNT_POINT"
fi

# Check if mount point is already in use
if mountpoint -q "$MOUNT_POINT" 2>/dev/null; then
    echo -e "${YELLOW}⚠️  $MOUNT_POINT is already mounted. Unmounting first...${NC}"
    fusermount -u "$MOUNT_POINT" || {
        echo -e "${RED}❌ Failed to unmount $MOUNT_POINT${NC}"
        exit 1
    }
fi

# Check if any secrets are configured
SECRET_COUNT=0
for var in DATABASE_PASSWORD API_KEY JWT_SECRET REDIS_PASSWORD VAULT_TOKEN CONFIG_JSON $(env | grep '^SECRET_' | cut -d= -f1); do
    if [ ! -z "${!var}" ]; then
        ((SECRET_COUNT++))
    fi
done

if [ $SECRET_COUNT -eq 0 ] && [ -z "$SECRETFS_URLS" ]; then
    echo -e "${YELLOW}⚠️  No secrets configured!${NC}"
    echo ""
    echo "SecretFS will mount an empty filesystem."
    echo ""
    echo "To add secrets, set environment variables before running this script:"
    echo "  export DATABASE_PASSWORD='your_secret'"
    echo "  export API_KEY='your_api_key'"
    echo "  export SECRET_CUSTOM_KEY='custom_value'"
    echo ""
    echo "Or configure external secret fetching:"
    echo "  export SECRETFS_URLS='https://vault.example.com/v1/secret'"
    echo "  export SECRETFS_AUTH_TOKEN='your_token'"
    echo ""
    read -p "Continue anyway? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 0
    fi
fi

echo -e "${GREEN}🚀 Starting SecretFS${NC}"
echo -e "${BLUE}📍 Mount point: $MOUNT_POINT${NC}"
echo -e "${BLUE}🔐 Encryption: ${SECRETFS_CIPHER_TYPE:-default}${NC}"
if [ $SECRET_COUNT -gt 0 ]; then
    echo -e "${BLUE}📊 Environment secrets: $SECRET_COUNT${NC}"
fi
if [ ! -z "$SECRETFS_URLS" ]; then
    echo -e "${BLUE}🌐 External URLs: $SECRETFS_URLS${NC}"
fi
echo ""
echo -e "${YELLOW}Press Ctrl+C to unmount${NC}"
echo ""

# Mount the filesystem
exec "$BINARY" "$MOUNT_POINT"
