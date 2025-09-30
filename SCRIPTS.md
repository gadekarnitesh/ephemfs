# SecretFS Scripts Guide

This document describes all the scripts available for SecretFS.

## 🚀 Main Scripts

### mount_ephemfs.sh
**Purpose**: Mount SecretFS with secrets from environment variables

**Usage**:
```bash
./mount_ephemfs.sh [mount_point] [options]
```

**Options**:
- `--help, -h` - Show help message
- `--release` - Use optimized release build
- `--build` - Force rebuild before mounting

**Examples**:
```bash
# Basic usage with default encryption
DATABASE_PASSWORD='secret123' API_KEY='sk-test' ./mount_ephemfs.sh

# Custom mount point
DATABASE_PASSWORD='secret123' ./mount_ephemfs.sh /mnt/secrets

# Production with RSA encryption
SECRETFS_CIPHER_TYPE=rsa \
SECRETFS_PUBLIC_KEY_FILE=public.pem \
DATABASE_PASSWORD='secret123' ./mount_ephemfs.sh /mnt/secrets --release

# With external secret fetching
SECRETFS_URLS='https://vault.example.com/v1/secret' \
SECRETFS_AUTH_TOKEN='hvs.token123' ./mount_ephemfs.sh
```

**Features**:
- ✅ Automatic build if needed
- ✅ Mount point creation
- ✅ Existing mount detection and cleanup
- ✅ Secret count validation
- ✅ Helpful error messages

---

### unmount_ephemfs.sh
**Purpose**: Safely unmount SecretFS and clear secrets from memory

**Usage**:
```bash
./unmount_ephemfs.sh [mount_point]
```

**Examples**:
```bash
# Unmount from default location
./unmount_ephemfs.sh

# Unmount from custom location
./unmount_ephemfs.sh /mnt/secrets
```

**Features**:
- ✅ Automatic memory cleanup
- ✅ Mount point validation
- ✅ Clear status messages

---

## 🧪 Test Scripts

### test-secretfs.sh
**Purpose**: Comprehensive test suite for all SecretFS features

**Usage**:
```bash
./test-secretfs.sh
```

**Tests Included**:
1. ✅ Default Encryption (XOR Cipher)
2. ✅ Plaintext Mode (No Encryption)
3. ✅ RSA Asymmetric Encryption
4. ✅ Custom Secrets (SECRET_* pattern)
5. ✅ Read-only Filesystem Security
6. ✅ File Permissions (0600)

**Output**:
- Detailed test results with pass/fail status
- Success rate percentage
- Automatic cleanup after tests

**Example Output**:
```
╔════════════════════════════════════════════════════════════╗
║         SecretFS Comprehensive Test Suite                 ║
╚════════════════════════════════════════════════════════════╝

✅ Tests Passed: 10
❌ Tests Failed: 0
Success Rate: 100%
🎉 All tests passed!
```

---

### test-encryption-options.sh
**Purpose**: Interactive demo of all encryption modes

**Usage**:
```bash
./test-encryption-options.sh
```

**Demonstrates**:
1. Default Encryption (XOR) - Works out of the box
2. Plaintext Mode - No encryption for development
3. RSA Encryption - Production-grade security

**Features**:
- ✅ Side-by-side comparison of encryption modes
- ✅ Shows encrypted vs decrypted data
- ✅ Demonstrates application-level access control
- ✅ Automatic key generation if needed
- ✅ Automatic cleanup

---

### test-rsa-encryption.sh
**Purpose**: Detailed RSA encryption testing and demonstration

**Usage**:
```bash
./test-rsa-encryption.sh
```

**Tests**:
1. RSA key pair generation
2. SecretFS mounting with RSA encryption
3. Encrypted data verification (cat shows binary)
4. Application decryption with private key
5. Security verification

**Features**:
- ✅ Complete RSA workflow demonstration
- ✅ Python decryption example
- ✅ Rust application example
- ✅ Security analysis

---

## 🔧 Build Scripts

### build.sh
**Purpose**: Build SecretFS binaries

**Usage**:
```bash
./build.sh [--release]
```

**Options**:
- `--release` - Build optimized release version

**Builds**:
- `ephemfs` - Main SecretFS binary
- `secretfs-keygen` - RSA key generation utility

---

## 📋 Quick Reference

### Common Workflows

#### Development Setup
```bash
# 1. Build
./build.sh

# 2. Mount with default encryption
DATABASE_PASSWORD='dev_secret' ./mount_ephemfs.sh

# 3. Use secrets
cat /tmp/secrets/database_password

# 4. Unmount
./unmount_ephemfs.sh
```

#### Production Setup with RSA
```bash
# 1. Build release version
./build.sh --release

# 2. Generate RSA keys
./target/release/secretfs-keygen generate private.pem public.pem

# 3. Mount with RSA encryption
SECRETFS_CIPHER_TYPE=rsa \
SECRETFS_PUBLIC_KEY_FILE=public.pem \
DATABASE_PASSWORD='prod_secret' \
./mount_ephemfs.sh /mnt/secrets --release

# 4. Application reads secrets (with private key)
SECRETFS_PRIVATE_KEY_FILE=private.pem \
cargo run --example rust_decrypt_demo

# 5. Unmount
./unmount_ephemfs.sh /mnt/secrets
```

#### Testing
```bash
# Run comprehensive test suite
./test-secretfs.sh

# Demo encryption options
./test-encryption-options.sh

# Test RSA encryption
./test-rsa-encryption.sh
```

---

## 🔐 Environment Variables

All scripts respect these environment variables:

### Secrets
- `DATABASE_PASSWORD` - Database password
- `API_KEY` - API key
- `JWT_SECRET` - JWT signing secret
- `SECRET_*` - Custom secrets (e.g., `SECRET_STRIPE_KEY`)

### Encryption
- `SECRETFS_CIPHER_TYPE` - `default`, `plaintext`, or `rsa`
- `SECRETFS_ENCRYPTION_KEY` - Key for default cipher
- `SECRETFS_PUBLIC_KEY_FILE` - RSA public key file
- `SECRETFS_PRIVATE_KEY_FILE` - RSA private key file (for apps)

### External Fetching
- `SECRETFS_URLS` - Comma-separated API URLs
- `SECRETFS_AUTH_TOKEN` - Bearer token for authentication

---

## 🆘 Troubleshooting

### Mount fails with "Transport endpoint is not connected"
```bash
# Unmount and try again
./unmount_ephemfs.sh
./mount_ephemfs.sh
```

### "No secrets configured" warning
```bash
# Set at least one secret
export DATABASE_PASSWORD='your_secret'
./mount_ephemfs.sh
```

### RSA encryption fails
```bash
# Generate keys first
./target/release/secretfs-keygen generate private.pem public.pem

# Then mount with RSA
SECRETFS_CIPHER_TYPE=rsa \
SECRETFS_PUBLIC_KEY_FILE=public.pem \
DATABASE_PASSWORD='secret' ./mount_ephemfs.sh
```

### Permission denied errors
```bash
# Ensure FUSE is installed
sudo apt-get install fuse3 libfuse3-dev

# Check user permissions
groups | grep fuse
```

---

## 📚 Related Documentation

- **README.md** - General SecretFS documentation
- **ENCRYPTION_OPTIONS.md** - Detailed encryption guide
- **RSA_ENCRYPTION.md** - RSA setup and usage
- **EXTERNAL_SECRETS.md** - External API integration
- **k8s-secret-fuse-pod.yaml** - Kubernetes deployment example

---

## 🎯 Script Selection Guide

| Task | Script | Notes |
|------|--------|-------|
| Mount secrets | `mount_ephemfs.sh` | Main script for daily use |
| Unmount secrets | `unmount_ephemfs.sh` | Clean shutdown |
| Test all features | `test-secretfs.sh` | Automated testing |
| Learn encryption | `test-encryption-options.sh` | Interactive demo |
| Test RSA | `test-rsa-encryption.sh` | RSA-specific tests |
| Build project | `build.sh` | Development builds |

---

## 💡 Tips

1. **Always unmount properly** to ensure memory cleanup
2. **Use `--release` flag** for production deployments
3. **Test locally first** before deploying to Kubernetes
4. **Keep private keys secure** - never commit to git
5. **Use RSA encryption** for production environments
6. **Run test suite** after making changes
