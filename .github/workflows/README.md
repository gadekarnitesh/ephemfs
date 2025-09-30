# GitHub Actions Workflows

This directory contains automated workflows for building and pushing Docker images.

## 📋 Available Workflows

### 1. `docker-simple.yml` ⭐ (Recommended)

**Purpose:** Simple automatic Docker build and push

**Triggers:**
- Push to `main` or `master` branch
- Manual trigger via GitHub UI

**Output:**
- `niteshgadekar/ephemfs:1.0`
- `niteshgadekar/ephemfs:latest`

**Platforms:**
- `linux/amd64`
- `linux/arm64`

**Use this if:** You want simple automatic builds on every push

---

### 2. `docker-build-push.yml` (Advanced)

**Purpose:** Advanced Docker build with automatic versioning

**Triggers:**
- Push to `main` or `master` branch
- Git tags (e.g., `v1.0.0`)
- Pull requests
- Manual trigger via GitHub UI

**Output:**
- Branch-based tags (e.g., `main`, `develop`)
- Version tags (e.g., `v1.0.0`, `1.0.0`, `1.0`, `1`)
- SHA-based tags (e.g., `main-abc1234`)
- `latest` (on default branch)
- `1.0` (on default branch)

**Platforms:**
- `linux/amd64`
- `linux/arm64`

**Use this if:** You want automatic versioning and multiple tag strategies

---

## 🔧 Setup Required

### GitHub Secrets

Add these secrets to your repository:

| Secret Name | Value | Description |
|-------------|-------|-------------|
| `DOCKER_USERNAME` | `niteshgadekar` | Docker Hub username |
| `DOCKER_PASSWORD` | `dckr_pat_...` | Docker Hub access token |

**How to add:**
1. Go to: Repository → Settings → Secrets and variables → Actions
2. Click: New repository secret
3. Add both secrets

### Docker Hub Access Token

1. Log in to Docker Hub: https://hub.docker.com/
2. Go to: Account Settings → Security
3. Click: New Access Token
4. Name: `github-actions-ephemfs`
5. Permissions: Read, Write, Delete
6. Click: Generate
7. Copy the token and add as `DOCKER_PASSWORD` secret

---

## 🚀 Usage

### Automatic Build

Push to main/master:

```bash
git push origin main
```

The workflow will automatically build and push.

### Manual Build

1. Go to: **Actions** tab
2. Select workflow: **Build and Push Docker Image (Simple)**
3. Click: **Run workflow**
4. Select branch: **main**
5. Click: **Run workflow**

### Version Release

Create and push a version tag:

```bash
git tag -a v1.0.0 -m "Release 1.0.0"
git push origin v1.0.0
```

This triggers the advanced workflow and creates multiple version tags.

---

## 📊 Build Status

Check build status:
- Go to: **Actions** tab
- View recent workflow runs
- Click on a run to see detailed logs

---

## 🔍 Verify

After successful build:

```bash
# Pull image
docker pull niteshgadekar/ephemfs:1.0

# Check on Docker Hub
# https://hub.docker.com/r/niteshgadekar/ephemfs
```

---

## 📚 Documentation

- **[GITHUB_ACTIONS_SETUP.md](../../GITHUB_ACTIONS_SETUP.md)** - Complete setup guide
- **[GITHUB_ACTIONS_SUMMARY.md](../../GITHUB_ACTIONS_SUMMARY.md)** - Quick summary
- **[DOCKER_HUB_SETUP.md](../../DOCKER_HUB_SETUP.md)** - Docker Hub usage

---

## 🐛 Troubleshooting

### Build Failed

1. Check workflow logs: Actions → Select run → View logs
2. Common issues:
   - Invalid Docker Hub credentials
   - Rust compilation errors
   - Network issues

### Image Not Found

1. Check Docker Hub: https://hub.docker.com/r/niteshgadekar/ephemfs
2. Verify build completed successfully
3. Check image name matches: `niteshgadekar/ephemfs:1.0`

---

## ✅ Quick Setup

Run the setup script:

```bash
./setup-github-actions.sh
```

This will guide you through the complete setup process.

