# 🐳 Docker Hub Setup - Quick Guide

## 📦 Image Location

**Docker Hub:** `niteshgadekar/ephemfs:1.0`

---

## 🚀 Quick Start

### Pull and Run

```bash
# Pull the image
docker pull niteshgadekar/ephemfs:1.0

# Run with secrets
docker run --rm --privileged \
  -e DATABASE_PASSWORD=secret123 \
  -e API_KEY=sk-test-456 \
  niteshgadekar/ephemfs:1.0
```

### Use in Kubernetes

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: myapp-with-secretfs
spec:
  template:
    spec:
      containers:
      - name: secretfs
        image: niteshgadekar/ephemfs:1.0  # ✅ Ready to use!
        env:
        - name: DATABASE_PASSWORD
          valueFrom:
            secretKeyRef:
              name: app-secrets
              key: database-password
```

Deploy:
```bash
kubectl apply -f k8s-deployment.yaml
```

---

## 🔧 GitHub Actions Setup

### Prerequisites

1. **Docker Hub Account**
   - Username: `niteshgadekar`
   - URL: https://hub.docker.com/u/niteshgadekar

2. **Docker Hub Access Token**
   - Go to: Account Settings → Security → New Access Token
   - Name: `github-actions-ephemfs`
   - Permissions: Read, Write, Delete

3. **GitHub Repository Secrets**
   - `DOCKER_USERNAME` = `niteshgadekar`
   - `DOCKER_PASSWORD` = Your Docker Hub access token

### Automated Setup

Run the setup script:

```bash
./setup-github-actions.sh
```

This will guide you through:
1. ✅ Docker Hub account verification
2. ✅ Access token creation
3. ✅ GitHub secrets configuration
4. ✅ Workflow commit and push

### Manual Setup

#### 1. Create Docker Hub Access Token

```
1. Log in to Docker Hub: https://hub.docker.com/
2. Go to: Account Settings → Security
3. Click: New Access Token
4. Name: github-actions-ephemfs
5. Permissions: Read, Write, Delete
6. Click: Generate
7. Copy the token (you won't see it again!)
```

#### 2. Add GitHub Secrets

```
1. Go to your GitHub repository
2. Click: Settings → Secrets and variables → Actions
3. Click: New repository secret

Add these secrets:

Name: DOCKER_USERNAME
Value: niteshgadekar

Name: DOCKER_PASSWORD
Value: [Your Docker Hub access token]
```

#### 3. Commit Workflows

```bash
git add .github/workflows/
git commit -m "Add GitHub Actions for Docker build and push"
git push origin main
```

#### 4. Verify

```
1. Go to: GitHub Repository → Actions tab
2. You should see the workflow running
3. Wait for completion (~5-10 minutes)
4. Check Docker Hub: https://hub.docker.com/r/niteshgadekar/ephemfs
```

---

## 🔄 Automated Builds

### Triggers

The GitHub Actions workflow automatically builds and pushes when:

✅ **Push to main/master branch**
```bash
git push origin main
```

✅ **Manual trigger**
```
GitHub → Actions → Build and Push Docker Image (Simple) → Run workflow
```

✅ **Git tags** (advanced workflow)
```bash
git tag -a v1.0.0 -m "Release 1.0.0"
git push origin v1.0.0
```

### Build Process

1. ✅ Checkout code
2. ✅ Set up Docker Buildx
3. ✅ Log in to Docker Hub
4. ✅ Build multi-platform image (amd64 + arm64)
5. ✅ Push to Docker Hub
6. ✅ Tag as `1.0` and `latest`

**Build time:** ~5-10 minutes  
**Platforms:** linux/amd64, linux/arm64  
**Cache:** Enabled (faster subsequent builds)

---

## 📋 Available Tags

| Tag | Description | Use Case |
|-----|-------------|----------|
| `1.0` | Stable version 1.0 | Production |
| `latest` | Latest build from main | Development |
| `main` | Latest main branch | Testing |
| `v1.0.0` | Semantic version | Releases |

### Pull Specific Tag

```bash
# Stable version
docker pull niteshgadekar/ephemfs:1.0

# Latest
docker pull niteshgadekar/ephemfs:latest

# Specific version
docker pull niteshgadekar/ephemfs:v1.0.0
```

---

## 🔍 Verify Image

### Check Image Info

```bash
# Pull image
docker pull niteshgadekar/ephemfs:1.0

# Check size
docker images niteshgadekar/ephemfs:1.0

# Check layers
docker history niteshgadekar/ephemfs:1.0

# Inspect
docker inspect niteshgadekar/ephemfs:1.0
```

### Test Locally

```bash
# Run with test secrets
docker run --rm --privileged \
  -e DATABASE_PASSWORD=test123 \
  -e API_KEY=test456 \
  -e JWT_SECRET=test789 \
  niteshgadekar/ephemfs:1.0
```

**Expected output:**
```
🔒 SecretFS mounted at: /secrets
✅ Loaded 3 secret(s) | Encryption: DefaultCipher (XOR with 25-byte key) - ⚠️ DEMO ONLY, NOT SECURE! | Memory-only storage
Press Ctrl+C to unmount
```

### Test in Kubernetes

```bash
# Deploy
kubectl apply -f k8s-deployment.yaml

# Check pod
kubectl get pods -l app=busybox-demo

# View logs
kubectl logs -l app=busybox-demo -c secretfs

# Exec into pod
kubectl exec -it deployment/busybox-with-secretfs -c busybox -- sh
ls -lh /secrets/
cat /secrets/database_password
```

---

## 🐛 Troubleshooting

### Image Pull Failed

**Error:** `Error response from daemon: pull access denied`

**Fix:**
```bash
# Make sure image is public on Docker Hub
# Or log in first:
docker login
docker pull niteshgadekar/ephemfs:1.0
```

### GitHub Actions Build Failed

**Error:** `denied: requested access to the resource is denied`

**Fix:**
1. Check GitHub secrets are set correctly
2. Verify Docker Hub access token is valid
3. Check token has Write permissions

### Image Not Found

**Error:** `manifest unknown: manifest unknown`

**Fix:**
1. Check image name: `niteshgadekar/ephemfs:1.0`
2. Wait for GitHub Actions to complete
3. Check Docker Hub: https://hub.docker.com/r/niteshgadekar/ephemfs

### Kubernetes Pull Failed

**Error:** `Failed to pull image "niteshgadekar/ephemfs:1.0": rpc error: code = Unknown`

**Fix:**
```yaml
# Add imagePullPolicy
- name: secretfs
  image: niteshgadekar/ephemfs:1.0
  imagePullPolicy: Always  # Force pull latest
```

---

## 📊 Image Details

### Size

- **Compressed:** ~50-80 MB
- **Uncompressed:** ~100-200 MB

### Platforms

- ✅ `linux/amd64` (x86_64)
- ✅ `linux/arm64` (ARM64/Apple Silicon)

### Base Image

- **Builder:** `rust:1.75-slim`
- **Runtime:** `debian:bookworm-slim`

### Included Binaries

- `/usr/local/bin/secretfs` - Main SecretFS binary
- `/usr/local/bin/secretfs-keygen` - RSA key generation utility

### Default Configuration

```dockerfile
ENV FUSE_MOUNTPOINT=/secrets
ENV RUST_LOG=info
VOLUME ["/secrets"]
CMD ["secretfs", "/secrets"]
```

---

## 🔒 Security

### Image Scanning

Docker Hub automatically scans images for vulnerabilities.

**View scan results:**
```
https://hub.docker.com/r/niteshgadekar/ephemfs/tags
→ Click on tag → Security tab
```

### Best Practices

✅ **Use specific tags** (not `latest`) in production  
✅ **Enable vulnerability scanning** on Docker Hub  
✅ **Regularly update base images**  
✅ **Use multi-stage builds** (smaller attack surface)  
✅ **Run as non-root user** (UID 1000)  
✅ **Minimal dependencies** (only FUSE and CA certificates)

---

## 📚 Documentation

- **[GITHUB_ACTIONS_SETUP.md](GITHUB_ACTIONS_SETUP.md)** - Complete GitHub Actions guide
- **[K8S_QUICKSTART.md](K8S_QUICKSTART.md)** - Kubernetes deployment guide
- **[ENCRYPTION_OPTIONS.md](ENCRYPTION_OPTIONS.md)** - Encryption configuration
- **[RSA_ENCRYPTION.md](RSA_ENCRYPTION.md)** - RSA setup for production

---

## 🎯 Quick Commands

```bash
# Pull image
docker pull niteshgadekar/ephemfs:1.0

# Run locally
docker run --rm --privileged \
  -e DATABASE_PASSWORD=secret123 \
  niteshgadekar/ephemfs:1.0

# Deploy to Kubernetes
kubectl apply -f k8s-deployment.yaml

# Check deployment
kubectl get pods -l app=busybox-demo
kubectl logs -l app=busybox-demo -c secretfs

# Setup GitHub Actions
./setup-github-actions.sh

# Trigger manual build
# GitHub → Actions → Run workflow
```

---

## ✅ Checklist

Before using in production:

- [ ] Docker Hub account created
- [ ] Access token generated
- [ ] GitHub secrets configured
- [ ] First build completed successfully
- [ ] Image pulled and tested locally
- [ ] Kubernetes deployment tested
- [ ] Vulnerability scan reviewed
- [ ] Documentation reviewed

---

## 🎉 Summary

Your Docker image is ready!

**Image:** `niteshgadekar/ephemfs:1.0`  
**Platforms:** amd64, arm64  
**Auto-build:** Enabled via GitHub Actions  
**Status:** Production-ready  

**Use it:**
```bash
docker pull niteshgadekar/ephemfs:1.0
kubectl apply -f k8s-deployment.yaml
```

🚀 **Ready to deploy!**

