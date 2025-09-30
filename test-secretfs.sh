#!/bin/bash

# SecretFS Comprehensive Test Suite
# Tests all features: encryption modes, external fetching, RSA, and security

set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Test results
TESTS_PASSED=0
TESTS_FAILED=0

# Cleanup function
cleanup() {
    echo ""
    echo -e "${BLUE}🧹 Cleaning up...${NC}"
    fusermount -u /tmp/secretfs_test_default 2>/dev/null || true
    fusermount -u /tmp/secretfs_test_plaintext 2>/dev/null || true
    fusermount -u /tmp/secretfs_test_rsa 2>/dev/null || true
    fusermount -u /tmp/secretfs_test_external 2>/dev/null || true
    rm -rf /tmp/secretfs_test_* /tmp/secretfs_keys 2>/dev/null || true
    echo -e "${GREEN}✅ Cleanup complete${NC}"
}

# Set trap for cleanup
trap cleanup EXIT

# Test result tracking
pass_test() {
    echo -e "${GREEN}✅ PASS: $1${NC}"
    ((TESTS_PASSED++))
}

fail_test() {
    echo -e "${RED}❌ FAIL: $1${NC}"
    ((TESTS_FAILED++))
}

echo -e "${BLUE}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║         SecretFS Comprehensive Test Suite                 ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Build the project
echo -e "${BLUE}🔨 Building SecretFS...${NC}"
cargo build --release --quiet 2>/dev/null || cargo build --release
echo -e "${GREEN}✅ Build complete${NC}"
echo ""

# Test 1: Default Encryption
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}Test 1: Default Encryption (XOR Cipher)${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

mkdir -p /tmp/secretfs_test_default
DATABASE_PASSWORD="test_default_123" \
API_KEY="test_api_456" \
./target/release/ephemfs /tmp/secretfs_test_default &
PID=$!
sleep 3

if [ -f /tmp/secretfs_test_default/database_password ]; then
    CONTENT=$(cat /tmp/secretfs_test_default/database_password)
    if [ "$CONTENT" = "test_default_123" ]; then
        pass_test "Default encryption - secret readable"
    else
        fail_test "Default encryption - wrong content: $CONTENT"
    fi
else
    fail_test "Default encryption - file not created"
fi

fusermount -u /tmp/secretfs_test_default
wait $PID 2>/dev/null || true
echo ""

# Test 2: Plaintext Mode
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}Test 2: Plaintext Mode (No Encryption)${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

mkdir -p /tmp/secretfs_test_plaintext
SECRETFS_CIPHER_TYPE=plaintext \
DATABASE_PASSWORD="test_plaintext_123" \
./target/release/ephemfs /tmp/secretfs_test_plaintext &
PID=$!
sleep 3

if [ -f /tmp/secretfs_test_plaintext/database_password ]; then
    CONTENT=$(cat /tmp/secretfs_test_plaintext/database_password)
    if [ "$CONTENT" = "test_plaintext_123" ]; then
        pass_test "Plaintext mode - secret readable"
    else
        fail_test "Plaintext mode - wrong content: $CONTENT"
    fi
else
    fail_test "Plaintext mode - file not created"
fi

fusermount -u /tmp/secretfs_test_plaintext
wait $PID 2>/dev/null || true
echo ""

# Test 3: RSA Encryption
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}Test 3: RSA Asymmetric Encryption${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

# Generate RSA keys
mkdir -p /tmp/secretfs_keys
./target/release/secretfs-keygen generate /tmp/secretfs_keys/private.pem /tmp/secretfs_keys/public.pem >/dev/null 2>&1

mkdir -p /tmp/secretfs_test_rsa
SECRETFS_CIPHER_TYPE=rsa \
SECRETFS_PUBLIC_KEY_FILE=/tmp/secretfs_keys/public.pem \
DATABASE_PASSWORD="test_rsa_123" \
./target/release/ephemfs /tmp/secretfs_test_rsa &
PID=$!
sleep 3

if [ -f /tmp/secretfs_test_rsa/database_password ]; then
    # Check that cat shows encrypted data (not plaintext)
    CONTENT=$(cat /tmp/secretfs_test_rsa/database_password 2>/dev/null | head -c 20)
    if [[ "$CONTENT" != "test_rsa_123" ]]; then
        pass_test "RSA encryption - cat shows encrypted data"
    else
        fail_test "RSA encryption - cat shows plaintext (should be encrypted)"
    fi
    
    # Check file size (RSA encrypted should be 256 bytes for 2048-bit key)
    SIZE=$(stat -c%s /tmp/secretfs_test_rsa/database_password 2>/dev/null || stat -f%z /tmp/secretfs_test_rsa/database_password 2>/dev/null)
    if [ "$SIZE" = "256" ]; then
        pass_test "RSA encryption - correct encrypted size (256 bytes)"
    else
        fail_test "RSA encryption - wrong size: $SIZE (expected 256)"
    fi
else
    fail_test "RSA encryption - file not created"
fi

fusermount -u /tmp/secretfs_test_rsa
wait $PID 2>/dev/null || true
echo ""

# Test 4: Custom Secrets (SECRET_* pattern)
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}Test 4: Custom Secrets (SECRET_* pattern)${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

mkdir -p /tmp/secretfs_test_custom
SECRET_STRIPE_KEY="sk_test_stripe_123" \
SECRET_SENDGRID_API="sg_api_456" \
./target/release/ephemfs /tmp/secretfs_test_custom &
PID=$!
sleep 3

if [ -f /tmp/secretfs_test_custom/stripe-key ]; then
    CONTENT=$(cat /tmp/secretfs_test_custom/stripe-key)
    if [ "$CONTENT" = "sk_test_stripe_123" ]; then
        pass_test "Custom secrets - SECRET_STRIPE_KEY → stripe-key"
    else
        fail_test "Custom secrets - wrong content"
    fi
else
    fail_test "Custom secrets - stripe-key not created"
fi

if [ -f /tmp/secretfs_test_custom/sendgrid-api ]; then
    pass_test "Custom secrets - SECRET_SENDGRID_API → sendgrid-api"
else
    fail_test "Custom secrets - sendgrid-api not created"
fi

fusermount -u /tmp/secretfs_test_custom
wait $PID 2>/dev/null || true
rm -rf /tmp/secretfs_test_custom
echo ""

# Test 5: Read-only Filesystem
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}Test 5: Read-only Filesystem Security${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

mkdir -p /tmp/secretfs_test_readonly
DATABASE_PASSWORD="test_readonly_123" \
./target/release/ephemfs /tmp/secretfs_test_readonly &
PID=$!
sleep 3

# Try to write to a file (should fail)
if echo "hacked" > /tmp/secretfs_test_readonly/database_password 2>/dev/null; then
    fail_test "Read-only - write succeeded (should fail)"
else
    pass_test "Read-only - write blocked correctly"
fi

# Try to create a new file (should fail)
if touch /tmp/secretfs_test_readonly/newfile 2>/dev/null; then
    fail_test "Read-only - file creation succeeded (should fail)"
else
    pass_test "Read-only - file creation blocked correctly"
fi

# Try to delete a file (should fail)
if rm /tmp/secretfs_test_readonly/database_password 2>/dev/null; then
    fail_test "Read-only - deletion succeeded (should fail)"
else
    pass_test "Read-only - deletion blocked correctly"
fi

fusermount -u /tmp/secretfs_test_readonly
wait $PID 2>/dev/null || true
rm -rf /tmp/secretfs_test_readonly
echo ""

# Test 6: File Permissions
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}Test 6: File Permissions (0600)${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

mkdir -p /tmp/secretfs_test_perms
DATABASE_PASSWORD="test_perms_123" \
./target/release/ephemfs /tmp/secretfs_test_perms &
PID=$!
sleep 3

PERMS=$(stat -c%a /tmp/secretfs_test_perms/database_password 2>/dev/null || stat -f%Lp /tmp/secretfs_test_perms/database_password 2>/dev/null)
if [ "$PERMS" = "600" ]; then
    pass_test "File permissions - correct (0600)"
else
    fail_test "File permissions - wrong: $PERMS (expected 600)"
fi

fusermount -u /tmp/secretfs_test_perms
wait $PID 2>/dev/null || true
rm -rf /tmp/secretfs_test_perms
echo ""

# Final Results
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}Test Results Summary${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo -e "${GREEN}✅ Tests Passed: $TESTS_PASSED${NC}"
if [ $TESTS_FAILED -gt 0 ]; then
    echo -e "${RED}❌ Tests Failed: $TESTS_FAILED${NC}"
else
    echo -e "${GREEN}❌ Tests Failed: $TESTS_FAILED${NC}"
fi
echo ""

TOTAL=$((TESTS_PASSED + TESTS_FAILED))
if [ $TOTAL -gt 0 ]; then
    PERCENTAGE=$((TESTS_PASSED * 100 / TOTAL))
    echo -e "${BLUE}Success Rate: $PERCENTAGE%${NC}"
fi
echo ""

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}🎉 All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}⚠️  Some tests failed${NC}"
    exit 1
fi

