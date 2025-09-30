# External Secret Fetching in SecretFS

SecretFS now supports fetching secrets from external HTTP/HTTPS APIs in addition to environment variables, providing a unified interface for secret management across multiple sources.

## 🌐 Overview

The external secret fetching feature allows SecretFS to:

- **Fetch secrets from HTTP/HTTPS APIs** (HashiCorp Vault, AWS Secrets Manager, etc.)
- **Combine multiple secret sources** (environment variables + external APIs)
- **Support various authentication methods** (Bearer tokens, custom headers)
- **Handle different JSON response formats** (flat objects, arrays of secrets)
- **Provide pluggable fetcher implementations** (HTTP, mock, custom)

## 🔧 Configuration

### Environment Variables

| Variable | Description | Default | Example |
|----------|-------------|---------|---------|
| `SECRETFS_URLS` | Comma-separated URLs to fetch secrets from | - | `https://vault.example.com/v1/secret,https://api.example.com/config` |
| `SECRETFS_AUTH_TOKEN` | Bearer token for API authentication | - | `hvs.CAESIJ1234567890abcdef` |
| `SECRETFS_FETCHER_TYPE` | Type of fetcher to use | `http` | `http`, `mock` |
| `SECRETFS_TIMEOUT_SECONDS` | HTTP request timeout | `30` | `60` |
| `SECRETFS_RETRY_ATTEMPTS` | Number of retry attempts | `3` | `5` |
| `SECRETFS_HEADERS` | Custom HTTP headers | - | `X-Vault-Namespace:prod,Content-Type:application/json` |

### Supported JSON Response Formats

#### 1. Flat Key-Value Object
```json
{
  "database_password": "secret123",
  "api_key": "sk-test-456",
  "jwt_secret": "jwt-signing-key"
}
```

#### 2. Array of Secret Objects
```json
[
  {
    "key": "database_password",
    "value": "secret123",
    "environment": "production",
    "type": "database"
  },
  {
    "key": "api_key",
    "value": "sk-test-456",
    "environment": "production",
    "type": "api_key"
  }
]
```

## 🏗️ Architecture

### Secret Fetcher Trait

```rust
pub trait SecretFetcher: Send + Sync {
    fn fetch_secrets(&self, config: &SecretFetchConfig) -> Result<Vec<FetchedSecret>, SecretFetchError>;
    fn fetcher_info(&self) -> String;
}
```

### Built-in Implementations

1. **HttpSecretFetcher**: Fetches secrets from HTTP/HTTPS APIs
2. **MockSecretFetcher**: Provides mock data for testing and development

### Custom Fetcher Implementation

```rust
use secretfs::secret_fetcher::{SecretFetcher, SecretFetchConfig, FetchedSecret, SecretFetchError};

pub struct CustomFetcher {
    // Your custom implementation
}

impl SecretFetcher for CustomFetcher {
    fn fetch_secrets(&self, config: &SecretFetchConfig) -> Result<Vec<FetchedSecret>, SecretFetchError> {
        // Implement your custom fetching logic
        // Connect to your secret management system
        // Parse and return secrets
    }
    
    fn fetcher_info(&self) -> String {
        "CustomFetcher (My Secret Management System)".to_string()
    }
}
```

## 🚀 Usage Examples

### Basic HTTP API

```bash
export SECRETFS_URLS="https://api.example.com/secrets"
export SECRETFS_AUTH_TOKEN="bearer-token-123"
./target/release/ephemfs /mnt/secrets
```

### HashiCorp Vault Integration

```bash
export SECRETFS_URLS="https://vault.company.com/v1/secret/data/myapp"
export SECRETFS_AUTH_TOKEN="$VAULT_TOKEN"
export SECRETFS_HEADERS="X-Vault-Namespace:production"
export SECRETFS_TIMEOUT_SECONDS="60"
./target/release/ephemfs /mnt/secrets
```

### Multiple APIs with Custom Headers

```bash
export SECRETFS_URLS="https://vault.example.com/v1/secret,https://api.example.com/config"
export SECRETFS_AUTH_TOKEN="vault-token-456"
export SECRETFS_HEADERS="X-Environment:production,X-Service:myapp,Content-Type:application/json"
./target/release/ephemfs /mnt/secrets
```

### Mock Fetcher for Testing

```bash
export SECRETFS_FETCHER_TYPE="mock"
export SECRETFS_URLS="mock://test"
./target/release/ephemfs /mnt/secrets
```

## 🐳 Container Integration

### Kubernetes Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: myapp
spec:
  template:
    spec:
      containers:
      - name: secret-fuse
        image: secretfs:latest
        env:
        - name: SECRETFS_URLS
          value: "https://vault.company.com/v1/secret/data/myapp"
        - name: SECRETFS_AUTH_TOKEN
          valueFrom:
            secretKeyRef:
              name: vault-token
              key: token
        - name: SECRETFS_CIPHER_TYPE
          value: "default"
        - name: SECRETFS_ENCRYPTION_KEY
          valueFrom:
            secretKeyRef:
              name: encryption-key
              key: key
        volumeMounts:
        - name: secrets-fuse-mnt
          mountPath: /mnt/secrets
          mountPropagation: Bidirectional
        securityContext:
          capabilities:
            add: ["SYS_ADMIN"]
      - name: app
        image: myapp:latest
        volumeMounts:
        - name: secrets-fuse-mnt
          mountPath: /mnt/secrets
          readOnly: true
        command: ["sh", "-c"]
        args:
        - |
          # Read secrets from FUSE mount
          export DB_PASSWORD=$(cat /mnt/secrets/database_password)
          export API_KEY=$(cat /mnt/secrets/api_key)
          exec ./myapp
      volumes:
      - name: secrets-fuse-mnt
        emptyDir: {}
```

### Docker Compose

```yaml
version: '3.8'
services:
  secret-fuse:
    image: secretfs:latest
    environment:
      - SECRETFS_URLS=https://api.example.com/secrets
      - SECRETFS_AUTH_TOKEN=${VAULT_TOKEN}
      - SECRETFS_HEADERS=X-Environment:production
      - SECRETFS_CIPHER_TYPE=default
      - SECRETFS_ENCRYPTION_KEY=${ENCRYPTION_KEY}
    volumes:
      - secrets-volume:/mnt/secrets:shared
    cap_add:
      - SYS_ADMIN
    devices:
      - /dev/fuse:/dev/fuse
    
  app:
    image: myapp:latest
    volumes:
      - secrets-volume:/mnt/secrets:ro
    depends_on:
      - secret-fuse
    command: |
      sh -c '
        while [ ! -f /mnt/secrets/database_password ]; do sleep 1; done
        export DB_PASSWORD=$$(cat /mnt/secrets/database_password)
        exec ./myapp
      '

volumes:
  secrets-volume:
```

## 🔒 Security Features

- **Memory-only storage**: Externally fetched secrets are encrypted and stored only in RAM
- **Automatic cleanup**: Secrets are zeroed when the container stops
- **Read-only filesystem**: All write operations are blocked
- **Encryption support**: All secrets (environment + external) are encrypted using the configured cipher
- **Secure authentication**: Bearer tokens and custom headers for API authentication
- **Timeout protection**: Configurable timeouts prevent hanging requests

## 🧪 Testing

### Run External Secret Tests

```bash
# Test all external secret fetching features
./test-external-secrets.sh

# Run comprehensive demo
./examples/external-api-demo.sh
```

### Mock Fetcher for Development

The mock fetcher provides sample secrets for testing:

```bash
export SECRETFS_FETCHER_TYPE="mock"
export SECRETFS_URLS="mock://test"
./target/release/ephemfs /mnt/secrets

# Check available secrets
ls -la /mnt/secrets/
cat /mnt/secrets/mock_api_key
cat /mnt/secrets/mock_database_url
```

## 🎯 Benefits

1. **Unified Interface**: Read secrets from files regardless of source (env vars or APIs)
2. **Multiple Sources**: Combine environment variables with external APIs
3. **Cloud Native**: Perfect for Kubernetes sidecar pattern
4. **Security First**: Memory-only storage with encryption
5. **Flexible APIs**: Support for various JSON formats and authentication methods
6. **Zero Dependencies**: No special libraries needed in your application
7. **Hot Reload**: Secrets can be refreshed without application restart
8. **Pluggable Architecture**: Easy to add custom fetcher implementations

## 🔮 Future Enhancements

- **Automatic refresh**: Periodic secret refresh from external sources
- **Caching strategies**: Configurable caching for performance optimization
- **Webhook support**: Real-time secret updates via webhooks
- **Additional authentication**: Support for mTLS, OAuth2, etc.
- **Metrics and monitoring**: Prometheus metrics for secret access patterns
- **Audit logging**: Detailed logging of secret access and refresh events

## 📚 Related Documentation

- [README.md](README.md) - Main project documentation
- [SECURITY.md](SECURITY.md) - Security architecture details
- [examples/custom_cipher.rs](examples/custom_cipher.rs) - Custom encryption examples
- [KUBERNETES_USAGE.md](KUBERNETES_USAGE.md) - Kubernetes deployment guide
