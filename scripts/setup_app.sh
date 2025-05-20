#!/bin/bash

echo "Transaction Manager Setup Script"
echo "==============================="
echo "This script will set up the database and prepare the environment for running the application."
echo

# Create directories
mkdir -p .sqlx
mkdir -p .cargo
mkdir -p performance_results

# Determine if PostgreSQL is installed locally
if command -v psql &> /dev/null; then
    POSTGRES_INSTALLED=true
    echo "✓ PostgreSQL client found."
else
    POSTGRES_INSTALLED=false
    echo "✗ PostgreSQL client not found."
fi

# Check if Docker is installed and has proper permissions
if command -v docker &> /dev/null; then
    # Test if we can run Docker without sudo
    if docker ps &> /dev/null; then
        DOCKER_AVAILABLE=true
        echo "✓ Docker is available and properly configured."
    else
        echo "✗ Docker is installed but you don't have permission to use it."
        echo "  You might need to add your user to the docker group:"
        echo "  sudo usermod -aG docker $USER"
        echo "  (Then log out and log back in)"
        DOCKER_AVAILABLE=false
    fi
else
    DOCKER_AVAILABLE=false
    echo "✗ Docker is not installed."
fi

# Try local PostgreSQL first
if [ "$POSTGRES_INSTALLED" = true ]; then
    echo
    echo "Attempting to use local PostgreSQL..."
    
    # Check if PostgreSQL server is running
    if pg_isready -h localhost -p 5433 -U postgres &> /dev/null; then
        echo "✓ PostgreSQL server is running."
        POSTGRES_RUNNING=true
    else
        echo "✗ PostgreSQL server is not running. Attempting to start it..."
        
        # Try to start PostgreSQL service
        if command -v systemctl &> /dev/null; then
            sudo systemctl start postgresql
        elif command -v service &> /dev/null; then
            sudo service postgresql start
        fi
        
        # Check again after trying to start
        sleep 3
        if pg_isready -h localhost -p 5433 -U postgres &> /dev/null; then
            echo "✓ PostgreSQL server started successfully."
            POSTGRES_RUNNING=true
        else
            echo "✗ Failed to start PostgreSQL server."
            POSTGRES_RUNNING=false
        fi
    fi
    
    # If PostgreSQL is running, set up the database
    if [ "$POSTGRES_RUNNING" = true ]; then
        # Set password for PostgreSQL commands
        export PGPASSWORD=postgres
        
        # Check if the database exists
        if psql -h localhost -U postgres -lqt | cut -d \| -f 1 | grep -qw txn_manager; then
            echo "✓ Database 'txn_manager' already exists."
        else
            echo "Creating database 'txn_manager'..."
            if psql -h localhost -U postgres -c "CREATE DATABASE txn_manager"; then
                echo "✓ Database created successfully."
            else
                echo "✗ Failed to create database. Check PostgreSQL permissions."
                POSTGRES_RUNNING=false
            fi
        fi
        
        # If database exists/was created, run migrations
        if [ "$POSTGRES_RUNNING" = true ]; then
            echo "Running database migrations..."
            if psql -h localhost -U postgres -d txn_manager -f migrations/20240101000001_initial_schema.sql; then
                echo "✓ Database schema created successfully."
                
                # Prepare SQLx metadata
                echo "Preparing SQLx metadata for offline development..."
                if DATABASE_URL=postgres://postgres:postgres@localhost:5433/txn_manager cargo sqlx prepare; then
                    echo "✓ SQLx metadata prepared successfully."
                    
                    # Create config for offline mode
                    echo '[env]
SQLX_OFFLINE = "true"' > .cargo/config.toml
                    
                    echo
                    echo "Local PostgreSQL setup completed successfully!"
                    echo "You can now run the application with: cargo run"
                    echo
                    echo "Database URL: postgres://postgres:postgres@localhost:5433/txn_manager"
                    exit 0
                else
                    echo "✗ Failed to prepare SQLx metadata."
                fi
            else
                echo "✗ Failed to run migrations."
            fi
        fi
    fi
fi

# If we're here, local PostgreSQL setup failed or is not available

# Try Docker if available
if [ "$DOCKER_AVAILABLE" = true ]; then
    echo
    echo "Attempting to use Docker for PostgreSQL..."
    
    # Check if container already exists
    if docker ps | grep -q "txn-manager-postgres"; then
        echo "✓ PostgreSQL container is already running."
    elif docker ps -a | grep -q "txn-manager-postgres"; then
        echo "Starting existing PostgreSQL container..."
        if docker start txn-manager-postgres; then
            echo "✓ Container started successfully."
        else
            echo "✗ Failed to start container."
            exit 1
        fi
    else
        echo "Creating new PostgreSQL container..."
        if docker run --name txn-manager-postgres -e POSTGRES_PASSWORD=postgres -e POSTGRES_USER=postgres -e POSTGRES_DB=txn_manager -p 5433:5432 -d postgres:16; then
            echo "✓ Container created and started successfully."
            
            # Wait for PostgreSQL to start in the container
            echo "Waiting for PostgreSQL to be ready in the container..."
            sleep 10
        else
            echo "✗ Failed to create container."
            exit 1
        fi
    fi
    
    # Check if PostgreSQL in Docker is running
    if pg_isready -h localhost -p 5433 -d txn_manager -U postgres; then
        echo "✓ PostgreSQL in Docker is running and accessible."
        
        # Prepare SQLx metadata
        echo "Preparing SQLx metadata for offline development..."
        if DATABASE_URL=postgres://postgres:postgres@localhost:5433/txn_manager cargo sqlx prepare; then
            echo "✓ SQLx metadata prepared successfully."
            
            # Create config for offline mode
            echo '[env]
SQLX_OFFLINE = "true"' > .cargo/config.toml
            
            echo
            echo "Docker PostgreSQL setup completed successfully!"
            echo "You can now run the application with: cargo run"
            echo
            echo "Database URL: postgres://postgres:postgres@localhost:5433/txn_manager"
            exit 0
        else
            echo "✗ Failed to prepare SQLx metadata."
        fi
    else
        echo "✗ PostgreSQL in Docker is not accessible."
    fi
fi

# If we reach here, both methods failed
echo
echo "ERROR: Failed to set up the database using both local PostgreSQL and Docker."
echo "Please ensure either PostgreSQL is properly installed and running locally,"
echo "or Docker is properly configured and accessible to your user."
echo
echo "For SQLx offline mode only (if you have SQLx metadata already):"
echo "mkdir -p .cargo && echo '[env]' > .cargo/config.toml && echo 'SQLX_OFFLINE = \"true\"' >> .cargo/config.toml"
exit 1 