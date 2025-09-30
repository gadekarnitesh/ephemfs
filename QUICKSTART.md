# SecretFS Quick Start Guide

Get started with SecretFS in 5 minutes!

## 🚀 Quick Start (3 Steps)

### Step 1: Build SecretFS

```bash
cargo build --release
```

### Step 2: Mount with Your Secrets

```bash
DATABASE_PASSWORD='my_secret_123' API_KEY='sk-test-456' ./mount_ephemfs.sh
```

**You should see:**
```
🚀 Starting SecretFS
📍 Mount point: /tmp/secrets
🔐 Encryption: default
📊 Environment secrets: 2

Press Ctrl+C to unmount
```

### Step 3: Verify (Open Another Terminal)

```bash
# List secrets
ls -la /tmp/secrets/

# Read secrets
cat /tmp/secrets/database_password
# Output: my_secret_123

cat /tmp/secrets/api_key
# Output: sk-test-456
```

**To unmount:** Press `Ctrl+C` in the first terminal, or run:
```bash
./unmount_ephemfs.sh
```

---

## ✅ Automated Verification

Run the verification script:

```bash
./verify-secretfs.sh
```

**Expected output:**
```
🔍 SecretFS Quick Verification
==============================

1. Mounting SecretFS...
2. Verifying secrets exist...
   ✅ database_password file exists
   ✅ api_key file exists
   ✅ jwt_secret file exists
3. Verifying secret content...
   ✅ database_password content correct: verify123
   ✅ api_key content correct: verify456
   ✅ jwt_secret content correct: verify789
4. Verifying file permissions...
   ✅ Permissions correct (0600 - owner read-only)
5. Verifying read-only filesystem...
   ✅ Write operations blocked (read-only)
   ✅ File creation blocked (read-only)
   ✅ File deletion blocked (read-only)
6. Unmounting SecretFS...
   ✅ Unmounted successfully
7. Verifying memory cleanup...
   ✅ Secrets cleared from memory

╔════════════════════════════════════════════════════════════╗
║                  🎉 All Checks Passed! 🎉                  ║
╚════════════════════════════════════════════════════════════╝
```

---

## 🔐 Try Different Encryption Modes

### Default Encryption (Works Out of the Box)

```bash
DATABASE_PASSWORD='secret123' ./mount_ephemfs.sh
```

### Plaintext Mode (Development Only)

```bash
SECRETFS_CIPHER_TYPE=plaintext DATABASE_PASSWORD='secret123' ./mount_ephemfs.sh
```

### RSA Encryption (Production)

```bash
# 1. Generate keys
./target/release/secretfs-keygen generate private.pem public.pem

# 2. Mount with RSA
SECRETFS_CIPHER_TYPE=rsa \
SECRETFS_PUBLIC_KEY_FILE=public.pem \
DATABASE_PASSWORD='secret123' ./mount_ephemfs.sh

# 3. In another terminal, cat shows encrypted data
cat /tmp/secrets/database_password
# Output: (binary encrypted data)

# 4. Authorized app can decrypt
SECRETFS_PRIVATE_KEY_FILE=private.pem \
cargo run --example rust_decrypt_demo
```

---

## 🧪 Run Tests

```bash
# Comprehensive test suite
./test-secretfs.sh

# Encryption modes demo
./test-encryption-options.sh

# RSA encryption tests
./test-rsa-encryption.sh
```

---

## 📖 Common Use Cases

### Use Case 1: Local Development

```bash
# Set secrets
export DATABASE_PASSWORD='dev_db_pass'
export API_KEY='dev_api_key'
export JWT_SECRET='dev_jwt_secret'

# Mount
./mount_ephemfs.sh

# Your app reads secrets
cat /tmp/secrets/database_password
```

### Use Case 2: Custom Secrets

```bash
# Any SECRET_* variable becomes a file
export SECRET_STRIPE_KEY='sk_test_stripe_123'
export SECRET_SENDGRID_API='sg_api_456'

./mount_ephemfs.sh

# Files created with kebab-case names
ls /tmp/secrets/
# stripe-key
# sendgrid-api
```

### Use Case 3: External Secret Fetching

```bash
# Fetch from Vault or other APIs
export SECRETFS_URLS='https://vault.example.com/v1/secret/data/myapp'
export SECRETFS_AUTH_TOKEN='hvs.your_token_here'

./mount_ephemfs.sh

# Secrets from API are mounted as files
ls /tmp/secrets/
```

### Use Case 4: Kubernetes Sidecar

See `k8s-secret-fuse-pod.yaml` for complete example:

```yaml
apiVersion: v1
kind: Pod
metadata:
  name: app-with-secrets
spec:
  containers:
  - name: secret-fuse
    image: secretfs:latest
    env:
    - name: DATABASE_PASSWORD
      valueFrom:
        secretKeyRef:
          name: app-secrets
          key: db-password
    volumeMounts:
    - name: secrets
      mountPath: /mnt/secrets
      
  - name: main-app
    image: myapp:latest
    volumeMounts:
    - name: secrets
      mountPath: /mnt/secrets
      readOnly: true
```

---

## 🆘 Troubleshooting

### Problem: "No secrets configured" warning

**Solution:**
```bash
# Set at least one secret
export DATABASE_PASSWORD='your_secret'
./mount_ephemfs.sh
```

### Problem: "Transport endpoint is not connected"

**Solution:**
```bash
# Unmount and try again
./unmount_ephemfs.sh
./mount_ephemfs.sh
```

### Problem: Permission denied

**Solution:**
```bash
# Install FUSE
sudo apt-get install fuse3 libfuse3-dev

# Add user to fuse group
sudo usermod -a -G fuse $USER
# Log out and back in
```

### Problem: RSA encryption fails

**Solution:**
```bash
# Generate keys first
./target/release/secretfs-keygen generate private.pem public.pem

# Then configure
SECRETFS_CIPHER_TYPE=rsa \
SECRETFS_PUBLIC_KEY_FILE=public.pem \
DATABASE_PASSWORD='secret' ./mount_ephemfs.sh
```

---

## 📚 Documentation

- **VERIFICATION.md** - Detailed verification guide
- **SCRIPTS.md** - All scripts explained
- **ENCRYPTION_OPTIONS.md** - Encryption setup guide
- **RSA_ENCRYPTION.md** - RSA details (optional)
- **README.md** - Complete documentation

---

## 🎯 Next Steps

1. ✅ **Verified it works?** Try different encryption modes
2. ✅ **Ready for production?** Use RSA encryption
3. ✅ **Deploying to Kubernetes?** See k8s-secret-fuse-pod.yaml
4. ✅ **Need external secrets?** Configure SECRETFS_URLS

---

## 💡 Tips

- **Always unmount properly** to ensure memory cleanup
- **Use `--release` flag** for production deployments
- **Keep private keys secure** - never commit to git
- **Use RSA encryption** for production environments
- **Test locally first** before deploying to Kubernetes

---

## 🎉 You're Ready!

SecretFS is now set up and verified. Start using it in your applications!

```bash
# Simple usage
DATABASE_PASSWORD='secret' ./mount_ephemfs.sh

# In your app
password=$(cat /tmp/secrets/database_password)
echo "Connecting to database with password: $password"
```

Happy coding! 🚀
