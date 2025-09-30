#!/bin/bash

# SecretFS Unmount Script
# Usage: ./unmount_ephemfs.sh [mount_point]

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Default mount point
DEFAULT_MOUNT_POINT="/tmp/secrets"

MOUNT_POINT="${1:-$DEFAULT_MOUNT_POINT}"

echo -e "${YELLOW}🔓 Unmounting SecretFS from: $MOUNT_POINT${NC}"

if mountpoint -q "$MOUNT_POINT" 2>/dev/null; then
    fusermount -u "$MOUNT_POINT"
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✅ Successfully unmounted $MOUNT_POINT${NC}"
        echo -e "${GREEN}🧹 All secrets cleared from memory${NC}"
    else
        echo -e "${RED}❌ Failed to unmount $MOUNT_POINT${NC}"
        exit 1
    fi
else
    echo -e "${YELLOW}⚠️  No filesystem mounted at $MOUNT_POINT${NC}"
fi
