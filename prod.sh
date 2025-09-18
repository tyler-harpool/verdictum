#!/bin/bash

# Production script with full multi-tenant configuration
echo "Starting Spin in production mode..."
echo "94 federal court districts with isolated databases"
echo ""
echo "Available districts: SDNY, EDNY, CDCA, NDCA, SDTX, NDIL, DDC, NDNY, WDNY..."
echo "Use header: X-Court-District: SDNY"
echo ""

# Create prod stores directory if it doesn't exist
mkdir -p .spin/prod/stores

# Run with production config
spin up --runtime-config-file runtime-config.toml --listen 0.0.0.0:3000