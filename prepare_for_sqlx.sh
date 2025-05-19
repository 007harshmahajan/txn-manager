#!/bin/bash

# This script prepares the database schema and SQLx metadata for offline mode

echo "Preparing environment for SQLx..."

# Check if PostgreSQL is running using TCP connection instead of socket
if ! pg_isready -h localhost -p 5432 -U postgres; then
    echo "PostgreSQL database is not available on localhost. Setting up with Docker..."
    
    # Check if docker is available
    if ! command -v docker &> /dev/null; then
        echo "Docker not found. Please install Docker or start PostgreSQL manually."
        exit 1
    fi
    
    # Start PostgreSQL with Docker
    if docker ps | grep -q "txn-manager-postgres"; then
        echo "PostgreSQL container is already running."
    else
        if docker ps -a | grep -q "txn-manager-postgres"; then
            echo "Starting existing PostgreSQL container..."
            docker start txn-manager-postgres
        else
            echo "Creating new PostgreSQL container..."
            docker run --name txn-manager-postgres \
                -e POSTGRES_PASSWORD=postgres \
                -e POSTGRES_USER=postgres \
                -e POSTGRES_DB=txn_manager \
                -p 5432:5432 \
                -d postgres:16
        fi
    fi
    
    # Wait for PostgreSQL to start
    echo "Waiting for PostgreSQL to start..."
    sleep 10
    
    # Verify connection again
    if ! pg_isready -h localhost -p 5432 -U postgres; then
        echo "Still unable to connect to PostgreSQL. Check Docker and try again."
        exit 1
    fi
fi

# Set up the database URL
DB_URL="postgres://postgres:postgres@localhost:5432/txn_manager"

# Create the database if it doesn't exist (using -h flag to force TCP connection)
if ! PGPASSWORD=postgres psql -h localhost -U postgres -lqt | cut -d \| -f 1 | grep -qw txn_manager; then
    echo "Creating database 'txn_manager'..."
    PGPASSWORD=postgres createdb -h localhost -U postgres txn_manager
    echo "Database 'txn_manager' created."
else
    echo "Database 'txn_manager' already exists."
fi

# Run the migrations manually to set up the schema (using -h flag for TCP)
echo "Running migrations to create the schema..."
PGPASSWORD=postgres psql -h localhost -U postgres -d txn_manager -f migrations/20240101000001_initial_schema.sql

# Install SQLx CLI if not available
if ! command -v sqlx &> /dev/null; then
    echo "Installing SQLx CLI..."
    cargo install sqlx-cli --no-default-features --features postgres
fi

# Prepare SQLx metadata for offline mode
echo "Preparing SQLx metadata for offline mode..."
DATABASE_URL=$DB_URL cargo sqlx prepare

# Create .cargo/config.toml for offline mode
mkdir -p .cargo
echo '[env]
SQLX_OFFLINE = "true"
SQLX_OFFLINE_DIR = ".sqlx"' > .cargo/config.toml

echo "SQLx preparation complete. You can now build without a database connection using:"
echo "cargo build" 