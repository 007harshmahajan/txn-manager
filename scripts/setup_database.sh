#!/bin/bash

echo "Setting up development database in Docker..."

# Check if Docker is installed
if ! command -v docker &> /dev/null; then
    echo "Docker is not installed. Please install Docker first."
    exit 1
fi

# Check if the container is already running
if docker ps | grep -q "txn-manager-postgres"; then
    echo "PostgreSQL container is already running."
else
    # Check if the container exists but is stopped
    if docker ps -a | grep -q "txn-manager-postgres"; then
        echo "Starting existing PostgreSQL container..."
        docker start txn-manager-postgres
    else
        echo "Creating new PostgreSQL container..."
        docker run --name txn-manager-postgres \
            -e POSTGRES_PASSWORD=postgres \
            -e POSTGRES_USER=postgres \
            -e POSTGRES_DB=txn_manager \
            -p 5433:5432 \
            -d postgres:16
    fi
fi

# Wait for PostgreSQL to be ready
echo "Waiting for PostgreSQL to be ready..."
sleep 5

# Try to connect and prepare SQLx metadata
if pg_isready -h localhost -p 5433 -d txn_manager -U postgres; then
    echo "Database is ready. Preparing SQLx metadata for offline mode..."
    
    # Create .sqlx directory if it doesn't exist
    mkdir -p .sqlx
    
    # Prepare SQLx metadata
    if command -v cargo-sqlx &> /dev/null; then
        echo "Running cargo sqlx prepare..."
        DATABASE_URL=postgres://postgres:postgres@localhost:5433/txn_manager cargo sqlx prepare
    else
        echo "cargo-sqlx not found. Installing it..."
        cargo install sqlx-cli --no-default-features --features postgres
        echo "Running cargo sqlx prepare..."
        DATABASE_URL=postgres://postgres:postgres@localhost:5433/txn_manager cargo sqlx prepare
    fi
    
    # Create or update .cargo/config.toml for offline mode
    mkdir -p .cargo
    echo '[env]
SQLX_OFFLINE = "true"' > .cargo/config.toml
    
    echo "SQLx metadata prepared for offline development."
else
    echo "Database connection failed. SQLx metadata preparation skipped."
    echo "You may need to run manually: DATABASE_URL=postgres://postgres:postgres@localhost:5433/txn_manager cargo sqlx prepare"
fi

# Create directories for performance testing
echo "Creating directories for performance testing..."
mkdir -p performance_results

echo "Database setup complete. You can now run the application with: cargo run"
echo "Database URL: postgres://postgres:postgres@localhost:5433/txn_manager" 