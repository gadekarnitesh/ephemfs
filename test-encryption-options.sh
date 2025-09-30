#!/bin/bash

# SecretFS Encryption Options Demo
# This script demonstrates all three encryption modes available in SecretFS

set -e

echo "🔐 SecretFS Encryption Options Demo"
echo "===================================="
echo ""

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Cleanup function
cleanup() {
    echo ""
    echo "🧹 Cleaning up..."
    fusermount -u /tmp/secretfs_demo_default 2>/dev/null || true
    fusermount -u /tmp/secretfs_demo_plaintext 2>/dev/null || true
    fusermount -u /tmp/secretfs_demo_rsa 2>/dev/null || true
    rm -rf /tmp/secretfs_demo_default /tmp/secretfs_demo_plaintext /tmp/secretfs_demo_rsa
    echo "✅ Cleanup complete"
}

# Set trap for cleanup
trap cleanup EXIT

# Create mount directories
mkdir -p /tmp/secretfs_demo_default
mkdir -p /tmp/secretfs_demo_plaintext
mkdir -p /tmp/secretfs_demo_rsa

echo -e "${BLUE}1. Default Encryption (XOR Cipher)${NC}"
echo "   ✅ Works out of the box - no configuration needed"
echo "   ✅ Good for development and testing"
echo "   ⚠️  Demo-level security only"
echo ""

# Test default encryption
DATABASE_PASSWORD="default_secret_123" \
API_KEY="default_api_456" \
./target/release/ephemfs /tmp/secretfs_demo_default &
DEFAULT_PID=$!

sleep 3

echo "   📁 Files created:"
ls -la /tmp/secretfs_demo_default/
echo ""
echo "   📖 Reading secrets (decrypted automatically):"
echo "   • database_password: $(cat /tmp/secretfs_demo_default/database_password)"
echo "   • api_key: $(cat /tmp/secretfs_demo_default/api_key)"

# Stop default encryption
fusermount -u /tmp/secretfs_demo_default
wait $DEFAULT_PID 2>/dev/null || true

echo ""
echo -e "${BLUE}2. Plaintext Mode (No Encryption)${NC}"
echo "   ✅ No encryption overhead"
echo "   ✅ Easy debugging and development"
echo "   ❌ No security - secrets visible to all processes"
echo "   ⚠️  Never use in production"
echo ""

# Test plaintext mode
SECRETFS_CIPHER_TYPE=plaintext \
DATABASE_PASSWORD="plaintext_secret_123" \
API_KEY="plaintext_api_456" \
./target/release/ephemfs /tmp/secretfs_demo_plaintext &
PLAINTEXT_PID=$!

sleep 3

echo "   📁 Files created:"
ls -la /tmp/secretfs_demo_plaintext/
echo ""
echo "   📖 Reading secrets (stored in plaintext):"
echo "   • database_password: $(cat /tmp/secretfs_demo_plaintext/database_password)"
echo "   • api_key: $(cat /tmp/secretfs_demo_plaintext/api_key)"

# Stop plaintext mode
fusermount -u /tmp/secretfs_demo_plaintext
wait $PLAINTEXT_PID 2>/dev/null || true

echo ""
echo -e "${BLUE}3. RSA Asymmetric Encryption (Production)${NC}"
echo "   ✅ Production-grade security"
echo "   ✅ Application-level access control"
echo "   ✅ Only authorized apps can decrypt secrets"
echo "   ✅ System tools (cat, grep) cannot read secrets"
echo ""

# Check if RSA keys exist, generate if needed
if [[ ! -f /tmp/secretfs_demo/private.pem ]]; then
    echo "   🔑 Generating RSA key pair..."
    mkdir -p /tmp/secretfs_demo
    ./target/release/secretfs-keygen generate /tmp/secretfs_demo/private.pem /tmp/secretfs_demo/public.pem
    echo "   ✅ RSA keys generated"
    echo ""
fi

# Test RSA encryption
SECRETFS_CIPHER_TYPE=rsa \
SECRETFS_PUBLIC_KEY_FILE=/tmp/secretfs_demo/public.pem \
DATABASE_PASSWORD="rsa_secret_123" \
API_KEY="rsa_api_456" \
./target/release/ephemfs /tmp/secretfs_demo_rsa &
RSA_PID=$!

sleep 3

echo "   📁 Files created:"
ls -la /tmp/secretfs_demo_rsa/
echo ""
echo "   🔒 System tools see encrypted data:"
echo "   • cat database_password (first 50 bytes):"
cat /tmp/secretfs_demo_rsa/database_password | head -c 50
echo ""
echo "     ⚠️  This is encrypted binary data - unreadable!"
echo ""

echo "   🔓 Authorized application can decrypt:"
SECRETFS_MOUNT_PATH=/tmp/secretfs_demo_rsa \
SECRETFS_PRIVATE_KEY_FILE=/tmp/secretfs_demo/private.pem \
cargo run --example rust_decrypt_demo --quiet 2>/dev/null | grep "Secret:" | head -2

# Stop RSA encryption
fusermount -u /tmp/secretfs_demo_rsa
wait $RSA_PID 2>/dev/null || true

echo ""
echo -e "${GREEN}🎉 Demo Complete!${NC}"
echo ""
echo -e "${YELLOW}Summary of Encryption Options:${NC}"
echo ""
echo "┌─────────────────┬─────────────────┬─────────────────┬─────────────────┐"
echo "│ Mode            │ Configuration   │ Security Level  │ Use Case        │"
echo "├─────────────────┼─────────────────┼─────────────────┼─────────────────┤"
echo "│ Default (XOR)   │ None required   │ Demo/Dev        │ Development     │"
echo "│ Plaintext       │ CIPHER_TYPE=    │ None            │ Local dev only  │"
echo "│                 │ plaintext       │                 │                 │"
echo "│ RSA Asymmetric  │ CIPHER_TYPE=rsa │ Production      │ Production      │"
echo "│                 │ + Key files     │                 │                 │"
echo "└─────────────────┴─────────────────┴─────────────────┴─────────────────┘"
echo ""
echo -e "${YELLOW}Environment Variables:${NC}"
echo "• SECRETFS_CIPHER_TYPE - 'default', 'plaintext', or 'rsa'"
echo "• SECRETFS_PUBLIC_KEY_FILE - Path to RSA public key (for RSA mode)"
echo "• SECRETFS_PRIVATE_KEY_FILE - Path to RSA private key (for applications)"
echo ""
echo -e "${YELLOW}Key Generation:${NC}"
echo "• ./target/release/secretfs-keygen generate private.pem public.pem"
echo ""
echo -e "${YELLOW}Documentation:${NC}"
echo "• ENCRYPTION_OPTIONS.md - Complete encryption guide"
echo "• RSA_ENCRYPTION.md - Detailed RSA setup and usage"
echo "• README.md - General SecretFS documentation"
echo ""
echo "✅ RSA encryption is completely optional - SecretFS works great without it!"
