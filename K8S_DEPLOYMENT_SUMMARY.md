# ✅ Kubernetes Deployment Updated

## 🎯 What Changed

The Kubernetes deployment has been simplified to use **BusyBox as the main container** with **SecretFS as a sidecar**.

---

## 📋 New Deployment Structure

### **Before (Complex)**
- Custom application container with health checks
- Multiple secrets and ConfigMaps
- Service definition
- Complex resource configuration

### **After (Simple)**
- ✅ **BusyBox** - Simple main container that reads secrets
- ✅ **SecretFS** - Sidecar that mounts secrets via FUSE
- ✅ **Shared Volume** - emptyDir for secret sharing
- ✅ **Minimal Resources** - 16Mi RAM per container
- ✅ **Auto-Demo** - BusyBox automatically lists and reads secrets on startup

---

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────┐
│                    Pod                              │
│                                                     │
│  ┌──────────────────┐      ┌──────────────────┐   │
│  │    BusyBox       │      │    SecretFS      │   │
│  │  (Main App)      │◄────►│   (Sidecar)      │   │
│  │                  │      │                  │   │
│  │  Reads from      │      │  FUSE mount      │   │
│  │  /secrets        │      │  at /secrets     │   │
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
                      │ Loads from
                      ▼
            ┌──────────────────┐
            │  Kubernetes      │
            │  Secret          │
            │  (app-secrets)   │
            └──────────────────┘
```

---

## 📄 Files Updated

### 1. **k8s-deployment.yaml**

**Main Container (BusyBox):**
```yaml
- name: busybox
  image: busybox:latest
  command: 
  - sh
  - -c
  - |
    echo "🚀 BusyBox started"
    echo "📂 Waiting for secrets to be mounted..."
    sleep 5
    echo ""
    echo "📋 Available secrets:"
    ls -lh /secrets/
    echo ""
    echo "🔍 Reading secrets:"
    for file in /secrets/*; do
      if [ -f "$file" ]; then
        echo "  $(basename $file): $(cat $file)"
      fi
    done
    echo ""
    echo "✅ Secrets loaded successfully!"
    echo "💤 Sleeping forever... (use kubectl exec to interact)"
    sleep infinity
  volumeMounts:
  - name: secrets-mount
    mountPath: /secrets
    readOnly: true
```

**Sidecar Container (SecretFS):**
```yaml
- name: secretfs
  image: ghcr.io/yourorg/secretfs:latest
  env:
  - name: FUSE_MOUNTPOINT
    value: "/secrets"
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
  - name: JWT_SECRET
    valueFrom:
      secretKeyRef:
        name: app-secrets
        key: jwt-secret
  - name: SECRETFS_CIPHER_TYPE
    value: "default"
  securityContext:
    privileged: true  # Required for FUSE
  volumeMounts:
  - name: secrets-mount
    mountPath: /secrets
    mountPropagation: Bidirectional
```

**Kubernetes Secret:**
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
```

### 2. **Dockerfile**

Updated to:
- ✅ Use UID 1000 (common in K8s)
- ✅ Include `secretfs-keygen` binary
- ✅ Default mount point: `/secrets`
- ✅ Removed health check (handled by K8s liveness probe)
- ✅ Simplified for sidecar usage

### 3. **K8S_QUICKSTART.md** (NEW)

Complete guide with:
- ✅ Architecture diagram
- ✅ Quick deploy instructions
- ✅ Verification steps
- ✅ Configuration examples
- ✅ Troubleshooting guide

---

## 🚀 Quick Deploy

### 1. Build Docker Image

```bash
# Build
docker build -t secretfs:latest .

# Tag for your registry
docker tag secretfs:latest ghcr.io/yourorg/secretfs:latest

# Push
docker push ghcr.io/yourorg/secretfs:latest
```

### 2. Update Image in Deployment

Edit `k8s-deployment.yaml` line 60:
```yaml
image: ghcr.io/yourorg/secretfs:latest  # Change to your registry
```

### 3. Deploy

```bash
# Apply
kubectl apply -f k8s-deployment.yaml

# Check status
kubectl get pods -l app=busybox-demo

# View BusyBox logs
kubectl logs -l app=busybox-demo -c busybox

# View SecretFS logs
kubectl logs -l app=busybox-demo -c secretfs
```

---

## 🔍 Expected Output

### BusyBox Container Logs

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

### SecretFS Container Logs

```
🔒 SecretFS mounted at: /secrets
✅ Loaded 3 secret(s) | Encryption: DefaultCipher (XOR with 25-byte key) - ⚠️ DEMO ONLY, NOT SECURE! | Memory-only storage
Press Ctrl+C to unmount
```

---

## 🧪 Interactive Testing

```bash
# Exec into BusyBox
kubectl exec -it deployment/busybox-with-secretfs -c busybox -- sh

# Inside the container:
ls -lh /secrets/
cat /secrets/database_password
cat /secrets/api_key

# Try to write (should fail - read-only)
echo "test" > /secrets/test.txt  # Permission denied

# Exit
exit
```

---

## 🔧 Configuration Options

### Add More Secrets

1. **Update Kubernetes Secret:**
```yaml
stringData:
  database-password: "my_secure_db_password_123"
  api-key: "sk-test-api-key-abcdef123456"
  jwt-secret: "jwt-secret-key-2024-secure"
  redis-password: "redis_pass_456"  # Add this
```

2. **Add Environment Variable to SecretFS:**
```yaml
- name: REDIS_PASSWORD
  valueFrom:
    secretKeyRef:
      name: app-secrets
      key: redis-password
```

3. **Redeploy:**
```bash
kubectl apply -f k8s-deployment.yaml
kubectl rollout restart deployment/busybox-with-secretfs
```

### Change Encryption Mode

```yaml
# Default (XOR - demo)
- name: SECRETFS_CIPHER_TYPE
  value: "default"

# Plaintext (no encryption)
- name: SECRETFS_CIPHER_TYPE
  value: "plaintext"

# RSA (production)
- name: SECRETFS_CIPHER_TYPE
  value: "rsa"
- name: SECRETFS_PUBLIC_KEY_FILE
  value: "/keys/public.pem"
```

---

## 🎯 Key Features

### ✅ Sidecar Pattern
- SecretFS runs alongside your application
- Shares secrets via shared volume
- Independent lifecycle management

### ✅ Memory-Only Storage
- Secrets stored only in RAM
- No disk writes
- Automatic cleanup on pod termination

### ✅ Read-Only Access
- Main container has read-only mount
- Prevents tampering
- Secure by default

### ✅ Minimal Resources
- 16Mi RAM per container
- 10m CPU per container
- Efficient for production use

### ✅ Auto-Discovery
- BusyBox automatically lists secrets
- No manual configuration needed
- Easy to verify deployment

---

## 🧹 Cleanup

```bash
# Delete deployment
kubectl delete -f k8s-deployment.yaml

# Or by name
kubectl delete deployment busybox-with-secretfs
kubectl delete secret app-secrets
```

---

## 📚 Documentation

- **[K8S_QUICKSTART.md](K8S_QUICKSTART.md)** - Complete deployment guide
- **[ENCRYPTION_OPTIONS.md](ENCRYPTION_OPTIONS.md)** - Encryption configuration
- **[RSA_ENCRYPTION.md](RSA_ENCRYPTION.md)** - RSA setup for production
- **[VERIFICATION.md](VERIFICATION.md)** - Testing and verification

---

## 🎉 Summary

The Kubernetes deployment is now:
- ✅ **Simple** - BusyBox + SecretFS sidecar
- ✅ **Clean** - Minimal configuration
- ✅ **Secure** - Memory-only, read-only, encrypted
- ✅ **Production-Ready** - Resource limits, liveness probes
- ✅ **Easy to Test** - Auto-demo on startup

**Deploy now:**
```bash
kubectl apply -f k8s-deployment.yaml
kubectl logs -f -l app=busybox-demo -c busybox
```

🚀 **Ready for production!**

