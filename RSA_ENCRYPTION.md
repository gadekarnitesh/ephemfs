# SecretFS RSA Asymmetric Encryption (Optional)

SecretFS supports **optional** RSA asymmetric encryption for production environments requiring the highest level of security. When enabled, secrets can only be decrypted by authorized applications with the corresponding private key.

> **Note**: RSA encryption is completely optional. SecretFS works perfectly without it using default symmetric encryption or plaintext mode for development.

## 🔐 How It Works

### Traditional Approach (Symmetric)
- **Problem**: All applications that can read files can see plaintext secrets
- **Risk**: `cat`, `grep`, and other tools can access sensitive data
- **Security**: Relies on file permissions and process isolation only

### SecretFS RSA Approach (Asymmetric)
- **Solution**: Secrets encrypted with RSA public key in SecretFS
- **Security**: Only applications with private key can decrypt secrets
- **Benefit**: Even `cat` command only sees encrypted binary data
- **Isolation**: True application-level access control

## 🎛️ Encryption Options

SecretFS supports three encryption modes via the `SECRETFS_CIPHER_TYPE` environment variable:

| Mode | Value | Description | Use Case |
|------|-------|-------------|----------|
| **Default** | `default` or unset | XOR symmetric encryption | Development, testing |
| **Plaintext** | `plaintext` | No encryption | Local development only |
| **RSA** | `rsa` | RSA asymmetric encryption | Production, high security |

### When to Use RSA Encryption

**✅ Use RSA when:**
- Production environments with sensitive data
- Multiple applications need different access levels
- Compliance requires application-level access control
- You need protection against system-level access (cat, grep, etc.)

**⚠️ Consider alternatives when:**
- Simple development/testing scenarios
- Single application accessing secrets
- Performance is critical (RSA is slower than symmetric)
- Key management complexity is a concern

## 🚀 Quick Start

### 1. Generate RSA Key Pair

```bash
# Generate 2048-bit RSA key pair
./target/release/secretfs-keygen generate private.pem public.pem

# Generate 4096-bit RSA key pair (higher security)
./target/release/secretfs-keygen generate private.pem public.pem 4096

# View key information
./target/release/secretfs-keygen info public.pem
./target/release/secretfs-keygen info private.pem
```

### 2. Configure SecretFS with Public Key

```bash
# Option 1: Using key file
export SECRETFS_CIPHER_TYPE=rsa
export SECRETFS_PUBLIC_KEY_FILE=public.pem

# Option 2: Using environment variable
export SECRETFS_CIPHER_TYPE=rsa
export SECRETFS_PUBLIC_KEY_PEM="$(cat public.pem)"

# Start SecretFS with your secrets
DATABASE_PASSWORD="secret123" \
API_KEY="sk-test-456" \
./target/release/ephemfs /mnt/secrets
```

### 3. Configure Applications with Private Key

```bash
# Option 1: Using key file
export SECRETFS_PRIVATE_KEY_FILE=private.pem

# Option 2: Using environment variable
export SECRETFS_PRIVATE_KEY_PEM="$(cat private.pem)"

# Run your application
cargo run --example rust_decrypt_demo
```

## 🛡️ Security Benefits

### ✅ Application-Level Access Control
- Only applications with the private key can decrypt secrets
- File system access alone is insufficient to read secrets
- True zero-trust security model

### ✅ Protection Against System Tools
```bash
# These commands only see encrypted binary data:
cat /mnt/secrets/database_password     # Shows encrypted bytes
grep "password" /mnt/secrets/*         # Finds nothing
hexdump /mnt/secrets/api_key          # Shows encrypted hex
```

### ✅ Memory-Only Storage
- Secrets encrypted in RAM, never written to disk
- Automatic memory zeroing on cleanup
- No persistence - secrets disappear when container stops

### ✅ Read-Only Filesystem
- Write operations blocked with "Read-only file system" error
- Prevents tampering and injection attacks
- Immutable secret storage

## 📚 API Usage

### Rust Applications

```rust
use ephemfs::secret_client::{SecretClient, convenience};

// Simple convenience API (automatic configuration)
let db_password = convenience::get_secret("database_password")?;
let all_secrets = convenience::get_all_secrets()?;

// Advanced API with explicit configuration
let client = SecretClient::new_with_rsa_decryption("/mnt/secrets")?;
let api_key = client.get_secret("api_key")?;
let secrets_list = client.list_secrets()?;

// Wait for secrets (useful for initialization)
let jwt_secret = convenience::wait_for_secret("jwt_secret", 30)?;
```

### Environment Variables

#### SecretFS Configuration
- `SECRETFS_CIPHER_TYPE=rsa` - Enable RSA encryption
- `SECRETFS_PUBLIC_KEY_FILE` - Path to public key file
- `SECRETFS_PUBLIC_KEY_PEM` - Public key in PEM format

#### Application Configuration
- `SECRETFS_PRIVATE_KEY_FILE` - Path to private key file
- `SECRETFS_PRIVATE_KEY_PEM` - Private key in PEM format
- `SECRETFS_MOUNT_PATH` - SecretFS mount path (default: `/mnt/secrets`)

## 🏗️ Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Environment   │    │    SecretFS      │    │   Application   │
│   Variables     │───▶│  (Public Key)    │◀───│ (Private Key)   │
│                 │    │                  │    │                 │
│ DATABASE_PWD=   │    │ ┌──────────────┐ │    │ ┌─────────────┐ │
│ "secret123"     │    │ │ Encrypted    │ │    │ │ Decrypted   │ │
│                 │    │ │ Binary Data  │ │    │ │ "secret123" │ │
│ API_KEY=        │    │ │              │ │    │ │             │ │
│ "sk-test-456"   │    │ │ RSA-2048     │ │    │ │ Plain Text  │ │
└─────────────────┘    │ │ Encrypted    │ │    │ │ Secrets     │ │
                       │ └──────────────┘ │    │ └─────────────┘ │
                       │                  │    │                 │
                       │ Memory-Only      │    │ Client Library  │
                       │ FUSE Filesystem  │    │ Auto-Decryption │
                       └──────────────────┘    └─────────────────┘
```

## 🔧 Key Management

### Development Environment
```bash
# Generate development keys
secretfs-keygen generate dev-private.pem dev-public.pem

# Use in development
export SECRETFS_PUBLIC_KEY_FILE=dev-public.pem
export SECRETFS_PRIVATE_KEY_FILE=dev-private.pem
```

### Production Environment
```bash
# Generate production keys with higher security
secretfs-keygen generate prod-private.pem prod-public.pem 4096

# Secure the private key
chmod 600 prod-private.pem
mv prod-private.pem /etc/secretfs/keys/

# Distribute public key (safe to share)
cp prod-public.pem /etc/secretfs/public.pem
```

### Kubernetes Deployment
```yaml
apiVersion: v1
kind: Secret
metadata:
  name: secretfs-keys
data:
  private.pem: <base64-encoded-private-key>
  public.pem: <base64-encoded-public-key>

---
apiVersion: v1
kind: Pod
metadata:
  name: secretfs-pod
spec:
  containers:
  - name: secretfs
    image: secretfs:latest
    env:
    - name: SECRETFS_CIPHER_TYPE
      value: "rsa"
    - name: SECRETFS_PUBLIC_KEY_PEM
      valueFrom:
        secretKeyRef:
          name: secretfs-keys
          key: public.pem
    volumeMounts:
    - name: secrets
      mountPath: /mnt/secrets
  
  - name: app
    image: myapp:latest
    env:
    - name: SECRETFS_PRIVATE_KEY_PEM
      valueFrom:
        secretKeyRef:
          name: secretfs-keys
          key: private.pem
    volumeMounts:
    - name: secrets
      mountPath: /mnt/secrets
  
  volumes:
  - name: secrets
    emptyDir: {}
```

## 🧪 Testing

### Run RSA Encryption Test
```bash
./test-rsa-encryption.sh
```

### Manual Testing
```bash
# Start SecretFS with RSA encryption
SECRETFS_CIPHER_TYPE=rsa \
SECRETFS_PUBLIC_KEY_FILE=public.pem \
DATABASE_PASSWORD="test123" \
./target/release/ephemfs /tmp/test_mount &

# Try to read with system tools (should show encrypted data)
cat /tmp/test_mount/database_password
hexdump -C /tmp/test_mount/database_password

# Decrypt with authorized application
SECRETFS_PRIVATE_KEY_FILE=private.pem \
cargo run --example rust_decrypt_demo
```

## 🔒 Security Best Practices

### Key Management
- **Generate unique key pairs** for each environment
- **Rotate keys regularly** according to security policy
- **Store private keys securely** (HSM, key management systems)
- **Never commit keys** to version control
- **Use strong key sizes** (2048-bit minimum, 4096-bit recommended)

### Access Control
- **Limit private key access** to authorized applications only
- **Use different keys** for different applications/services
- **Monitor key usage** and access patterns
- **Implement key revocation** procedures

### Deployment
- **Use secure channels** for key distribution
- **Validate key integrity** before use
- **Implement proper logging** for security events
- **Regular security audits** of key management practices

## 🚨 Important Notes

### Limitations
- **RSA key size limits**: Maximum plaintext size is key_size - 11 bytes
- **Performance**: RSA encryption is slower than symmetric encryption
- **Key distribution**: Requires secure key management infrastructure

### Fallback Behavior
- If RSA cipher initialization fails, SecretFS falls back to default cipher
- Applications without private keys fall back to plaintext mode
- Graceful degradation ensures system availability

### Compatibility
- Works with all existing SecretFS features (external fetching, etc.)
- Compatible with Kubernetes sidecar pattern
- Supports both file-based and environment-based key configuration

## 📖 Examples

See the `examples/` directory for complete working examples:
- `rust_decrypt_demo.rs` - Rust application with RSA decryption
- `secret_reader_app.rs` - Full-featured secret client application

## 🆘 Troubleshooting

### Common Issues

**"RSA key error: No public key configuration found"**
- Set `SECRETFS_PUBLIC_KEY_FILE` or `SECRETFS_PUBLIC_KEY_PEM`

**"Failed to decrypt secret: RSA decryption failed"**
- Verify private key matches the public key used for encryption
- Check private key file permissions and readability

**"Input/output error" when reading files**
- This is expected behavior - use SecretFS client library instead
- System tools cannot decrypt RSA-encrypted secrets

**"Read-only file system" when writing**
- This is a security feature - SecretFS is intentionally read-only
- Secrets can only be updated by restarting SecretFS with new values
