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
if ! pg_isready &> /dev/null; then
    echo "PostgreSQL service is not running. Starting PostgreSQL..."
    echo "On systemd systems: sudo systemctl start postgresql"
    echo "On Ubuntu/Debian: sudo service postgresql start"
    exit 1
fi

# Create database if it doesn't exist
echo "Creating database 'txn_manager' if it doesn't exist..."
if ! psql -U postgres -lqt | cut -d \| -f 1 | grep -qw txn_manager; then
    sudo -u postgres psql -c "CREATE DATABASE txn_manager"
    echo "Database 'txn_manager' created."
else
    echo "Database 'txn_manager' already exists."
fi

# Create postgres user if it doesn't exist with password postgres
echo "Creating user 'postgres' with password 'postgres' if needed..."
if ! sudo -u postgres psql -tAc "SELECT 1 FROM pg_roles WHERE rolname='postgres'" | grep -q 1; then
    sudo -u postgres psql -c "CREATE USER postgres WITH PASSWORD 'postgres'"
    echo "User 'postgres' created."
else
    # Update password for existing postgres user
    sudo -u postgres psql -c "ALTER USER postgres WITH PASSWORD 'postgres'"
    echo "Password for user 'postgres' updated."
fi

# Grant privileges
echo "Granting privileges on 'txn_manager' database to 'postgres' user..."
sudo -u postgres psql -c "GRANT ALL PRIVILEGES ON DATABASE txn_manager TO postgres"

echo "Database setup complete. You can now run the application with: cargo run"
echo "Database URL: postgres://postgres:postgres@localhost:5432/txn_manager" 