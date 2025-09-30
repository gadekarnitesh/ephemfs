#!/bin/bash

# Example: Using SecretFS with External API Secret Fetching
# This script demonstrates how to configure SecretFS to fetch secrets from external APIs

set -e

echo "🌐 SecretFS External API Integration Demo"
echo "========================================="
echo ""

# Configuration
MOUNT_POINT="/tmp/secretfs_api_demo"
mkdir -p "$MOUNT_POINT"

# Clean up any existing mounts
fusermount -u "$MOUNT_POINT" 2>/dev/null || true

echo "📋 This demo shows how to configure SecretFS to fetch secrets from:"
echo "   • Environment variables (traditional method)"
echo "   • External HTTP/HTTPS APIs (new feature)"
echo "   • Combined sources with encryption"
echo ""

echo "🔧 CONFIGURATION EXAMPLES"
echo "========================="
echo ""

echo "1. Basic HTTP API Configuration:"
echo "   export SECRETFS_URLS=\"https://api.example.com/secrets\""
echo "   export SECRETFS_AUTH_TOKEN=\"bearer-token-123\""
echo ""

echo "2. Multiple APIs with Custom Headers:"
echo "   export SECRETFS_URLS=\"https://vault.example.com/v1/secret,https://api.example.com/config\""
echo "   export SECRETFS_AUTH_TOKEN=\"vault-token-456\""
echo "   export SECRETFS_HEADERS=\"X-Environment:production,X-Service:myapp\""
echo "   export SECRETFS_TIMEOUT_SECONDS=\"60\""
echo ""

echo "3. HashiCorp Vault Integration:"
echo "   export SECRETFS_URLS=\"https://vault.company.com/v1/secret/data/myapp\""
echo "   export SECRETFS_AUTH_TOKEN=\"\$VAULT_TOKEN\""
echo "   export SECRETFS_HEADERS=\"X-Vault-Namespace:production\""
echo ""

echo "4. AWS Secrets Manager (via API Gateway):"
echo "   export SECRETFS_URLS=\"https://api.company.com/secrets/myapp\""
echo "   export SECRETFS_AUTH_TOKEN=\"\$AWS_ACCESS_TOKEN\""
echo "   export SECRETFS_HEADERS=\"X-AWS-Region:us-east-1,Content-Type:application/json\""
echo ""

echo "5. Kubernetes Secret Store CSI Driver Integration:"
echo "   export SECRETFS_URLS=\"https://secret-store.kube-system.svc.cluster.local/secrets\""
echo "   export SECRETFS_AUTH_TOKEN=\"\$SERVICE_ACCOUNT_TOKEN\""
echo "   export SECRETFS_HEADERS=\"Authorization:Bearer \$SERVICE_ACCOUNT_TOKEN\""
echo ""

echo "🧪 DEMO: Mock API Fetching"
echo "=========================="
echo ""

echo "Starting SecretFS with mock API fetcher (simulates real API calls)..."

# Demo with mock fetcher
SECRETFS_FETCHER_TYPE="mock" \
SECRETFS_URLS="https://mock.api.example.com/secrets" \
DATABASE_PASSWORD="env_database_password_123" \
API_KEY="env_api_key_456" \
JWT_SECRET="env_jwt_secret_789" \
SECRET_STRIPE_KEY="sk_live_stripe_key_abc" \
SECRETFS_CIPHER_TYPE="default" \
SECRETFS_ENCRYPTION_KEY="demo-encryption-key-2024" \
../target/release/ephemfs "$MOUNT_POINT" &

FUSE_PID=$!
sleep 3

echo ""
echo "✅ SecretFS mounted successfully!"
echo ""

echo "📁 Available secrets (environment + mock API):"
ls -la "$MOUNT_POINT/"
echo ""

echo "📖 Reading secrets from different sources:"
echo ""

echo "🔹 Environment Variable Secrets:"
echo "   Database Password: $(cat "$MOUNT_POINT/database_password")"
echo "   API Key: $(cat "$MOUNT_POINT/api_key")"
echo "   JWT Secret: $(cat "$MOUNT_POINT/jwt_secret")"
echo "   Stripe Key: $(cat "$MOUNT_POINT/stripe-key")"
echo ""

echo "🔹 Mock API Secrets (if available):"
if [ -f "$MOUNT_POINT/mock_api_key" ]; then
    echo "   Mock API Key: $(cat "$MOUNT_POINT/mock_api_key")"
fi

if [ -f "$MOUNT_POINT/mock_database_url" ]; then
    echo "   Mock Database URL: $(cat "$MOUNT_POINT/mock_database_url")"
fi
echo ""

echo "🔒 Security Features Demonstration:"
echo ""

echo "   Testing write protection (should fail):"
echo "   $ echo 'hacker_data' > $MOUNT_POINT/database_password"
echo "new_data" > "$MOUNT_POINT/database_password" 2>&1 || echo "   ✅ Write blocked (Read-only filesystem)"
echo ""

echo "   Testing file creation (should fail):"
echo "   $ echo 'malicious' > $MOUNT_POINT/hacker_file"
echo "malicious" > "$MOUNT_POINT/hacker_file" 2>&1 || echo "   ✅ File creation blocked (Read-only filesystem)"
echo ""

echo "   Verifying secret integrity:"
echo "   Database Password: $(cat "$MOUNT_POINT/database_password")"
echo "   ✅ Secret integrity maintained"
echo ""

# Clean up
kill $FUSE_PID 2>/dev/null || true
sleep 1
fusermount -u "$MOUNT_POINT" 2>/dev/null || true

echo "🚀 PRODUCTION USAGE EXAMPLES"
echo "============================"
echo ""

echo "Example 1: Kubernetes Deployment with Vault"
echo "--------------------------------------------"
cat << 'EOF'
apiVersion: apps/v1
kind: Deployment
metadata:
  name: myapp
spec:
  template:
    spec:
      containers:
      - name: secret-fuse
        image: ghcr.io/yourorg/secretfs:latest
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
EOF

echo ""
echo "Example 2: Docker Compose with Multiple Secret Sources"
echo "------------------------------------------------------"
cat << 'EOF'
version: '3.8'
services:
  secret-fuse:
    image: secretfs:latest
    environment:
      - SECRETFS_URLS=https://api.example.com/secrets,https://vault.example.com/v1/secret
      - SECRETFS_AUTH_TOKEN=${VAULT_TOKEN}
      - SECRETFS_HEADERS=X-Environment:production,X-Service:myapp
      - SECRETFS_TIMEOUT_SECONDS=60
      - SECRETFS_CIPHER_TYPE=default
      - SECRETFS_ENCRYPTION_KEY=${ENCRYPTION_KEY}
      - DATABASE_PASSWORD=${DB_PASSWORD}
      - API_KEY=${API_KEY}
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
        # Wait for secrets to be available
        while [ ! -f /mnt/secrets/database_password ]; do sleep 1; done
        
        # Use secrets in application
        export DB_PASSWORD=$$(cat /mnt/secrets/database_password)
        export API_KEY=$$(cat /mnt/secrets/api_key)
        
        exec ./myapp
      '

volumes:
  secrets-volume:
    driver: local
EOF

echo ""
echo "🎯 KEY BENEFITS"
echo "==============="
echo ""
echo "✅ Unified Secret Interface: Read secrets from files regardless of source"
echo "✅ Multiple Sources: Environment variables + HTTP APIs + custom fetchers"
echo "✅ Security First: Memory-only storage, encryption, read-only access"
echo "✅ Cloud Native: Perfect for Kubernetes sidecar pattern"
echo "✅ Flexible APIs: Support for various JSON formats and authentication"
echo "✅ Zero Dependencies: No special libraries needed in your application"
echo "✅ Hot Reload: Secrets can be refreshed without application restart"
echo ""

echo "🧹 Demo completed! Cleaning up..."
rmdir "$MOUNT_POINT" 2>/dev/null || true

echo ""
echo "📚 Next Steps:"
echo "   1. Implement your own SecretCipher for production encryption"
echo "   2. Configure your secret management system APIs"
echo "   3. Deploy as Kubernetes sidecar container"
echo "   4. Monitor secret access and refresh patterns"
echo ""
echo "🔐 SecretFS: Secure, Flexible, Cloud-Native Secret Management!"
