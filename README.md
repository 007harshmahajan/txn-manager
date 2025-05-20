# Transaction Manager

A backend service for managing transactions and user accounts in a financial system.

## Features

- User Management: Registration, authentication, and profile management
- Transaction Management: Create, view, and list transactions
- Account Management: Balance tracking, multiple accounts per user
- JWT Authentication: Secure API access
- PostgreSQL Database: Reliable data storage

## Tech Stack

- [Rust](https://www.rust-lang.org/): Primary programming language
- [Axum](https://github.com/tokio-rs/axum): Web framework
- [SQLx](https://github.com/launchbadge/sqlx): Database interactions
- [PostgreSQL](https://www.postgresql.org/): Relational database
- [Docker](https://www.docker.com/): Containerization

## Database Schema

The system uses PostgreSQL and consists of three main tables:

### Users Table

Stores user information and authentication details.

```sql
CREATE TABLE users (
    id UUID PRIMARY KEY,
    username VARCHAR(50) NOT NULL UNIQUE,
    email VARCHAR(100) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    first_name VARCHAR(50),
    last_name VARCHAR(50),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);
```

### Accounts Table

Stores account information, with each account belonging to a user.

```sql
CREATE TABLE accounts (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    balance DECIMAL(19, 4) NOT NULL DEFAULT 0.0,
    currency VARCHAR(3) NOT NULL DEFAULT 'USD',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    CONSTRAINT balance_non_negative CHECK (balance >= 0)
);

CREATE INDEX idx_accounts_user ON accounts(user_id);
```

### Transactions Table

Records all financial transactions between accounts.

```sql
CREATE TABLE transactions (
    id UUID PRIMARY KEY,
    sender_account_id UUID REFERENCES accounts(id),
    receiver_account_id UUID REFERENCES accounts(id),
    amount DECIMAL(19, 4) NOT NULL,
    currency VARCHAR(3) NOT NULL,
    transaction_type VARCHAR(10) NOT NULL CHECK (transaction_type IN ('TRANSFER', 'DEPOSIT', 'WITHDRAWAL')),
    status VARCHAR(10) NOT NULL DEFAULT 'PENDING' CHECK (status IN ('PENDING', 'COMPLETED', 'FAILED')),
    description TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    CONSTRAINT amount_positive CHECK (amount > 0),
    CONSTRAINT transaction_not_self CHECK (
        (transaction_type = 'TRANSFER' AND sender_account_id IS NOT NULL AND receiver_account_id IS NOT NULL AND sender_account_id != receiver_account_id) OR
        (transaction_type = 'DEPOSIT' AND sender_account_id IS NULL AND receiver_account_id IS NOT NULL) OR
        (transaction_type = 'WITHDRAWAL' AND sender_account_id IS NOT NULL AND receiver_account_id IS NULL)
    )
);

CREATE INDEX idx_transactions_sender ON transactions(sender_account_id);
CREATE INDEX idx_transactions_receiver ON transactions(receiver_account_id);
```

### Entity Relationship Diagram

```
┌─────────────┐       ┌─────────────┐       ┌─────────────┐
│   Users     │       │  Accounts   │       │Transactions │
├─────────────┤       ├─────────────┤       ├─────────────┤
│ id          │       │ id          │       │ id          │
│ username    │       │ user_id     │─────┐ │ sender_id   │─┐
│ email       │       │ balance     │     │ │ receiver_id │ │
│ password    │  1:N  │ currency    │  1:N│ │ amount      │ │
│ first_name  ├───────┤ created_at  │     └─┤ currency    │ │
│ last_name   │       │ updated_at  │       │ type        │ │
│ created_at  │       └─────────────┘       │ status      │ │
│ updated_at  │                             │ description │ │
└─────────────┘                             │ created_at  │ │
                                            │ updated_at  │ │
                                            └──────┬──────┘ │
                                                   │        │
                                                   └────────┘
```

## Project Structure

```
txn-manager/
├── src/                 # Source code
│   ├── api/             # API route handlers
│   ├── config/          # Configuration management
│   ├── db/              # Database connection setup
│   ├── middleware/      # Axum middleware (auth, etc.)
│   ├── models/          # Data models
│   ├── services/        # Business logic
│   └── utils/           # Shared utilities
├── migrations/          # Database migrations
├── tests/               # Integration tests
├── API_DOCUMENTATION.md # Detailed API documentation
└── README.md            # This file
```

## Documentation

- For detailed API documentation, refer to [API_DOCUMENTATION.md](./API_DOCUMENTATION.md)
- For build instructions, see [BUILDING.md](./BUILDING.md)
- For performance optimization, see [PERFORMANCE.md](./PERFORMANCE.md)

## Setup and Installation

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (1.75 or later)
- [PostgreSQL](https://www.postgresql.org/download/) (14 or later)
- [Docker](https://docs.docker.com/get-docker/) (optional, for containerized deployment)

### Quick Start with Docker

The fastest way to get started is with Docker:

```bash
# Clone the repository
git clone https://github.com/yourusername/txn-manager.git
cd txn-manager

# Start with Docker Compose
docker-compose up -d
```

The API will be available at http://localhost:8080

### Local Development Setup

1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/txn-manager.git
   cd txn-manager
   ```

2. Create a `.env` file based on the example:
   ```bash
   cp .env.example .env
   # Edit the .env file with your configuration
   ```

3. Set up the database:
   ```bash
   # Option 1: Using the combined setup script (recommended)
   ./setup_app.sh
   
   # Option 2: Setup with local PostgreSQL
   ./setup_local_database.sh
   
   # Option 3: Setup with Docker
   ./setup_database.sh
   
   # Option 4: SQLx offline mode only (no database)
   ./setup_sqlx_offline.sh
   ```

4. Build and run the application:
   ```bash
   cargo run
   ```

   The API will be available at http://localhost:8080

## Troubleshooting

For common issues and their solutions, see [BUILDING.md](./BUILDING.md).

## Running Tests

```bash
# Run unit tests
cargo test

# Run integration tests (requires PostgreSQL)
cargo test --test integration -- --ignored
```

## Performance Testing

The Transaction Manager includes comprehensive performance testing tools:

```bash
# Run performance tests with the built-in script
./run_performance_tests.sh

# Track performance improvements over time
./track_performance.sh
```

Performance test results and optimization recommendations are documented in [PERFORMANCE.md](./PERFORMANCE.md).

Key performance metrics:
- Health endpoint: ~1.7ms average response time
- User operations: Optimized for concurrent usage with proper connection pooling
- Database queries: Efficiently structured for minimal latency

For production deployments, always use release builds:
```bash
cargo run --release
```

## Performance Considerations

The service is designed to handle multiple concurrent users efficiently:

- Connection pooling for database access
- Async/await for non-blocking operations
- Proper error handling to prevent resource leaks
- Optimized database queries with appropriate indices
- Regular performance testing and optimization

## License

This project is licensed under the MIT License - see the LICENSE file for details.
