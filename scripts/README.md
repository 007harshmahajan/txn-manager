# Scripts Directory

This directory contains utility scripts for setting up, managing, and testing the Transaction Manager application.

## Main Scripts

- **setup.sh**: Primary setup script for quick deployment (also available in root directory)
- **setup_database.sh**: Sets up the database using Docker
- **setup_local_database.sh**: Sets up the database using a local PostgreSQL installation
- **setup_sqlx_offline.sh**: Configures SQLx for offline development

## SQLx Scripts

- **prepare_for_sqlx.sh**: Prepares the environment for SQLx
- **prepare_sqlx.sh**: Generates SQLx metadata for offline mode

## Testing Scripts

- **run_performance_tests.sh**: Runs performance tests
- **track_performance.sh**: Tracks performance metrics over time
- **load-test.js**: JavaScript file for k6 load testing

## Usage

Most scripts can be run directly from this directory:

```bash
cd scripts
./setup_database.sh
```

However, the main setup script should be run from the project root:

```bash
cd ..
./setup.sh
``` 