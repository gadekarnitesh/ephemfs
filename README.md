# SecretFS - Encrypted Memory-Only FUSE Filesystem for Kubernetes Secrets

SecretFS is a specialized FUSE (Filesystem in Userspace) implementation written in Rust that mounts secrets as **encrypted files in memory-only storage**. It's designed to work as a sidecar container in Kubernetes pods, providing a secure way to access secrets as regular files with **pluggable encryption** and **zero disk I/O**.

## 🔑 Key Features

- **🧠 Memory-Only Storage**: Secrets exist only in RAM, never written to disk
- **🔐 Flexible Encryption**: Default XOR, plaintext, or RSA asymmetric encryption
- **🛡️ Security-First**: Read-only filesystem, automatic memory cleanup, secure permissions
- **🚀 Kubernetes Ready**: Perfect for sidecar container pattern
- **⚡ High Performance**: Direct memory access, no I/O overhead
- **🔧 Configurable**: Environment variable-based configuration
- **📁 File Interface**: Standard filesystem operations (ls, cat, find, etc.)
- **🏥 Health Checks**: Built-in health checking for Kubernetes deployments
- **🔒 Zero Hardcoded Secrets**: No hardcoded values, environment-driven only

## Prerequisites

### Linux
```bash
sudo apt-get install fuse3 libfuse3-dev
# or on older systems:
sudo apt-get install fuse libfuse-dev
```

### macOS
```bash
brew install macfuse
```

## Building

```bash
cargo build --release
```

## Usage

### Standalone Usage

```bash
# Basic usage
./target/release/ephemfs <mount_point>

# Example: Mount secrets at /tmp/secrets
./target/release/ephemfs /tmp/secrets_mount

# With environment variables and encryption
DATABASE_PASSWORD="my_db_pass" \
API_KEY="my_api_key" \
SECRETFS_CIPHER_TYPE="default" \
SECRETFS_ENCRYPTION_KEY="my-secure-key-2024" \
./target/release/ephemfs /tmp/secrets_mount

# With plaintext (no encryption)
DATABASE_PASSWORD="my_db_pass" \
SECRETFS_CIPHER_TYPE="plaintext" \
./target/release/ephemfs /tmp/secrets_mount
```

### Kubernetes Sidecar Usage

Deploy using the provided Kubernetes manifests:

```bash
# Simple pod with sidecar
kubectl apply -f k8s-secret-fuse-pod.yaml

# Production deployment
kubectl apply -f k8s-deployment.yaml
```

### Docker Compose Usage

```bash
# Build and run with docker-compose
docker-compose up --build

# View logs
docker-compose logs -f secret-fuse
docker-compose logs -f app
```

## Environment Variables

SecretFS supports the following environment variables:

### Standard Secrets
- `DATABASE_PASSWORD` - Database password
- `API_KEY` - API key for external services
- `JWT_SECRET` - JWT signing secret
- `REDIS_PASSWORD` - Redis password
- `VAULT_TOKEN` - Vault authentication token
- `CONFIG_JSON` - JSON configuration content

### Custom Secrets
- `SECRET_*` - Any environment variable starting with `SECRET_` will be mounted as a file
  - Example: `SECRET_STRIPE_KEY` becomes `/mnt/secrets/stripe-key`
  - Example: `SECRET_SENDGRID_API_KEY` becomes `/mnt/secrets/sendgrid-api-key`

### Encryption Configuration

SecretFS supports three encryption modes:

- `SECRETFS_CIPHER_TYPE` - Encryption method:
  - `"default"` (or unset) - XOR cipher for development/testing
  - `"plaintext"` - No encryption (local development only)
  - `"rsa"` - RSA asymmetric encryption (production)

**Default/XOR Encryption:**
- `SECRETFS_ENCRYPTION_KEY` - Encryption key for default cipher

**RSA Encryption (Production):**
- `SECRETFS_PUBLIC_KEY_FILE` - Path to RSA public key file
- `SECRETFS_PUBLIC_KEY_PEM` - RSA public key in PEM format
- Generate keys with: `./target/release/secretfs-keygen generate private.pem public.pem`

> 📖 **See [ENCRYPTION_OPTIONS.md](ENCRYPTION_OPTIONS.md) for detailed encryption setup guide**

### External Secret Fetching

SecretFS can fetch secrets from external HTTP/HTTPS APIs in addition to environment variables:

- `SECRETFS_URLS` - Comma-separated list of URLs to fetch secrets from
  - Example: `"https://api.example.com/secrets,https://vault.example.com/v1/secret"`
- `SECRETFS_AUTH_TOKEN` - Bearer token for API authentication (optional)
- `SECRETFS_FETCHER_TYPE` - Type of fetcher to use:
  - `"http"` - HTTP/HTTPS fetcher (default)
  - `"mock"` - Mock fetcher for testing and development
- `SECRETFS_TIMEOUT_SECONDS` - HTTP request timeout in seconds (default: 30)
- `SECRETFS_RETRY_ATTEMPTS` - Number of retry attempts (default: 3)
- `SECRETFS_HEADERS` - Custom HTTP headers (format: `"Key1:Value1,Key2:Value2"`)

#### Supported JSON Response Formats

1. **Flat key-value object**:
```json
{
  "database_password": "secret123",
  "api_key": "sk-test-456",
  "jwt_secret": "jwt-signing-key"
}
```

2. **Array of secret objects**:
```json
[
  {
    "key": "database_password",
    "value": "secret123",
    "environment": "production"
  },
  {
    "key": "api_key",
    "value": "sk-test-456",
    "environment": "production"
  }
]
```

### General Configuration

- `FUSE_MOUNTPOINT` - Override the mount point (default: from command line)

### Example Session

1. **Test with environment variables**:
   ```bash
   # Set secrets via environment variables
   export DATABASE_PASSWORD="my_secure_db_password"
   export API_KEY="sk-1234567890abcdef"
   export SECRET_STRIPE_KEY="sk_live_stripe_key_example"

   # Mount the filesystem
   ./target/debug/ephemfs /tmp/secrets_mount
   ```

2. **Browse the mounted secrets** (in another terminal):
   ```bash
   # List available secrets
   ls -la /tmp/secrets_mount/

   # Read secrets
   cat /tmp/secrets_mount/database_password
   cat /tmp/secrets_mount/api_key
   cat /tmp/secrets_mount/stripe-key

   # Use in your application
   DB_PASS=$(cat /tmp/secrets_mount/database_password)
   API_KEY=$(cat /tmp/secrets_mount/api_key)
   ```

3. **Unmount**:
   ```bash
   # Press Ctrl+C in the terminal running the filesystem
   # or use fusermount
   fusermount -u /tmp/secrets_mount
   ```

4. **Test with external secret fetching**:
   ```bash
   # Set up environment variables and external URLs
   export DATABASE_PASSWORD="env_db_password"
   export API_KEY="env_api_key"
   export SECRETFS_URLS="https://api.example.com/secrets,https://vault.example.com/v1/secret"
   export SECRETFS_AUTH_TOKEN="bearer-token-123"
   export SECRETFS_TIMEOUT_SECONDS="60"
   export SECRETFS_HEADERS="X-Environment:production,X-Service:myapp"

   # Mount with external fetching enabled
   ./target/debug/ephemfs /tmp/secrets_mount
   ```

5. **Test with mock fetcher** (for development):
   ```bash
   # Use mock fetcher for testing
   export SECRETFS_FETCHER_TYPE="mock"
   export SECRETFS_URLS="mock://test"
   export DATABASE_PASSWORD="test_db_password"

   # Mount with mock data
   ./target/debug/ephemfs /tmp/secrets_mount

   # Check available secrets (environment + mock)
   ls -la /tmp/secrets_mount/
   cat /tmp/secrets_mount/mock_api_key
   cat /tmp/secrets_mount/mock_database_url
   ```

## Implementation Details

### Architecture

- **EphemFS struct**: Main filesystem implementation
- **FileInfo struct**: Metadata for each file/directory
- **Filesystem trait**: FUSE operations implementation

### Key FUSE Operations Implemented

- `lookup`: Find files by name in directories
- `getattr`: Get file attributes (size, permissions, timestamps)
- `read`: Read file contents
- `readdir`: List directory contents

### File Discovery

The filesystem recursively scans the source directory at startup and builds an in-memory index of all files and directories. Hidden files (starting with '.') are automatically excluded.

## 🔐 Custom Encryption Implementation

SecretFS supports pluggable encryption through the `SecretCipher` trait. You can implement your own encryption methods:

### Implementing Custom Cipher

```rust
use secretfs::encryption::{SecretCipher, EncryptionError};

pub struct MyCustomCipher {
    key: Vec<u8>,
}

impl SecretCipher for MyCustomCipher {
    fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        // Your encryption logic here
        // Example: AES-256-GCM, ChaCha20-Poly1305, etc.
        todo!("Implement your encryption")
    }

    fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        // Your decryption logic here
        todo!("Implement your decryption")
    }

    fn cipher_info(&self) -> String {
        "MyCustomCipher (AES-256-GCM)".to_string()
    }
}
```

### Production Encryption Examples

For production use, consider implementing:

- **AES-256-GCM**: Using the `aes-gcm` crate
- **ChaCha20-Poly1305**: Using the `chacha20poly1305` crate
- **Age encryption**: Using the `age` crate
- **Hardware Security Modules**: Integration with HSM APIs

See `examples/custom_cipher.rs` for complete implementation examples.

### Environment-Based Cipher Selection

```bash
# Use your custom cipher
SECRETFS_CIPHER_TYPE="aes256" \
SECRETFS_AES_KEY="your-256-bit-hex-key" \
./target/release/ephemfs /mnt/secrets
```

## 🛡️ Security Features

### Memory-Only Storage
- **Zero Disk I/O**: Secrets never written to disk
- **Automatic Cleanup**: Memory zeroed on container exit
- **No Swap Exposure**: Container limits prevent swapping
- **FUSE Virtual FS**: No backing store, pure memory

### Encryption at Rest (in Memory)
- **Pluggable Ciphers**: Implement your own encryption
- **Key Management**: Environment-based key configuration
- **Secure Defaults**: XOR cipher for demo, plaintext for development

### Access Control
- **Read-Only Filesystem**: No write operations allowed
- **Secure Permissions**: 0600 file permissions (owner only)
- **Process Isolation**: Container-based isolation
- **No Network Access**: Local-only operation

## Limitations

- **Read-only**: Files cannot be modified through the FUSE mount
- **Memory usage**: All secrets stored encrypted in memory
- **Container lifecycle**: Secrets disappear when container stops
- **Demo encryption**: Default XOR cipher is not cryptographically secure

## Troubleshooting

### Permission Denied
```bash
# Add your user to the fuse group
sudo usermod -a -G fuse $USER
# Then log out and back in
```

### Mount Point Busy
```bash
# Unmount any existing mounts
fusermount -u /tmp/ephemfs_mount
```

### FUSE Not Available
```bash
# Check if FUSE is loaded
lsmod | grep fuse
# Load FUSE module if needed
sudo modprobe fuse
```

## Development

### Running Tests
```bash
cargo test
```

### Debug Mode
The application prints debug information to stdout when running, showing FUSE operations as they occur.

## License

This project is open source. Feel free to modify and distribute.
