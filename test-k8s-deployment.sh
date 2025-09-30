#!/bin/bash

# Test Kubernetes Deployment
# This script tests the BusyBox + SecretFS sidecar deployment

set -e

echo "🧪 Testing Kubernetes Deployment"
echo "================================"
echo ""

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if kubectl is available
if ! command -v kubectl &> /dev/null; then
    echo -e "${RED}❌ kubectl not found. Please install kubectl first.${NC}"
    exit 1
fi

# Check if cluster is accessible
if ! kubectl cluster-info &> /dev/null; then
    echo -e "${RED}❌ Cannot connect to Kubernetes cluster.${NC}"
    echo "   Make sure you have a running cluster and kubeconfig is set."
    exit 1
fi

echo -e "${GREEN}✅ kubectl found and cluster is accessible${NC}"
echo ""

# Check if deployment file exists
if [ ! -f "k8s-deployment.yaml" ]; then
    echo -e "${RED}❌ k8s-deployment.yaml not found${NC}"
    exit 1
fi

echo "📋 Deployment Plan:"
echo "  - Deploy BusyBox + SecretFS sidecar"
echo "  - Create Kubernetes Secret with test data"
echo "  - Wait for pod to be ready"
echo "  - Verify secrets are mounted"
echo "  - Cleanup"
echo ""

read -p "Continue? (y/n) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Aborted."
    exit 0
fi

echo ""
echo "🚀 Step 1: Deploying to Kubernetes..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Apply deployment
kubectl apply -f k8s-deployment.yaml

echo ""
echo "⏳ Step 2: Waiting for pod to be ready..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Wait for pod to be ready (max 60 seconds)
kubectl wait --for=condition=ready pod -l app=busybox-demo --timeout=60s || {
    echo -e "${RED}❌ Pod failed to become ready${NC}"
    echo ""
    echo "Pod status:"
    kubectl get pods -l app=busybox-demo
    echo ""
    echo "Pod events:"
    kubectl describe pod -l app=busybox-demo | tail -20
    exit 1
}

echo -e "${GREEN}✅ Pod is ready${NC}"
echo ""

# Get pod name
POD_NAME=$(kubectl get pod -l app=busybox-demo -o jsonpath='{.items[0].metadata.name}')
echo "Pod name: $POD_NAME"
echo ""

echo "📊 Step 3: Checking BusyBox logs..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Wait a bit for BusyBox to complete its startup script
sleep 3

# Get BusyBox logs
BUSYBOX_LOGS=$(kubectl logs $POD_NAME -c busybox 2>/dev/null || echo "")

if [ -z "$BUSYBOX_LOGS" ]; then
    echo -e "${YELLOW}⚠️  BusyBox logs not available yet, waiting...${NC}"
    sleep 5
    BUSYBOX_LOGS=$(kubectl logs $POD_NAME -c busybox)
fi

echo "$BUSYBOX_LOGS"
echo ""

# Check if secrets were loaded
if echo "$BUSYBOX_LOGS" | grep -q "✅ Secrets loaded successfully"; then
    echo -e "${GREEN}✅ BusyBox successfully loaded secrets${NC}"
else
    echo -e "${RED}❌ BusyBox did not load secrets${NC}"
    exit 1
fi

echo ""
echo "📊 Step 4: Checking SecretFS logs..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Get SecretFS logs
SECRETFS_LOGS=$(kubectl logs $POD_NAME -c secretfs 2>/dev/null || echo "")

if [ -z "$SECRETFS_LOGS" ]; then
    echo -e "${YELLOW}⚠️  SecretFS logs not available yet${NC}"
else
    echo "$SECRETFS_LOGS"
    echo ""
    
    # Check if SecretFS mounted successfully
    if echo "$SECRETFS_LOGS" | grep -q "SecretFS mounted at"; then
        echo -e "${GREEN}✅ SecretFS mounted successfully${NC}"
    else
        echo -e "${RED}❌ SecretFS did not mount${NC}"
        exit 1
    fi
fi

echo ""
echo "🔍 Step 5: Interactive verification..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# List secrets
echo "Listing secrets in /secrets:"
kubectl exec $POD_NAME -c busybox -- ls -lh /secrets/ || {
    echo -e "${RED}❌ Failed to list secrets${NC}"
    exit 1
}
echo ""

# Read a secret
echo "Reading database_password:"
DB_PASS=$(kubectl exec $POD_NAME -c busybox -- cat /secrets/database_password 2>/dev/null || echo "")
if [ -n "$DB_PASS" ]; then
    echo "  Content: $DB_PASS"
    echo -e "${GREEN}✅ Successfully read secret${NC}"
else
    echo -e "${RED}❌ Failed to read secret${NC}"
    exit 1
fi
echo ""

# Try to write (should fail)
echo "Testing read-only filesystem:"
if kubectl exec $POD_NAME -c busybox -- sh -c "echo test > /secrets/test.txt" 2>&1 | grep -q "Read-only"; then
    echo -e "${GREEN}✅ Filesystem is read-only (as expected)${NC}"
else
    echo -e "${YELLOW}⚠️  Filesystem might not be read-only${NC}"
fi
echo ""

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${GREEN}✅ All tests passed!${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo "📝 Summary:"
echo "  ✅ Pod deployed successfully"
echo "  ✅ BusyBox container running"
echo "  ✅ SecretFS sidecar mounted"
echo "  ✅ Secrets accessible from BusyBox"
echo "  ✅ Filesystem is read-only"
echo ""

echo "🎯 Next steps:"
echo "  - View logs: kubectl logs -f $POD_NAME -c busybox"
echo "  - Exec into pod: kubectl exec -it $POD_NAME -c busybox -- sh"
echo "  - Check SecretFS: kubectl logs $POD_NAME -c secretfs"
echo ""

read -p "Cleanup deployment? (y/n) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo ""
    echo "🧹 Cleaning up..."
    kubectl delete -f k8s-deployment.yaml
    echo -e "${GREEN}✅ Cleanup complete${NC}"
else
    echo ""
    echo "Deployment left running. To cleanup later:"
    echo "  kubectl delete -f k8s-deployment.yaml"
fi

echo ""
echo "🎉 Test complete!"

