# GitHub Actions - Docker Build & Push Setup

## 🎯 Overview

Automated Docker image building and pushing to Docker Hub using GitHub Actions.

**Image Location:** `niteshgadekar/ephemfs:1.0`

---

## 📋 Workflows Created

### 1. **docker-simple.yml** (Recommended)

Simple workflow that builds and pushes to Docker Hub on every push to main/master.

**Tags created:**
- `niteshgadekar/ephemfs:1.0`
- `niteshgadekar/ephemfs:latest`

**Platforms:**
- `linux/amd64` (x86_64)
- `linux/arm64` (ARM64/Apple Silicon)

**Triggers:**
- Push to `main` or `master` branch
- Manual trigger via GitHub UI

### 2. **docker-build-push.yml** (Advanced)

Advanced workflow with automatic versioning and multiple tag strategies.

**Tags created:**
- Branch name (e.g., `main`, `develop`)
- Git SHA (e.g., `main-abc1234`)
- Semantic versions (e.g., `v1.0.0`, `1.0`, `1`)
- `latest` (on default branch)
- `1.0` (on default branch)

**Triggers:**
- Push to `main` or `master` branch
- Git tags (e.g., `v1.0.0`)
- Pull requests
- Manual trigger via GitHub UI

---

## 🔧 Setup Instructions

### Step 1: Create Docker Hub Account

If you don't have one:
1. Go to https://hub.docker.com/
2. Sign up for a free account
3. Verify your email

### Step 2: Create Docker Hub Access Token

1. Log in to Docker Hub
2. Go to **Account Settings** → **Security**
3. Click **New Access Token**
4. Name: `github-actions-ephemfs`
5. Permissions: **Read, Write, Delete**
6. Click **Generate**
7. **Copy the token** (you won't see it again!)

### Step 3: Add Secrets to GitHub Repository

1. Go to your GitHub repository
2. Click **Settings** → **Secrets and variables** → **Actions**
3. Click **New repository secret**

**Add these two secrets:**

| Secret Name | Value |
|-------------|-------|
| `DOCKER_USERNAME` | `niteshgadekar` |
| `DOCKER_PASSWORD` | Your Docker Hub access token from Step 2 |

**Screenshot guide:**
```
Settings → Secrets and variables → Actions → New repository secret

Name: DOCKER_USERNAME
Secret: niteshgadekar
[Add secret]

Name: DOCKER_PASSWORD
Secret: dckr_pat_xxxxxxxxxxxxxxxxxxxxx
[Add secret]
```

### Step 4: Commit and Push Workflows

The workflows are already created in `.github/workflows/`:
- `docker-simple.yml`
- `docker-build-push.yml`

Commit and push them:

```bash
git add .github/workflows/
git commit -m "Add GitHub Actions for Docker build and push"
git push origin main
```

### Step 5: Verify Workflow Execution

1. Go to your GitHub repository
2. Click **Actions** tab
3. You should see the workflow running
4. Click on the workflow run to see details

---

## 🚀 Usage

### Automatic Build (on push)

Simply push to main/master:

```bash
git add .
git commit -m "Update code"
git push origin main
```

The workflow will automatically:
1. ✅ Build Docker image
2. ✅ Push to `niteshgadekar/ephemfs:1.0`
3. ✅ Push to `niteshgadekar/ephemfs:latest`

### Manual Build (workflow_dispatch)

1. Go to **Actions** tab
2. Select **Build and Push Docker Image (Simple)**
3. Click **Run workflow**
4. Select branch (usually `main`)
5. Click **Run workflow**

### Build with Git Tags (Advanced)

Create and push a version tag:

```bash
# Create a tag
git tag -a v1.0.0 -m "Release version 1.0.0"

# Push the tag
git push origin v1.0.0
```

This will create additional tags:
- `niteshgadekar/ephemfs:v1.0.0`
- `niteshgadekar/ephemfs:1.0.0`
- `niteshgadekar/ephemfs:1.0`
- `niteshgadekar/ephemfs:1`

---

## 🔍 Verify Image on Docker Hub

### Check on Docker Hub Website

1. Go to https://hub.docker.com/r/niteshgadekar/ephemfs
2. You should see your image with tags

### Pull and Test Locally

```bash
# Pull the image
docker pull niteshgadekar/ephemfs:1.0

# Run it
docker run --rm --privileged \
  -e DATABASE_PASSWORD=test123 \
  -e API_KEY=test456 \
  niteshgadekar/ephemfs:1.0
```

### Use in Kubernetes

Update `k8s-deployment.yaml`:

```yaml
- name: secretfs
  image: niteshgadekar/ephemfs:1.0  # ✅ Your image!
  env:
  - name: FUSE_MOUNTPOINT
    value: "/secrets"
  # ... rest of config
```

Deploy:

```bash
kubectl apply -f k8s-deployment.yaml
```

---

## 📊 Workflow Details

### Simple Workflow (docker-simple.yml)

```yaml
name: Build and Push Docker Image (Simple)

on:
  push:
    branches:
      - main
      - master
  workflow_dispatch:

jobs:
  build:
    runs-on: ubuntu-latest
    
    steps:
      - Checkout code
      - Set up Docker Buildx
      - Log in to Docker Hub
      - Build and push Docker image
        Tags: niteshgadekar/ephemfs:1.0, niteshgadekar/ephemfs:latest
        Platforms: linux/amd64, linux/arm64
```

**Build time:** ~5-10 minutes  
**Cache:** Enabled (faster subsequent builds)  
**Multi-platform:** Yes (amd64 + arm64)

### Advanced Workflow (docker-build-push.yml)

```yaml
name: Build and Push Docker Image

on:
  push:
    branches: [main, master]
    tags: ['v*.*.*']
  pull_request:
    branches: [main, master]
  workflow_dispatch:

jobs:
  build-and-push:
    runs-on: ubuntu-latest
    
    steps:
      - Checkout code
      - Set up Docker Buildx
      - Log in to Docker Hub
      - Extract metadata (automatic tagging)
      - Build and push Docker image
        Tags: Multiple (based on git ref)
        Platforms: linux/amd64, linux/arm64
```

**Build time:** ~5-10 minutes  
**Cache:** Enabled  
**Multi-platform:** Yes  
**Auto-versioning:** Yes

---

## 🐛 Troubleshooting

### Error: "denied: requested access to the resource is denied"

**Cause:** Invalid Docker Hub credentials

**Fix:**
1. Verify `DOCKER_USERNAME` is correct: `niteshgadekar`
2. Regenerate Docker Hub access token
3. Update `DOCKER_PASSWORD` secret in GitHub

### Error: "failed to solve: process "/bin/sh -c cargo build --release" did not complete successfully"

**Cause:** Build failure in Dockerfile

**Fix:**
1. Test build locally: `docker build -t test .`
2. Check Rust dependencies in `Cargo.toml`
3. Check for compilation errors

### Error: "buildx failed with: ERROR: failed to solve: failed to push"

**Cause:** Network or registry issue

**Fix:**
1. Check Docker Hub status: https://status.docker.com/
2. Retry the workflow
3. Check repository permissions on Docker Hub

### Workflow doesn't trigger

**Cause:** Workflow file not in correct location or syntax error

**Fix:**
1. Ensure file is in `.github/workflows/`
2. Check YAML syntax: https://www.yamllint.com/
3. Check branch name matches trigger (main vs master)

### Build is slow

**Cause:** No cache or building from scratch

**Fix:**
1. Wait for first build to complete (creates cache)
2. Subsequent builds will be faster (~2-3 minutes)
3. Cache is stored in GitHub Actions cache

---

## 📈 Monitoring

### View Workflow Runs

```
GitHub Repository → Actions → Select workflow → View runs
```

### View Build Logs

```
Actions → Select run → Click on job → Expand steps
```

### Check Image Size

```bash
docker images niteshgadekar/ephemfs:1.0
```

Expected size: ~100-200 MB (multi-stage build)

### Check Image Layers

```bash
docker history niteshgadekar/ephemfs:1.0
```

---

## 🔒 Security Best Practices

### ✅ Do's

- ✅ Use Docker Hub access tokens (not password)
- ✅ Set token permissions to minimum required
- ✅ Use GitHub Secrets for credentials
- ✅ Enable 2FA on Docker Hub account
- ✅ Regularly rotate access tokens
- ✅ Use multi-stage builds (smaller images)
- ✅ Scan images for vulnerabilities

### ❌ Don'ts

- ❌ Don't commit credentials to git
- ❌ Don't use your Docker Hub password
- ❌ Don't share access tokens
- ❌ Don't use overly permissive tokens
- ❌ Don't disable security scanning

---

## 🎯 Next Steps

### 1. Enable Docker Hub Vulnerability Scanning

Docker Hub automatically scans images for vulnerabilities.

View results:
```
https://hub.docker.com/r/niteshgadekar/ephemfs/tags
→ Click on tag → Security tab
```

### 2. Add Image Signing (Optional)

Sign images with Docker Content Trust:

```yaml
- name: Sign image
  run: |
    export DOCKER_CONTENT_TRUST=1
    docker push niteshgadekar/ephemfs:1.0
```

### 3. Add Automated Tests

Add testing step before push:

```yaml
- name: Test image
  run: |
    docker run --rm niteshgadekar/ephemfs:1.0 secretfs --version
```

### 4. Create Release Workflow

Automate releases with changelog:

```yaml
- name: Create Release
  uses: actions/create-release@v1
  with:
    tag_name: ${{ github.ref }}
    release_name: Release ${{ github.ref }}
```

---

## 📚 Resources

- **GitHub Actions Docs:** https://docs.github.com/en/actions
- **Docker Build Push Action:** https://github.com/docker/build-push-action
- **Docker Hub:** https://hub.docker.com/r/niteshgadekar/ephemfs
- **Dockerfile Best Practices:** https://docs.docker.com/develop/dev-best-practices/

---

## ✅ Checklist

Before pushing to production:

- [ ] Docker Hub account created
- [ ] Access token generated
- [ ] GitHub secrets configured (`DOCKER_USERNAME`, `DOCKER_PASSWORD`)
- [ ] Workflow files committed (`.github/workflows/*.yml`)
- [ ] First build completed successfully
- [ ] Image pulled and tested locally
- [ ] Kubernetes deployment updated with new image
- [ ] Documentation updated

---

## 🎉 Summary

You now have automated Docker image building and pushing!

**Every time you push to main:**
1. ✅ GitHub Actions triggers
2. ✅ Docker image builds (multi-platform)
3. ✅ Image pushes to Docker Hub
4. ✅ Available at `niteshgadekar/ephemfs:1.0`

**Use in Kubernetes:**
```yaml
image: niteshgadekar/ephemfs:1.0
```

**Pull locally:**
```bash
docker pull niteshgadekar/ephemfs:1.0
```

🚀 **Ready to automate!**

