# Building the  Transaction Manager

This document provides detailed instructions for building and running the transaction manager service.

## Prerequisites

- Rust and Cargo (1.75 or newer)
- PostgreSQL (for running the application)
- Docker (optional, for containerized database)

## Building the Application

### 1. Clone the repository

```bash
git clone https://github.com/yourusername/txn-manager.git
cd txn-manager
```

### 2. Build the application

```bash
cargo build
```

If you encounter issues with the SQLx dependency during build time, it's likely because SQLx attempts to validate your SQL queries against an actual database during compilation. We've configured the project to use offline mode by default, but if you still have issues, try:

```bash
SQLX_OFFLINE=true cargo build
```

## Important Changes to Note

We've implemented custom type handling for rust_decimal::Decimal to work with SQLx. This allows us to avoid using the "decimal" feature which isn't available in the current version of SQLx. Instead, we implement the necessary traits manually in `src/models/transaction.rs`.

## Setting Up the Database

### Option 1: Using Docker (Recommended)

This is the easiest way to get started. It will set up PostgreSQL and prepare SQLx for offline mode.

```bash
sudo ./setup_database.sh
```

If you encounter permission issues with Docker, you can add your user to the Docker group:

```bash
sudo usermod -aG docker $USER
# Then log out and log back in
```

### Option 2: Using a Local PostgreSQL Installation

Make sure PostgreSQL is installed and running on your system, then:

```bash
./setup_local_database.sh
```

## Running the Application

### 1. Set up environment variables

```bash
cp .env.example .env
# Edit .env as needed
```

### 2. Run the application

```bash
cargo run
```

## SQLx Offline Mode

This project uses SQLx's offline mode to allow building without a database connection. This is set up automatically when you run the setup_database.sh script. 

If you need to manually prepare SQLx for offline mode:

1. Make sure your database is running and accessible
2. Install SQLx CLI if you don't have it:
   ```bash
   cargo install sqlx-cli --no-default-features --features postgres
   ```
3. Prepare the metadata:
   ```bash
   DATABASE_URL=postgres://postgres:postgres@localhost:5432/txn_manager cargo sqlx prepare --merged
   ```
4. Enable offline mode by creating `.cargo/config.toml`:
   ```toml
   [env]
   SQLX_OFFLINE = "true"
   SQLX_OFFLINE_DIR = ".sqlx"
   RUSTFLAGS = "--cfg tokio_unstable --cfg sqlx_macros_unstable"
   ```

## Common Issues and Solutions

### 1. SQLx Feature Error

If you see an error like:
```
error: failed to select a version for `sqlx`... the package `txn-manager` depends on `sqlx`, with features: `...` but `sqlx` does not have these features.
```

This means that the feature you're trying to use (like "decimal") isn't available in the current version of SQLx. We've addressed this by:

1. Removing the problematic feature from Cargo.toml
2. Implementing custom type conversion for Decimal in the models

If you're still having issues, check that your Cargo.toml has:
```toml
sqlx = { version = "0.7.3", features = ["runtime-tokio-native-tls", "postgres", "uuid", "chrono", "json", "migrate"] }
```

And NOT:
```toml
sqlx = { version = "0.7.3", features = ["runtime-tokio-native-tls", "postgres", "uuid", "chrono", "json", "migrate", "decimal"] }
```

### 2. Database Connection Error

If you encounter errors related to database connection during build:

- Make sure PostgreSQL is installed and running
- Verify the connection details in your `.env` file
- Make sure SQLx offline mode is enabled (SQLX_OFFLINE=true)
- Try running one of the database setup scripts

### 3. Docker Permission Issues

If you get "permission denied" errors when running Docker commands:

```bash
sudo ./setup_database.sh  # Run with sudo
```

Or add your user to the Docker group:

```bash
sudo usermod -aG docker $USER
# Then log out and log back in
```

### 4. PostgreSQL Authentication Issues

If PostgreSQL rejects your connection with authentication errors, review the credentials in your `.env` file and ensure they match with what was set up in the database.

## Troubleshooting

If you continue to encounter issues, please check the logs and error messages for more specific information about what's going wrong.

For problems with SQL queries or database schema, you may need to examine the migration files and ensure they're compatible with your version of PostgreSQL. 