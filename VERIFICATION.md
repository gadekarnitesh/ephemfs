# SecretFS Verification Guide

This guide shows you how to verify that SecretFS is working correctly.

## 🔍 Quick Verification

### Step 1: Mount SecretFS

Open a terminal and run:

```bash
DATABASE_PASSWORD='secret123' API_KEY='sk-test-456' ./mount_ephemfs.sh
```

**Expected Output:**
```
🚀 Starting SecretFS
📍 Mount point: /tmp/secrets
🔐 Encryption: default
📊 Environment secrets: 2

Press Ctrl+C to unmount
```

### Step 2: Verify in Another Terminal

Open a **second terminal** and verify the secrets:

```bash
# List the mounted secrets
ls -la /tmp/secrets/

# Expected output:
# -rw------- 1 user user  9 Jan  1  1970 api_key
# -rw------- 1 user user  9 Jan  1  1970 database_password

# Read the secrets
cat /tmp/secrets/database_password
# Output: secret123

cat /tmp/secrets/api_key
# Output: sk-test-456
```

### Step 3: Verify Security Features

```bash
# Check file permissions (should be 0600 - owner read-only)
stat /tmp/secrets/database_password

# Try to write (should fail - read-only filesystem)
echo "hacked" > /tmp/secrets/database_password
# Expected: bash: /tmp/secrets/database_password: Read-only file system

# Try to create new file (should fail)
touch /tmp/secrets/newfile
# Expected: touch: cannot touch '/tmp/secrets/newfile': Read-only file system
```

### Step 4: Unmount

Go back to the **first terminal** and press `Ctrl+C`, or in another terminal:

```bash
./unmount_ephemfs.sh
```

**Expected Output:**
```
🔓 Unmounting SecretFS from: /tmp/secrets
✅ Successfully unmounted /tmp/secrets
🧹 All secrets cleared from memory
```

### Step 5: Verify Cleanup

```bash
# Secrets should be gone
ls /tmp/secrets/
# Output: (empty or directory not found)

cat /tmp/secrets/database_password
# Expected: cat: /tmp/secrets/database_password: No such file or directory
```

---

## 🧪 Automated Verification

Run the comprehensive test suite:

```bash
./test-secretfs.sh
```

**Expected Output:**
```
╔════════════════════════════════════════════════════════════╗
║         SecretFS Comprehensive Test Suite                 ║
╚════════════════════════════════════════════════════════════╝

✅ PASS: Default encryption - secret readable
✅ PASS: Plaintext mode - secret readable
✅ PASS: RSA encryption - cat shows encrypted data
✅ PASS: RSA encryption - correct encrypted size (256 bytes)
✅ PASS: Custom secrets - SECRET_STRIPE_KEY → stripe-key
✅ PASS: Custom secrets - SECRET_SENDGRID_API → sendgrid-api
✅ PASS: Read-only - write blocked correctly
✅ PASS: Read-only - file creation blocked correctly
✅ PASS: Read-only - deletion blocked correctly
✅ PASS: File permissions - correct (0600)

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Test Results Summary
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

✅ Tests Passed: 10
❌ Tests Failed: 0
Success Rate: 100%

🎉 All tests passed!
```

---

## 🔐 Verify Different Encryption Modes

### Default Encryption (XOR)

```bash
# Terminal 1: Mount
DATABASE_PASSWORD='test123' ./mount_ephemfs.sh

# Terminal 2: Verify
cat /tmp/secrets/database_password
# Output: test123 (decrypted automatically)
```

### Plaintext Mode

```bash
# Terminal 1: Mount
SECRETFS_CIPHER_TYPE=plaintext DATABASE_PASSWORD='test123' ./mount_ephemfs.sh

# Terminal 2: Verify
cat /tmp/secrets/database_password
# Output: test123 (stored in plaintext)
```

### RSA Encryption

```bash
# Generate keys first
./target/release/secretfs-keygen generate private.pem public.pem

# Terminal 1: Mount with RSA
SECRETFS_CIPHER_TYPE=rsa \
SECRETFS_PUBLIC_KEY_FILE=public.pem \
DATABASE_PASSWORD='test123' ./mount_ephemfs.sh

# Terminal 2: Verify cat shows encrypted data
cat /tmp/secrets/database_password
# Output: (binary encrypted data - unreadable)

# Terminal 2: Verify with authorized application
SECRETFS_PRIVATE_KEY_FILE=private.pem \
cargo run --example rust_decrypt_demo
# Output: Successfully decrypted: test123
```

---

## 🎯 Verification Checklist

Use this checklist to verify all features:

### Basic Functionality
- [ ] SecretFS mounts successfully
- [ ] Secrets appear as files in mount point
- [ ] Secrets can be read with `cat`
- [ ] Secrets have correct content
- [ ] SecretFS unmounts cleanly

### Security Features
- [ ] File permissions are 0600 (owner read-only)
- [ ] Write operations are blocked
- [ ] File creation is blocked
- [ ] File deletion is blocked
- [ ] Secrets disappear after unmount

### Encryption Modes
- [ ] Default encryption works
- [ ] Plaintext mode works
- [ ] RSA encryption works (if configured)
- [ ] RSA fallback works (when not configured)

### Custom Secrets
- [ ] `SECRET_*` environment variables work
- [ ] Names are converted to kebab-case
- [ ] Standard secrets (DATABASE_PASSWORD, etc.) work

### Memory Safety
- [ ] No disk writes (check with `lsof` or `strace`)
- [ ] Memory is cleared after unmount
- [ ] No secret data in swap (if swap is disabled)

---

## 🔬 Advanced Verification

### Verify No Disk Writes

```bash
# Terminal 1: Mount
DATABASE_PASSWORD='test123' ./mount_ephemfs.sh

# Terminal 2: Monitor disk I/O
sudo iotop -o
# Should show NO disk writes from ephemfs process

# Or use strace
ps aux | grep ephemfs
sudo strace -p <PID> -e trace=write,open,openat 2>&1 | grep -v "/tmp/secrets"
# Should show no file writes outside of FUSE operations
```

### Verify Memory-Only Storage

```bash
# Check that secrets are not in any files
sudo grep -r "secret123" /tmp/ 2>/dev/null | grep -v "/tmp/secrets"
# Should return nothing (except the FUSE mount point)

# Check process memory (requires root)
ps aux | grep ephemfs
sudo cat /proc/<PID>/maps
# Should show memory mappings but no file-backed storage for secrets
```

### Verify Encryption

```bash
# For default encryption, check that raw memory doesn't contain plaintext
ps aux | grep ephemfs
sudo gdb -p <PID>
(gdb) dump memory /tmp/memdump.bin 0x<start> 0x<end>
(gdb) quit

strings /tmp/memdump.bin | grep "secret123"
# Should not find plaintext (encrypted in memory)
```

---

## 🐛 Troubleshooting Verification

### Issue: Mount point is empty

**Check:**
```bash
# Are secrets configured?
env | grep -E "(DATABASE_PASSWORD|API_KEY|SECRET_)"

# Is the filesystem mounted?
mount | grep ephemfs
```

**Solution:**
```bash
# Set secrets before mounting
export DATABASE_PASSWORD='test123'
./mount_ephemfs.sh
```

### Issue: Permission denied

**Check:**
```bash
# Is FUSE installed?
which fusermount

# Are you in the fuse group?
groups | grep fuse
```

**Solution:**
```bash
# Install FUSE
sudo apt-get install fuse3 libfuse3-dev

# Add user to fuse group
sudo usermod -a -G fuse $USER
# Log out and back in
```

### Issue: Transport endpoint is not connected

**Check:**
```bash
# Is there a stale mount?
mount | grep /tmp/secrets
```

**Solution:**
```bash
# Force unmount
fusermount -u /tmp/secrets
# Or
sudo umount -l /tmp/secrets

# Then mount again
./mount_ephemfs.sh
```

### Issue: RSA encryption not working

**Check:**
```bash
# Are keys generated?
ls -la private.pem public.pem

# Is SECRETFS_CIPHER_TYPE set?
echo $SECRETFS_CIPHER_TYPE

# Is public key configured?
echo $SECRETFS_PUBLIC_KEY_FILE
```

**Solution:**
```bash
# Generate keys
./target/release/secretfs-keygen generate private.pem public.pem

# Configure and mount
SECRETFS_CIPHER_TYPE=rsa \
SECRETFS_PUBLIC_KEY_FILE=public.pem \
DATABASE_PASSWORD='test123' ./mount_ephemfs.sh
```

---

## 📊 Expected Behavior Summary

| Action | Expected Result |
|--------|----------------|
| Mount with secrets | Secrets appear as files |
| `cat secret_file` | Shows decrypted content |
| `ls -la` | Shows 0600 permissions |
| Write to file | Fails with "Read-only file system" |
| Create new file | Fails with "Read-only file system" |
| Delete file | Fails with "Read-only file system" |
| Unmount | Secrets disappear, memory cleared |
| `cat` after unmount | "No such file or directory" |

---

## ✅ Quick Verification Script

Save this as `verify-secretfs.sh`:

```bash
#!/bin/bash
set -e

echo "🔍 SecretFS Quick Verification"
echo "=============================="
echo ""

# Mount
echo "1. Mounting SecretFS..."
DATABASE_PASSWORD='verify123' API_KEY='verify456' ./mount_ephemfs.sh &
MOUNT_PID=$!
sleep 3

# Verify
echo "2. Verifying secrets..."
if [ -f /tmp/secrets/database_password ]; then
    CONTENT=$(cat /tmp/secrets/database_password)
    if [ "$CONTENT" = "verify123" ]; then
        echo "   ✅ Secret content correct"
    else
        echo "   ❌ Secret content wrong: $CONTENT"
    fi
else
    echo "   ❌ Secret file not found"
fi

# Check permissions
PERMS=$(stat -c%a /tmp/secrets/database_password 2>/dev/null || stat -f%Lp /tmp/secrets/database_password 2>/dev/null)
if [ "$PERMS" = "600" ]; then
    echo "   ✅ Permissions correct (0600)"
else
    echo "   ❌ Permissions wrong: $PERMS"
fi

# Check read-only
if echo "test" > /tmp/secrets/database_password 2>/dev/null; then
    echo "   ❌ Filesystem is writable (should be read-only)"
else
    echo "   ✅ Filesystem is read-only"
fi

# Unmount
echo "3. Unmounting..."
fusermount -u /tmp/secrets
wait $MOUNT_PID 2>/dev/null || true

# Verify cleanup
if [ -f /tmp/secrets/database_password ]; then
    echo "   ❌ Secrets still present after unmount"
else
    echo "   ✅ Secrets cleared after unmount"
fi

echo ""
echo "🎉 Verification complete!"
```

Run it:
```bash
chmod +x verify-secretfs.sh
./verify-secretfs.sh
```

---

## 📚 Next Steps

After verification:

1. **For Development**: Use default encryption
   ```bash
   DATABASE_PASSWORD='dev_secret' ./mount_ephemfs.sh
   ```

2. **For Production**: Use RSA encryption
   ```bash
   SECRETFS_CIPHER_TYPE=rsa \
   SECRETFS_PUBLIC_KEY_FILE=public.pem \
   DATABASE_PASSWORD='prod_secret' ./mount_ephemfs.sh --release
   ```

3. **For Kubernetes**: See `k8s-secret-fuse-pod.yaml`

4. **For More Tests**: Run `./test-secretfs.sh`
