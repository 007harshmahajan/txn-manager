# Transaction Manager Setup Instructions

This document provides detailed instructions for setting up and running the Transaction Manager application.

## Prerequisites

- Git
- One of the following setup methods:
  - Docker and Docker Compose (recommended, easiest)
  - Rust 1.75+ and PostgreSQL (for direct installation)

## Option 1: Quick Setup with Docker (Recommended)

This is the easiest way to get started with minimal dependencies.

### Step 1: Clone the repository

```bash
git clone https://github.com/yourusername/txn-manager.git
cd txn-manager
```

### Step 2: Run the setup script

```bash
./setup.sh
```

That's it! The setup script will:
- Check for required dependencies
- Create a secure configuration
- Build and start all services
- Provide connection information

The application will be available at http://localhost:8080

To stop the application:
```bash
docker-compose down
```

## Option 2: Manual Docker Setup

If you prefer to run the Docker commands manually:

```bash
# Clone the repository
git clone https://github.com/yourusername/txn-manager.git
cd txn-manager

# Create .env file
cp .env.example .env

# Build and start the services
docker-compose build
docker-compose up -d
```

## Option 3: Direct Installation (Without Docker)

If you prefer to run the application directly on your system:

### Step 1: Install PostgreSQL

#### Ubuntu/Debian
```bash
sudo apt update
sudo apt install postgresql postgresql-contrib
sudo systemctl start postgresql
sudo systemctl enable postgresql
```

#### macOS (using Homebrew)
```bash
brew install postgresql
brew services start postgresql
```

### Step 2: Create a database

```bash
sudo -u postgres psql -c "CREATE USER txn_manager_user WITH PASSWORD 'your_password';"
sudo -u postgres psql -c "CREATE DATABASE txn_manager OWNER txn_manager_user;"
```

### Step 3: Set up the application

```bash
# Clone the repository
git clone https://github.com/yourusername/txn-manager.git
cd txn-manager

# Create .env file
cp .env.example .env

# Edit the .env file with your database credentials
DATABASE_URL=postgres://username:password@localhost:5433/txn_manager

# Run migrations
cargo install sqlx-cli
sqlx migrate run

# Build and run the application
cargo build --release
./target/release/txn-manager
```

## Testing the Installation

Once the application is running, you can test it with a simple HTTP request:

```bash
curl http://localhost:8080/
```

You should receive an "OK" response.

To test the API endpoints, you can use the following curl commands:

```bash
# Register a new user
curl -X POST http://localhost:8080/api/v1/users/register \
  -H "Content-Type: application/json" \
  -d '{"username":"testuser","email":"test@example.com","password":"securepassword","first_name":"Test","last_name":"User"}'

# Login
curl -X POST http://localhost:8080/api/v1/users/login \
  -H "Content-Type: application/json" \
  -d '{"username":"testuser","password":"securepassword"}'
```

## Common Issues and Troubleshooting

### Docker Compose Python Error

If you encounter an error about Python dependencies when running `docker-compose`:

```
ImportError: cannot import name 'Mapping' from 'collections'
```

Try one of these solutions:

1. Use the newer Docker Compose V2 syntax:
   ```bash
   docker compose up -d  # Note: no hyphen between docker and compose
   ```

2. Fix Python dependencies:
   ```bash
   pip uninstall -y urllib3 requests
   pip install --user docker-compose
   ```

### Database Connection Issues

If the application fails to connect to the database:

1. Check if PostgreSQL is running:
   ```bash
   sudo systemctl status postgresql  # Linux
   brew services info postgresql     # macOS
   ```

2. Verify your connection string in `.env`:
   ```
   DATABASE_URL=postgres://username:password@localhost:5433/txn_manager
   ```

3. Ensure PostgreSQL is accepting connections:
   ```bash
   sudo -u postgres psql -c "ALTER SYSTEM SET listen_addresses TO '*';"
   sudo -u postgres psql -c "ALTER SYSTEM SET max_connections TO 100;"
   sudo systemctl restart postgresql
   ```

### Permission Issues with Docker

If you encounter permission issues with Docker:

```bash
sudo usermod -aG docker $USER
# Log out and log back in for changes to take effect
```

## Updating the Application

To update to the latest version:

```bash
# With Docker
git pull
docker-compose down
docker-compose build
docker-compose up -d

# Without Docker
git pull
cargo build --release
# Restart the application
```

## Additional Resources

- [API Documentation](./API_DOCUMENTATION.md)
- [Performance Information](./PERFORMANCE.md)
- [Building from Source](./BUILDING.md) 