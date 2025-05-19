#!/bin/bash

echo "Setting up SQLx for offline development..."

# Check if database is reachable
if pg_isready -h localhost -p 5432 -d txn_manager -U postgres; then
    echo "Database is available. Preparing SQLx for offline development..."
    
    # Prepare SQLx for offline development - without any flags
    DATABASE_URL=postgres://postgres:postgres@localhost:5432/txn_manager cargo sqlx prepare
    
    if [ $? -eq 0 ]; then
        echo "SQLx prepared successfully. You can now build without a database connection."
        echo "To build, use: SQLX_OFFLINE=true cargo build"
    else
        echo "Failed to prepare SQLx. Make sure your database is properly set up."
    fi
else
    echo "Database is not available. Please set up your database first."
    echo "You can use the setup_local_database.sh script or Docker."
fi 