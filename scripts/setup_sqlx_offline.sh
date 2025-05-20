#!/bin/bash

echo "Setting up SQLx offline mode without database..."
echo "This script sets up the minimum required files for SQLx offline mode"
echo "without requiring a database connection."
echo

# Create necessary directories
mkdir -p .sqlx
mkdir -p .cargo

# Create the minimal required metadata.json
echo '{
  "format": 1,
  "db_name": "PostgreSQL",
  "db_version": "14.0",
  "checksums": {}
}' > .sqlx/metadata.json

# Create the cargo config for offline mode
echo '[env]
SQLX_OFFLINE = "true"' > .cargo/config.toml

echo "✓ SQLx offline mode configured successfully."
echo
echo "You can now build the application with:"
echo "  cargo build"
echo
echo "Note: This configuration doesn't include query metadata,"
echo "so some SQL queries may fail at runtime. This is intended"
echo "only for getting started with development." 