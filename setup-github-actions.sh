#!/bin/bash

# Setup script for GitHub Actions Docker build and push
# This script helps you configure GitHub secrets for Docker Hub

set -e

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  GitHub Actions Setup for Docker Build & Push             ║${NC}"
echo -e "${BLUE}║  Target: niteshgadekar/ephemfs:1.0                         ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Check if gh CLI is installed
if command -v gh &> /dev/null; then
    GH_CLI_AVAILABLE=true
    echo -e "${GREEN}✅ GitHub CLI (gh) detected${NC}"
else
    GH_CLI_AVAILABLE=false
    echo -e "${YELLOW}⚠️  GitHub CLI (gh) not found${NC}"
    echo "   Install from: https://cli.github.com/"
fi
echo ""

# Step 1: Docker Hub Account
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}Step 1: Docker Hub Account${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo "Do you have a Docker Hub account?"
echo "  Username: niteshgadekar"
echo "  URL: https://hub.docker.com/u/niteshgadekar"
echo ""
read -p "Continue? (y/n) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo ""
    echo "Please create a Docker Hub account first:"
    echo "  1. Go to https://hub.docker.com/"
    echo "  2. Sign up for a free account"
    echo "  3. Verify your email"
    echo "  4. Run this script again"
    exit 0
fi

# Step 2: Docker Hub Access Token
echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}Step 2: Docker Hub Access Token${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo "You need to create a Docker Hub access token:"
echo ""
echo "  1. Log in to Docker Hub: https://hub.docker.com/"
echo "  2. Go to: Account Settings → Security"
echo "  3. Click: New Access Token"
echo "  4. Name: github-actions-ephemfs"
echo "  5. Permissions: Read, Write, Delete"
echo "  6. Click: Generate"
echo "  7. Copy the token (you won't see it again!)"
echo ""
read -p "Have you created the access token? (y/n) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo ""
    echo "Please create the access token first, then run this script again."
    exit 0
fi

# Step 3: Configure GitHub Secrets
echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}Step 3: Configure GitHub Secrets${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

if [ "$GH_CLI_AVAILABLE" = true ]; then
    echo "We can configure secrets automatically using GitHub CLI."
    echo ""
    read -p "Configure secrets automatically? (y/n) " -n 1 -r
    echo
    
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo ""
        echo "Enter your Docker Hub access token:"
        read -s DOCKER_TOKEN
        echo ""
        
        echo "Setting DOCKER_USERNAME secret..."
        echo "niteshgadekar" | gh secret set DOCKER_USERNAME
        
        echo "Setting DOCKER_PASSWORD secret..."
        echo "$DOCKER_TOKEN" | gh secret set DOCKER_PASSWORD
        
        echo ""
        echo -e "${GREEN}✅ Secrets configured successfully!${NC}"
        echo ""
        
        # Verify
        echo "Verifying secrets..."
        gh secret list
        echo ""
    else
        echo ""
        echo "Manual configuration:"
        echo ""
        echo "  1. Go to your GitHub repository"
        echo "  2. Click: Settings → Secrets and variables → Actions"
        echo "  3. Click: New repository secret"
        echo ""
        echo "  Add these secrets:"
        echo ""
        echo "  Name: DOCKER_USERNAME"
        echo "  Value: niteshgadekar"
        echo ""
        echo "  Name: DOCKER_PASSWORD"
        echo "  Value: [Your Docker Hub access token]"
        echo ""
    fi
else
    echo "Manual configuration required:"
    echo ""
    echo "  1. Go to your GitHub repository"
    echo "  2. Click: Settings → Secrets and variables → Actions"
    echo "  3. Click: New repository secret"
    echo ""
    echo "  Add these secrets:"
    echo ""
    echo "  Name: DOCKER_USERNAME"
    echo "  Value: niteshgadekar"
    echo ""
    echo "  Name: DOCKER_PASSWORD"
    echo "  Value: [Your Docker Hub access token]"
    echo ""
    read -p "Press Enter when done..."
fi

# Step 4: Commit and Push Workflows
echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}Step 4: Commit and Push Workflows${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

if [ -d ".github/workflows" ]; then
    echo "Workflow files found:"
    ls -lh .github/workflows/*.yml
    echo ""
    
    read -p "Commit and push workflows? (y/n) " -n 1 -r
    echo
    
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo ""
        echo "Committing workflows..."
        git add .github/workflows/
        git commit -m "Add GitHub Actions for Docker build and push" || echo "Already committed"
        
        echo ""
        echo "Pushing to remote..."
        git push origin main || git push origin master || echo "Push failed - please push manually"
        
        echo ""
        echo -e "${GREEN}✅ Workflows pushed!${NC}"
    fi
else
    echo -e "${RED}❌ .github/workflows directory not found${NC}"
    echo "   Please ensure workflow files are in .github/workflows/"
fi

# Step 5: Summary
echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}Setup Complete!${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo -e "${GREEN}✅ GitHub Actions configured for Docker build and push${NC}"
echo ""
echo "Next steps:"
echo ""
echo "  1. Go to your GitHub repository"
echo "  2. Click: Actions tab"
echo "  3. You should see the workflow running"
echo ""
echo "  Or trigger manually:"
echo "  - Actions → Build and Push Docker Image (Simple) → Run workflow"
echo ""
echo "Once built, your image will be available at:"
echo "  ${GREEN}docker pull niteshgadekar/ephemfs:1.0${NC}"
echo ""
echo "Use in Kubernetes:"
echo "  ${GREEN}kubectl apply -f k8s-deployment.yaml${NC}"
echo ""
echo "Documentation:"
echo "  - GITHUB_ACTIONS_SETUP.md - Complete setup guide"
echo "  - K8S_QUICKSTART.md - Kubernetes deployment guide"
echo ""
echo -e "${BLUE}🚀 Ready to build!${NC}"

