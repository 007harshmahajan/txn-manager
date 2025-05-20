use dotenv::dotenv;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::sync::Arc;
use std::sync::Once;
use uuid::Uuid;

// Import from the crate root
use txn_manager::{AccountService, TransactionService, UserService};

static INIT: Once = Once::new();

/// Sets up a test database with a unique name for isolation
pub async fn setup() -> (PgPool, String) {
    INIT.call_once(|| {
        dotenv().ok();
    });

    // Create a unique database name for this test run
    let db_name = format!("test_db_{}", Uuid::new_v4().to_string().replace("-", ""));

    // Connect to the default postgres database to create our test database
    let admin_url = "postgres://postgres:postgres@localhost:5433/postgres";
    let admin_pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(admin_url)
        .await
        .expect("Failed to connect to postgres database");

    // Create the test database
    sqlx::query(&format!("CREATE DATABASE {}", db_name))
        .execute(&admin_pool)
        .await
        .expect("Failed to create test database");

    // Connect to the new test database
    let db_url = format!("postgres://postgres:postgres@localhost:5433/{}", db_name);
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("Failed to connect to test database");

    // Run migrations to set up the schema
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    (pool, db_url)
}

/// Creates a user service for testing
pub fn create_user_service(pool: PgPool) -> Arc<UserService> {
    Arc::new(UserService::new(pool, "test_secret".to_string()))
}

/// Creates an account service for testing
pub fn create_account_service(pool: PgPool) -> Arc<AccountService> {
    Arc::new(AccountService::new(pool))
}

/// Creates a transaction service for testing
pub fn create_transaction_service(pool: PgPool) -> Arc<TransactionService> {
    // Create account service first as it's needed by transaction service
    let account_service = AccountService::new(pool.clone());
    Arc::new(TransactionService::new(pool, account_service))
}

/// Tears down the test database
pub async fn teardown(db_url: &str) {
    // Extract database name from URL
    let db_name = db_url.split('/').last().unwrap();

    // Connect to the default postgres database to drop our test database
    let admin_url = "postgres://postgres:postgres@localhost:5433/postgres";
    let admin_pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(admin_url)
        .await
        .expect("Failed to connect to postgres database");

    // Terminate all connections to the test database
    sqlx::query(&format!(
        "SELECT pg_terminate_backend(pg_stat_activity.pid) 
         FROM pg_stat_activity 
         WHERE pg_stat_activity.datname = '{}'
         AND pid <> pg_backend_pid()",
        db_name
    ))
    .execute(&admin_pool)
    .await
    .expect("Failed to terminate connections to test database");

    // Drop the test database
    sqlx::query(&format!("DROP DATABASE {}", db_name))
        .execute(&admin_pool)
        .await
        .expect("Failed to drop test database");
}
