# SecretFS Encryption Options

SecretFS provides flexible encryption options to meet different security requirements and use cases.

## 🎛️ Available Encryption Modes

### 1. Default Encryption (Recommended for Development)

**Configuration:**
```bash
# Option 1: Use default (no configuration needed)
./target/release/ephemfs /mnt/secrets

# Option 2: Explicit configuration
export SECRETFS_CIPHER_TYPE=default
export SECRETFS_ENCRYPTION_KEY="my-secret-key-2024"
./target/release/ephemfs /mnt/secrets
```

**Features:**
- ✅ XOR-based symmetric encryption
- ✅ Simple setup - works out of the box
- ✅ Good for development and testing
- ✅ Configurable encryption key
- ⚠️ Demo-level security (not for production)

### 2. Plaintext Mode (Development Only)

**Configuration:**
```bash
export SECRETFS_CIPHER_TYPE=plaintext
./target/release/ephemfs /mnt/secrets
```

**Features:**
- ✅ No encryption overhead
- ✅ Easy debugging and development
- ✅ Direct file access shows plaintext
- ❌ No security - secrets visible to all processes
- ⚠️ Never use in production

### 3. RSA Asymmetric Encryption (Production)

**Configuration:**
```bash
# Step 1: Generate RSA key pair
./target/release/secretfs-keygen generate private.pem public.pem

# Step 2: Configure SecretFS with public key
export SECRETFS_CIPHER_TYPE=rsa
export SECRETFS_PUBLIC_KEY_FILE=public.pem
./target/release/ephemfs /mnt/secrets

# Step 3: Configure applications with private key
export SECRETFS_PRIVATE_KEY_FILE=private.pem
```

**Features:**
- ✅ Production-grade security
- ✅ Application-level access control
- ✅ Only authorized apps can decrypt secrets
- ✅ System tools (cat, grep) cannot read secrets
- ✅ Perfect for multi-tenant environments
- ⚠️ Requires key management
- ⚠️ Slower than symmetric encryption

## 🚀 Quick Setup Examples

### Development Setup (Default Encryption)
```bash
# Simple development setup
DATABASE_PASSWORD="dev_secret_123" \
API_KEY="dev_api_key_456" \
./target/release/ephemfs /mnt/secrets

# Your app can read secrets normally
cat /mnt/secrets/database_password  # Shows decrypted value
```

### Production Setup (RSA Encryption)
```bash
# 1. Generate production keys
./target/release/secretfs-keygen generate /etc/secretfs/private.pem /etc/secretfs/public.pem
chmod 600 /etc/secretfs/private.pem

# 2. Start SecretFS (sidecar container)
SECRETFS_CIPHER_TYPE=rsa \
SECRETFS_PUBLIC_KEY_FILE=/etc/secretfs/public.pem \
DATABASE_PASSWORD="prod_secret_123" \
API_KEY="prod_api_key_456" \
./target/release/ephemfs /mnt/secrets

# 3. System tools cannot read secrets
cat /mnt/secrets/database_password  # Shows encrypted binary data

# 4. Authorized applications can decrypt
SECRETFS_PRIVATE_KEY_FILE=/etc/secretfs/private.pem \
cargo run --example rust_decrypt_demo
```

## 🔧 Environment Variables Reference

### General Configuration
| Variable | Description | Default |
|----------|-------------|---------|
| `SECRETFS_CIPHER_TYPE` | Encryption mode: `default`, `plaintext`, `rsa` | `default` |

### Default Encryption
| Variable | Description | Default |
|----------|-------------|---------|
| `SECRETFS_ENCRYPTION_KEY` | Symmetric encryption key | `default-secretfs-key-2024` |

### RSA Encryption
| Variable | Description | Required |
|----------|-------------|----------|
| `SECRETFS_PUBLIC_KEY_FILE` | Path to RSA public key file | Yes (for RSA) |
| `SECRETFS_PUBLIC_KEY_PEM` | RSA public key in PEM format | Alternative to file |
| `SECRETFS_PRIVATE_KEY_FILE` | Path to RSA private key file | Yes (for apps) |
| `SECRETFS_PRIVATE_KEY_PEM` | RSA private key in PEM format | Alternative to file |

## 🏗️ Architecture Comparison

### Default/Plaintext Mode
```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Environment   │    │    SecretFS      │    │   Application   │
│   Variables     │───▶│                  │◀───│                 │
│                 │    │ ┌──────────────┐ │    │ ┌─────────────┐ │
│ DATABASE_PWD=   │    │ │ Plaintext or │ │    │ │ Direct File │ │
│ "secret123"     │    │ │ XOR Encrypted│ │    │ │ Access      │ │
│                 │    │ │              │ │    │ │             │ │
│ API_KEY=        │    │ │ Simple       │ │    │ │ cat, grep   │ │
│ "sk-test-456"   │    │ │ Decryption   │ │    │ │ work        │ │
└─────────────────┘    │ └──────────────┘ │    │ └─────────────┘ │
                       └──────────────────┘    └─────────────────┘
```

### RSA Mode
```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Environment   │    │    SecretFS      │    │   Application   │
│   Variables     │───▶│  (Public Key)    │◀───│ (Private Key)   │
│                 │    │                  │    │                 │
│ DATABASE_PWD=   │    │ ┌──────────────┐ │    │ ┌─────────────┐ │
│ "secret123"     │    │ │ RSA Encrypted│ │    │ │ Client Lib  │ │
│                 │    │ │ Binary Data  │ │    │ │ Decryption  │ │
│ API_KEY=        │    │ │              │ │    │ │             │ │
│ "sk-test-456"   │    │ │ cat/grep     │ │    │ │ Authorized  │ │
└─────────────────┘    │ │ cannot read  │ │    │ │ Access Only │ │
                       │ └──────────────┘ │    │ └─────────────┘ │
                       └──────────────────┘    └─────────────────┘
```

## 🎯 Choosing the Right Mode

### Use Default Encryption When:
- ✅ Developing and testing applications
- ✅ Single application accessing secrets
- ✅ Simple deployment requirements
- ✅ Learning SecretFS features

### Use Plaintext Mode When:
- ✅ Local development only
- ✅ Debugging secret access issues
- ✅ Performance testing without encryption overhead
- ❌ Never in production or shared environments

### Use RSA Encryption When:
- ✅ Production environments
- ✅ Multiple applications with different access levels
- ✅ Compliance requirements for access control
- ✅ Protection against system-level access
- ✅ Multi-tenant or shared environments

## 🔄 Migration Between Modes

You can easily switch between encryption modes by changing the `SECRETFS_CIPHER_TYPE` environment variable:

```bash
# Start with default encryption
SECRETFS_CIPHER_TYPE=default ./target/release/ephemfs /mnt/secrets

# Switch to plaintext for debugging
SECRETFS_CIPHER_TYPE=plaintext ./target/release/ephemfs /mnt/secrets

# Upgrade to RSA for production
SECRETFS_CIPHER_TYPE=rsa \
SECRETFS_PUBLIC_KEY_FILE=public.pem \
./target/release/ephemfs /mnt/secrets
```

## 🆘 Troubleshooting

### "No encryption key configured" (Default mode)
```bash
export SECRETFS_ENCRYPTION_KEY="your-secret-key"
```

### "RSA key error: No public key configuration found" (RSA mode)
```bash
# Generate keys first
./target/release/secretfs-keygen generate private.pem public.pem

# Then configure
export SECRETFS_PUBLIC_KEY_FILE=public.pem
```

### "Failed to decrypt secret" (RSA mode)
```bash
# For applications, configure private key
export SECRETFS_PRIVATE_KEY_FILE=private.pem
```

## 📚 Next Steps

- **Default/Plaintext**: Start using SecretFS immediately - no additional setup needed
- **RSA**: Read the complete [RSA_ENCRYPTION.md](RSA_ENCRYPTION.md) guide for detailed setup
- **Examples**: Check the `examples/` directory for working code samples
- **Testing**: Run `./test-rsa-encryption.sh` to see RSA encryption in action
