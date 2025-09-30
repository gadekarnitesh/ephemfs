# Kubernetes Deployment Quick Start

## 📦 Simple BusyBox + SecretFS Sidecar Demo

This deployment demonstrates SecretFS as a Kubernetes sidecar container with a simple BusyBox pod.

### Architecture

```
┌─────────────────────────────────────────────────────┐
│                    Pod                              │
│                                                     │
│  ┌──────────────────┐      ┌──────────────────┐   │
│  │    BusyBox       │      │    SecretFS      │   │
│  │  (Main App)      │◄────►│   (Sidecar)      │   │
│  │                  │      │                  │   │
│  │  Reads secrets   │      │  Mounts secrets  │   │
│  │  from /secrets   │      │  as FUSE FS      │   │
│  └──────────────────┘      └──────────────────┘   │
│           │                         │              │
│           └─────────┬───────────────┘              │
│                     │                              │
│              ┌──────▼──────┐                       │
│              │  /secrets   │                       │
│              │  (emptyDir) │                       │
│              └─────────────┘                       │
└─────────────────────────────────────────────────────┘
                      │
                      │ Reads from
                      ▼
            ┌──────────────────┐
            │  Kubernetes      │
            │  Secret          │
            │  (app-secrets)   │
            └──────────────────┘
```

### Features

✅ **Simple Setup** - BusyBox pod with SecretFS sidecar  
✅ **Memory-Only** - Secrets stored only in RAM  
✅ **Shared Volume** - Both containers access `/secrets` via emptyDir  
✅ **Auto-Mount** - SecretFS automatically mounts secrets from K8s Secret  
✅ **Read-Only** - Main container has read-only access to secrets  
✅ **Minimal Resources** - Only 16Mi RAM per container  

---

## 🚀 Quick Deploy

### 1. Build SecretFS Docker Image

```bash
# Build the Docker image
docker build -t secretfs:latest .

# Tag for your registry (replace with your registry)
docker tag secretfs:latest ghcr.io/yourorg/secretfs:latest

# Push to registry
docker push ghcr.io/yourorg/secretfs:latest
```

### 2. Update Image in Deployment

Edit `k8s-deployment.yaml` and replace:
```yaml
image: ghcr.io/yourorg/secretfs:latest
```

With your actual image location.

### 3. Deploy to Kubernetes

```bash
# Apply the deployment
kubectl apply -f k8s-deployment.yaml

# Check pod status
kubectl get pods -l app=busybox-demo

# View logs from BusyBox (main container)
kubectl logs -l app=busybox-demo -c busybox

# View logs from SecretFS (sidecar)
kubectl logs -l app=busybox-demo -c secretfs
```

---

## 🔍 Verify Deployment

### Check BusyBox Logs

```bash
kubectl logs -l app=busybox-demo -c busybox
```

**Expected output:**
```
🚀 BusyBox started
📂 Waiting for secrets to be mounted...

📋 Available secrets:
-rw------- 1 root root 25 Jan 01 00:00 api_key
-rw------- 1 root root 25 Jan 01 00:00 database_password
-rw------- 1 root root 27 Jan 01 00:00 jwt_secret

🔍 Reading secrets:
  api_key: sk-test-api-key-abcdef123456
  database_password: my_secure_db_password_123
  jwt_secret: jwt-secret-key-2024-secure

✅ Secrets loaded successfully!
💤 Sleeping forever... (use kubectl exec to interact)
```

### Check SecretFS Logs

```bash
kubectl logs -l app=busybox-demo -c secretfs
```

**Expected output:**
```
🔒 SecretFS mounted at: /secrets
✅ Loaded 3 secret(s) | Encryption: DefaultCipher (XOR with 25-byte key) - ⚠️ DEMO ONLY, NOT SECURE! | Memory-only storage
Press Ctrl+C to unmount
```

### Interactive Testing

```bash
# Exec into BusyBox container
kubectl exec -it deployment/busybox-with-secretfs -c busybox -- sh

# Inside the container:
ls -lh /secrets/
cat /secrets/database_password
cat /secrets/api_key
cat /secrets/jwt_secret

# Try to write (should fail - read-only)
echo "test" > /secrets/test.txt  # Permission denied

# Exit
exit
```

---

## 🔧 Configuration

### Add More Secrets

Edit the `app-secrets` Secret in `k8s-deployment.yaml`:

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: app-secrets
type: Opaque
stringData:
  database-password: "my_secure_db_password_123"
  api-key: "sk-test-api-key-abcdef123456"
  jwt-secret: "jwt-secret-key-2024-secure"
  redis-password: "redis_password_456"        # Add this
  stripe-key: "sk_live_stripe_key_789"        # Add this
```

Then add environment variables to the SecretFS container:

```yaml
- name: REDIS_PASSWORD
  valueFrom:
    secretKeyRef:
      name: app-secrets
      key: redis-password
- name: SECRET_STRIPE_KEY  # Will become /secrets/stripe-key
  valueFrom:
    secretKeyRef:
      name: app-secrets
      key: stripe-key
```

### Change Encryption Mode

Edit the SecretFS container environment:

```yaml
# Default encryption (XOR - demo only)
- name: SECRETFS_CIPHER_TYPE
  value: "default"

# Plaintext (no encryption - local dev only)
- name: SECRETFS_CIPHER_TYPE
  value: "plaintext"

# RSA encryption (production)
- name: SECRETFS_CIPHER_TYPE
  value: "rsa"
- name: SECRETFS_PUBLIC_KEY_FILE
  value: "/keys/public.pem"
```

### Adjust Resources

```yaml
resources:
  requests:
    memory: "16Mi"   # Increase if you have many/large secrets
    cpu: "10m"
  limits:
    memory: "32Mi"
    cpu: "50m"
```

---

## 🧹 Cleanup

```bash
# Delete the deployment
kubectl delete -f k8s-deployment.yaml

# Or delete by name
kubectl delete deployment busybox-with-secretfs
kubectl delete secret app-secrets
```

---

## 📝 Notes

### Security Context

SecretFS requires `privileged: true` to mount FUSE filesystems. In production:

- Use Pod Security Policies or Pod Security Standards
- Restrict which pods can use privileged containers
- Consider using a dedicated namespace with RBAC

### Volume Mount Propagation

The `mountPropagation: Bidirectional` setting is required for FUSE mounts to be visible to other containers in the pod.

### Liveness Probe

The liveness probe checks if secrets are accessible:

```yaml
livenessProbe:
  exec:
    command:
    - sh
    - -c
    - test -f /secrets/database_password
  initialDelaySeconds: 5
  periodSeconds: 10
```

If SecretFS crashes, Kubernetes will restart the container.

---

## 🎯 Use Cases

### 1. **Development/Testing**
- Quick secret injection without complex setup
- Test secret rotation scenarios
- Debug secret access issues

### 2. **Legacy Applications**
- Apps that read secrets from files
- No code changes required
- Drop-in replacement for mounted secrets

### 3. **Multi-Secret Applications**
- Consolidate multiple K8s Secrets into one mount point
- Easier secret management
- Consistent secret access pattern

### 4. **Secret Rotation**
- Update K8s Secret
- Restart pod to reload secrets
- Or implement hot-reload in SecretFS

---

## 🔗 Next Steps

- **Production Setup**: See [RSA_ENCRYPTION.md](RSA_ENCRYPTION.md) for RSA encryption
- **External Secrets**: See [EXTERNAL_SECRETS.md](EXTERNAL_SECRETS.md) for Vault/AWS integration
- **Monitoring**: Add Prometheus metrics for secret access
- **Helm Chart**: Package as Helm chart for easier deployment

---

## 🐛 Troubleshooting

### Pod Stuck in Pending

```bash
kubectl describe pod -l app=busybox-demo
```

Check for:
- Image pull errors
- Resource constraints
- Node selector issues

### SecretFS Not Mounting

```bash
kubectl logs -l app=busybox-demo -c secretfs
```

Check for:
- Missing environment variables
- FUSE not available on node
- Privileged security context denied

### BusyBox Can't Read Secrets

```bash
kubectl exec -it deployment/busybox-with-secretfs -c busybox -- ls -la /secrets/
```

Check for:
- Mount propagation settings
- Volume mount configuration
- SecretFS startup timing

### Permission Denied

Ensure:
- SecretFS has `privileged: true`
- Volume has `mountPropagation: Bidirectional`
- Node supports FUSE

---

**Ready to deploy!** 🚀

```bash
kubectl apply -f k8s-deployment.yaml
kubectl logs -f -l app=busybox-demo -c busybox
```

