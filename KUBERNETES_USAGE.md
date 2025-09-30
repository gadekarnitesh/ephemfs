# SecretFS Kubernetes Usage Guide

This guide shows how to use SecretFS as a sidecar container in Kubernetes to provide secrets as files to your main application.

## Quick Start

1. **Deploy the demo pod**:
   ```bash
   kubectl apply -f k8s-secret-fuse-pod.yaml
   ```

2. **Check the pod status**:
   ```bash
   kubectl get pods secret-fuse-demo
   kubectl logs secret-fuse-demo -c secret-fuse
   kubectl logs secret-fuse-demo -c app
   ```

3. **Exec into the main app container to see secrets**:
   ```bash
   kubectl exec -it secret-fuse-demo -c app -- sh
   ls -la /mnt/secrets/
   cat /mnt/secrets/database_password
   ```

## How It Works

### Sidecar Pattern

```
┌─────────────────────────────────────────────────────────┐
│                        Pod                              │
│                                                         │
│  ┌─────────────────┐    ┌─────────────────────────────┐ │
│  │   SecretFS      │    │     Main Application       │ │
│  │   (Sidecar)     │    │                             │ │
│  │                 │    │  Reads secrets from:       │ │
│  │ Env Vars ────┐  │    │  /mnt/secrets/              │ │
│  │ - DB_PASS    │  │    │  - database_password        │ │
│  │ - API_KEY    │  │    │  - api_key                  │ │
│  │ - JWT_SECRET │  │    │  - config.json              │ │
│  │              │  │    │                             │ │
│  │ Mounts to ───┼──┼────┼─► /mnt/secrets/             │ │
│  │ FUSE FS      │  │    │   (shared volume)           │ │
│  └─────────────────┘    └─────────────────────────────┘ │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

### Secret Flow

1. **Kubernetes Secrets/ConfigMaps** → Environment Variables
2. **Environment Variables** → SecretFS sidecar container
3. **SecretFS** → Mounts secrets as files in shared volume
4. **Main Application** → Reads secrets as regular files

## Configuration

### Environment Variables

The SecretFS sidecar supports these environment variables:

#### Standard Secrets
- `DATABASE_PASSWORD` → `/mnt/secrets/database_password`
- `API_KEY` → `/mnt/secrets/api_key`
- `JWT_SECRET` → `/mnt/secrets/jwt_secret`
- `REDIS_PASSWORD` → `/mnt/secrets/redis_password`
- `VAULT_TOKEN` → `/mnt/secrets/vault_token`
- `CONFIG_JSON` → `/mnt/secrets/config.json`

#### Custom Secrets (SECRET_* pattern)
- `SECRET_STRIPE_KEY` → `/mnt/secrets/stripe-key`
- `SECRET_SENDGRID_API_KEY` → `/mnt/secrets/sendgrid-api-key`
- `SECRET_CUSTOM_TOKEN` → `/mnt/secrets/custom-token`

### Kubernetes Secret Integration

```yaml
# Create Kubernetes secrets
apiVersion: v1
kind: Secret
metadata:
  name: app-secrets
stringData:
  database-password: "prod_db_pass_123"
  api-key: "sk-prod-api-key-456"
  jwt-secret: "prod-jwt-secret-789"

---
# Use in pod
spec:
  containers:
  - name: secret-fuse
    env:
    - name: DATABASE_PASSWORD
      valueFrom:
        secretKeyRef:
          name: app-secrets
          key: database-password
    - name: API_KEY
      valueFrom:
        secretKeyRef:
          name: app-secrets
          key: api-key
```

## Security Features

### File Permissions
- All secret files are created with `0600` permissions (read-only for owner)
- Only the container user can read the secrets
- Secrets are not visible to other processes

### Container Security
- SecretFS runs as non-root user (UID 1000)
- Minimal capabilities required (`SYS_ADMIN` for FUSE)
- No privileged mode required
- Secrets exist only in memory (FUSE filesystem)

### Kubernetes Security
- Secrets are loaded from Kubernetes Secrets (encrypted at rest)
- No secrets in container images
- No secrets in environment variables visible to main app
- Secrets are isolated per pod

## Production Deployment

Use the production deployment manifest:

```bash
kubectl apply -f k8s-deployment.yaml
```

This includes:
- Proper resource limits
- Health checks
- Multiple replicas
- Service configuration
- ConfigMap for application config

## Troubleshooting

### Pod Not Starting
```bash
# Check events
kubectl describe pod <pod-name>

# Check sidecar logs
kubectl logs <pod-name> -c secret-fuse

# Check main app logs
kubectl logs <pod-name> -c app
```

### Secrets Not Available
```bash
# Exec into sidecar container
kubectl exec -it <pod-name> -c secret-fuse -- sh

# Check mount point
ls -la /mnt/secrets/

# Check environment variables
env | grep -E "(DATABASE|API|JWT|SECRET_)"
```

### FUSE Issues
```bash
# Check if FUSE is available on nodes
kubectl get nodes -o wide
kubectl describe node <node-name>

# Check node FUSE support
kubectl debug node/<node-name> -it --image=alpine -- sh
# In debug pod:
ls -la /dev/fuse
lsmod | grep fuse
```

## Benefits

1. **Security**: Secrets never stored in container images
2. **Simplicity**: Main app reads secrets as regular files
3. **Flexibility**: Support for any secret via environment variables
4. **Kubernetes Native**: Integrates with Kubernetes Secrets and ConfigMaps
5. **Zero Dependencies**: Main application needs no special libraries
6. **Language Agnostic**: Works with any programming language
7. **Standard Tools**: Use `cat`, `head`, `tail`, etc. to read secrets

## Example Application Code

### Python
```python
# Read database password
with open('/mnt/secrets/database_password', 'r') as f:
    db_password = f.read().strip()

# Read API key
with open('/mnt/secrets/api_key', 'r') as f:
    api_key = f.read().strip()

# Read JSON config
import json
with open('/mnt/secrets/config.json', 'r') as f:
    config = json.load(f)
```

### Go
```go
package main

import (
    "io/ioutil"
    "encoding/json"
)

// Read database password
dbPassword, err := ioutil.ReadFile("/mnt/secrets/database_password")
if err != nil {
    log.Fatal(err)
}

// Read JSON config
configData, err := ioutil.ReadFile("/mnt/secrets/config.json")
if err != nil {
    log.Fatal(err)
}

var config map[string]interface{}
json.Unmarshal(configData, &config)
```

### Node.js
```javascript
const fs = require('fs');

// Read database password
const dbPassword = fs.readFileSync('/mnt/secrets/database_password', 'utf8').trim();

// Read API key
const apiKey = fs.readFileSync('/mnt/secrets/api_key', 'utf8').trim();

// Read JSON config
const config = JSON.parse(fs.readFileSync('/mnt/secrets/config.json', 'utf8'));
```
