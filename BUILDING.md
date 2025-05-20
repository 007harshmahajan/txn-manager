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

## Testing

The Transaction Manager includes a comprehensive test suite covering unit tests, integration tests, and performance tests.

### Unit Tests

Unit tests validate individual components in isolation. Run them with:

```bash
# Run all unit tests
cargo test --lib

# Run unit tests for a specific module
cargo test --lib -- utils::auth
```

### Integration Tests

Integration tests require a running PostgreSQL instance as they interact with a real database. These tests create temporary test databases that are cleaned up after the tests complete.

```bash
# Run all integration tests
cargo test --test integration

# Run a specific integration test
cargo test --test integration -- transaction_tests
```

### Performance Testing

Performance tests help ensure the application maintains acceptable response times under various conditions.

#### Setup for Performance Testing

1. Install the necessary tools:

   ```bash
   # Install hey (HTTP load generator)
   go install github.com/rakyll/hey@latest

   # Alternatively, use Apache Bench (ab) if already installed
   # sudo apt-get install apache2-utils
   ```

2. Start the application in release mode:

   ```bash
   cargo run --release
   ```

3. Run performance tests:

   ```bash
   # Test the health endpoint with 200 requests (50 concurrent)
   hey -n 200 -c 50 http://localhost:8080/

   # Test user registration with 50 requests (10 concurrent)
   hey -n 50 -c 10 -m POST -H "Content-Type: application/json" -d '{"username":"perftest","email":"perf@example.com","password":"securepassword","first_name":"Performance","last_name":"Test"}' http://localhost:8080/api/v1/users/register
   ```

### Load Testing

Load testing helps identify how the system performs under heavy usage.

1. Install k6 (modern load testing tool):

   ```bash
   # On Ubuntu/Debian
   sudo apt-key adv --keyserver hkp://keyserver.ubuntu.com:80 --recv-keys C5AD17C747E3415A3642D57D77C6C491D6AC1D69
   echo "deb https://dl.k6.io/deb stable main" | sudo tee /etc/apt/sources.list.d/k6.list
   sudo apt-get update
   sudo apt-get install k6

   # On macOS
   brew install k6
   ```

2. Create a load test script (e.g., `load-test.js`):

   ```javascript
   import http from 'k6/http';
   import { sleep, check } from 'k6';

   export const options = {
     vus: 100,           // Virtual users
     duration: '30s',    // Test duration
     thresholds: {
       http_req_duration: ['p(95)<500'], // 95% of requests must complete below 500ms
     },
   };

   // Test user creation and login
   export default function () {
     // Generate unique username
     const username = `user_${Date.now()}_${Math.floor(Math.random() * 10000)}`;
     
     // Register a new user
     const registerRes = http.post('http://localhost:8080/api/v1/users/register', JSON.stringify({
       username: username,
       email: `${username}@example.com`,
       password: 'securepassword123',
       first_name: 'Load',
       last_name: 'Test'
     }), {
       headers: { 'Content-Type': 'application/json' },
     });
     
     check(registerRes, {
       'register success': (r) => r.status === 200,
     });
     
     sleep(1);
     
     // Login with created user
     const loginRes = http.post('http://localhost:8080/api/v1/users/login', JSON.stringify({
       username: username,
       password: 'securepassword123',
     }), {
       headers: { 'Content-Type': 'application/json' },
     });
     
     check(loginRes, {
       'login success': (r) => r.status === 200,
     });
     
     sleep(1);
   }
   ```

3. Run the load test:

   ```bash
   k6 run load-test.js
   ```

4. Analyze the results to identify performance bottlenecks and optimize the application.

### Benchmark Testing

For micro-benchmarks of specific functions, use Rust's built-in benchmarking features:

1. Add the benchmark dependency to your `Cargo.toml`:

   ```toml
   [dev-dependencies]
   criterion = "0.3"

   [[bench]]
   name = "transaction_benchmark"
   harness = false
   ```

2. Create benchmark files in a `benches/` directory:

   ```rust
   // benches/transaction_benchmark.rs
   use criterion::{criterion_group, criterion_main, Criterion};
   use txn_manager::models::decimal::SqlxDecimal;
   use rust_decimal::Decimal;
   use std::str::FromStr;

   fn decimal_conversion_benchmark(c: &mut Criterion) {
       c.bench_function("decimal_to_sqlx_decimal", |b| {
           let decimal = Decimal::from_str("123.456").unwrap();
           b.iter(|| SqlxDecimal::from(decimal))
       });
   }

   criterion_group!(benches, decimal_conversion_benchmark);
   criterion_main!(benches);
   ```

3. Run the benchmarks:

   ```bash
   cargo bench
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

## Performance Optimization

The Transaction Manager has undergone performance testing with both individual endpoints (using `hey` or Apache Bench) and holistic load testing (using k6). The detailed results and analysis are available in [PERFORMANCE.md](./PERFORMANCE.md).

### Key Performance Findings

- The core application framework is performant (health endpoint responds in ~1.7ms on average)
- Under load, the system experiences significant slowdowns:
  - User login operations average 644ms
  - The 95th percentile response time is 3.3s (target: 500ms)
  - Registration requests sometimes fail with 500 errors under load

### Recommended Optimizations

Based on the performance test results, several optimizations have been identified:

1. **Database Connection Pooling**: Increase the maximum connections from 5 to 20 and add minimum connection settings:

   ```rust
   let pool = PgPoolOptions::new()
       .max_connections(20)  // Increase from 5
       .min_connections(5)   // Maintain ready connections
       .acquire_timeout(Duration::from_secs(5))
       .idle_timeout(Duration::from_secs(30))
       .connect(database_url)
       .await?;
   ```

2. **Query Optimization**: Add appropriate indices to frequently queried columns, particularly for user authentication flows.

3. **Authentication Performance**: Consider caching authentication results and optimizing the password hashing work factor.

4. **Error Handling**: Add retry logic and improved logging for database operations to reduce 500 errors.

For a complete list of recommendations and implementation details, see the [PERFORMANCE.md](./PERFORMANCE.md) document.

### Running Optimized Builds

For production or performance testing, always use release mode builds:

```bash
cargo build --release
cargo run --release
```

## Troubleshooting

If you continue to encounter issues, please check the logs and error messages for more specific information about what's going wrong.

For problems with SQL queries or database schema, you may need to examine the migration files and ensure they're compatible with your version of PostgreSQL. 