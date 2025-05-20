#!/bin/bash

echo "Setting up local PostgreSQL database..."

# Check if psql is installed
if ! command -v psql &> /dev/null; then
    echo "PostgreSQL client (psql) is not installed. Please install PostgreSQL first."
    echo "On Ubuntu/Debian: sudo apt install postgresql postgresql-contrib"
    echo "On Fedora/RHEL: sudo dnf install postgresql postgresql-server"
    echo "On Arch: sudo pacman -S postgresql"
    exit 1
fi

# Check if PostgreSQL service is running
if ! pg_isready -h localhost -p 5433 -U postgres &> /dev/null; then
    echo "PostgreSQL service is not running. Starting PostgreSQL..."
    echo "Attempting to start PostgreSQL service..."
    
    # Try different methods to start PostgreSQL
    if command -v systemctl &> /dev/null; then
        sudo systemctl start postgresql
    elif command -v service &> /dev/null; then
        sudo service postgresql start
    else
        echo "Could not determine how to start PostgreSQL service."
        echo "Please start it manually and try again."
        exit 1
    fi
    
    # Wait for PostgreSQL to start
    echo "Waiting for PostgreSQL to start..."
    sleep 5
    
    # Check again if PostgreSQL is running
    if ! pg_isready -h localhost -p 5433 -U postgres &> /dev/null; then
        echo "Failed to start PostgreSQL service. Please start it manually and try again."
        exit 1
    fi
    
    echo "PostgreSQL service started successfully."
fi

# Set PGPASSWORD environment variable for passwordless command-line access
export PGPASSWORD=postgres

# Create database if it doesn't exist
echo "Creating database 'txn_manager' if it doesn't exist..."
if ! psql -h localhost -U postgres -lqt | cut -d \| -f 1 | grep -qw txn_manager; then
    psql -h localhost -U postgres -c "CREATE DATABASE txn_manager"
    echo "Database 'txn_manager' created."
else
    echo "Database 'txn_manager' already exists."
fi

# Run migrations to set up schema
echo "Running migrations to set up database schema..."
psql -h localhost -U postgres -d txn_manager -f migrations/20240101000001_initial_schema.sql

# Prepare SQLx metadata for offline mode
echo "Preparing SQLx metadata for offline mode..."
mkdir -p .sqlx

# Run SQLx prepare without --merged flag
DATABASE_URL=postgres://postgres:postgres@localhost:5433/txn_manager cargo sqlx prepare

# Create or update .cargo/config.toml for offline mode
mkdir -p .cargo
echo '[env]
SQLX_OFFLINE = "true"' > .cargo/config.toml

# Create directories for performance testing
echo "Creating directories for performance testing..."
mkdir -p performance_results

echo "Database and SQLx setup complete! You can now run the application with: cargo run"
echo "Database URL: postgres://postgres:postgres@localhost:5433/txn_manager" 