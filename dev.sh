#!/bin/bash

# Development script with hot reload and sample districts
echo "Starting Spin in development mode with hot reload..."
echo ""
echo "Available test districts: SDNY, EDNY, NDCA, CDCA"
echo ""
echo "Examples:"
echo "  curl -H 'X-Court-District: SDNY' http://localhost:3000/api/attorneys"
echo "  curl -H 'X-Court-District: EDNY' http://localhost:3000/api/cases"
echo ""

# Create dev stores directory if it doesn't exist
mkdir -p .spin/dev/stores

# Run with watch and development config
spin watch --runtime-config-file dev-config.toml