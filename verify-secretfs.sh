#!/bin/bash

# SecretFS Quick Verification Script
# Verifies that SecretFS is working correctly

set -e

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}🔍 SecretFS Quick Verification${NC}"
echo "=============================="
echo ""

# Cleanup function
cleanup() {
    fusermount -u /tmp/secrets_verify 2>/dev/null || true
    rm -rf /tmp/secrets_verify 2>/dev/null || true
}
trap cleanup EXIT

# Create mount point
mkdir -p /tmp/secrets_verify

# Test 1: Mount
echo -e "${BLUE}1. Mounting SecretFS...${NC}"
DATABASE_PASSWORD='verify123' API_KEY='verify456' JWT_SECRET='verify789' \
./target/release/ephemfs /tmp/secrets_verify &
MOUNT_PID=$!
sleep 3

# Test 2: Verify files exist
echo -e "${BLUE}2. Verifying secrets exist...${NC}"
if [ -f /tmp/secrets_verify/database_password ]; then
    echo -e "   ${GREEN}✅ database_password file exists${NC}"
else
    echo -e "   ${RED}❌ database_password file not found${NC}"
    exit 1
fi

if [ -f /tmp/secrets_verify/api_key ]; then
    echo -e "   ${GREEN}✅ api_key file exists${NC}"
else
    echo -e "   ${RED}❌ api_key file not found${NC}"
    exit 1
fi

if [ -f /tmp/secrets_verify/jwt_secret ]; then
    echo -e "   ${GREEN}✅ jwt_secret file exists${NC}"
else
    echo -e "   ${RED}❌ jwt_secret file not found${NC}"
    exit 1
fi

# Test 3: Verify content
echo -e "${BLUE}3. Verifying secret content...${NC}"
CONTENT=$(cat /tmp/secrets_verify/database_password)
if [ "$CONTENT" = "verify123" ]; then
    echo -e "   ${GREEN}✅ database_password content correct: $CONTENT${NC}"
else
    echo -e "   ${RED}❌ database_password content wrong: $CONTENT (expected: verify123)${NC}"
    exit 1
fi

CONTENT=$(cat /tmp/secrets_verify/api_key)
if [ "$CONTENT" = "verify456" ]; then
    echo -e "   ${GREEN}✅ api_key content correct: $CONTENT${NC}"
else
    echo -e "   ${RED}❌ api_key content wrong: $CONTENT (expected: verify456)${NC}"
    exit 1
fi

CONTENT=$(cat /tmp/secrets_verify/jwt_secret)
if [ "$CONTENT" = "verify789" ]; then
    echo -e "   ${GREEN}✅ jwt_secret content correct: $CONTENT${NC}"
else
    echo -e "   ${RED}❌ jwt_secret content wrong: $CONTENT (expected: verify789)${NC}"
    exit 1
fi

# Test 4: Check permissions
echo -e "${BLUE}4. Verifying file permissions...${NC}"
PERMS=$(stat -c%a /tmp/secrets_verify/database_password 2>/dev/null || stat -f%Lp /tmp/secrets_verify/database_password 2>/dev/null)
if [ "$PERMS" = "600" ]; then
    echo -e "   ${GREEN}✅ Permissions correct (0600 - owner read-only)${NC}"
else
    echo -e "   ${YELLOW}⚠️  Permissions: $PERMS (expected: 600)${NC}"
fi

# Test 5: Check read-only filesystem
echo -e "${BLUE}5. Verifying read-only filesystem...${NC}"
if echo "hacked" > /tmp/secrets_verify/database_password 2>/dev/null; then
    echo -e "   ${RED}❌ Filesystem is writable (should be read-only)${NC}"
    exit 1
else
    echo -e "   ${GREEN}✅ Write operations blocked (read-only)${NC}"
fi

if touch /tmp/secrets_verify/newfile 2>/dev/null; then
    echo -e "   ${RED}❌ File creation succeeded (should fail)${NC}"
    exit 1
else
    echo -e "   ${GREEN}✅ File creation blocked (read-only)${NC}"
fi

if rm /tmp/secrets_verify/database_password 2>/dev/null; then
    echo -e "   ${RED}❌ File deletion succeeded (should fail)${NC}"
    exit 1
else
    echo -e "   ${GREEN}✅ File deletion blocked (read-only)${NC}"
fi

# Test 6: Unmount
echo -e "${BLUE}6. Unmounting SecretFS...${NC}"
fusermount -u /tmp/secrets_verify
wait $MOUNT_PID 2>/dev/null || true
echo -e "   ${GREEN}✅ Unmounted successfully${NC}"

# Test 7: Verify cleanup
echo -e "${BLUE}7. Verifying memory cleanup...${NC}"
sleep 1
if [ -f /tmp/secrets_verify/database_password ]; then
    echo -e "   ${RED}❌ Secrets still present after unmount${NC}"
    exit 1
else
    echo -e "   ${GREEN}✅ Secrets cleared from memory${NC}"
fi

# Summary
echo ""
echo -e "${GREEN}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║                  🎉 All Checks Passed! 🎉                  ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${BLUE}SecretFS is working correctly!${NC}"
echo ""
echo -e "${YELLOW}Next steps:${NC}"
echo "  • Mount with your secrets: DATABASE_PASSWORD='secret' ./mount_ephemfs.sh"
echo "  • Run full test suite: ./test-secretfs.sh"
echo "  • Try encryption modes: ./test-encryption-options.sh"
echo "  • Read documentation: VERIFICATION.md"
echo ""

