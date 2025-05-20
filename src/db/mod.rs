use anyhow::Result;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::time::Duration;
use sqlx::Postgres;
use sqlx::migrate::MigrateDatabase;

#[cfg(not(debug_assertions))]
pub async fn init_db_pool(database_url: &str) -> Result<PgPool> {
    // Create database if it doesn't exist
    if !Postgres::database_exists(database_url).await? {
        Postgres::create_database(database_url).await?;
    }

    // Connect to the database with optimized connection pool settings
    let pool = PgPoolOptions::new()
        .max_connections(20)        // Increased from 5 for better concurrency
        .min_connections(5)         // Keep a minimum pool of connections ready
        .acquire_timeout(Duration::from_secs(5))
        .idle_timeout(Duration::from_secs(30))  // Release idle connections
        .max_lifetime(Duration::from_secs(1800)) // 30-minute max lifetime
        .connect(database_url)
        .await?;

    // Run migrations
    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}

#[cfg(debug_assertions)]
pub async fn init_db_pool(database_url: &str) -> Result<PgPool> {
    // Try to connect to the database with a short timeout
    // In debug mode, we use less aggressive pooling
    let connect_result = PgPoolOptions::new()
        .max_connections(10)        // Increased from 5, but still modest for dev
        .min_connections(2)         // Maintain a small pool for development
        .acquire_timeout(Duration::from_secs(3))
        .connect(database_url)
        .await;

    match connect_result {
        Ok(pool) => {
            // Run migrations if connected successfully
            let _ = sqlx::migrate!("./migrations").run(&pool).await;
            Ok(pool)
        }
        Err(err) => {
            // In debug mode, provide helpful information but continue
            eprintln!("Database connection failed: {}", err);
            eprintln!("Using the application might lead to runtime errors.");
            eprintln!("Please set up your database using:");
            eprintln!("  - For local PostgreSQL: ./setup_local_database.sh");
            eprintln!("  - For Docker: sudo ./setup_database.sh");

            // Return error to be handled at the call site
            Err(err.into())
        }
    }
}
