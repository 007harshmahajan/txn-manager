use anyhow::Result;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::time::Duration;

#[cfg(not(debug_assertions))]
pub async fn init_db_pool(database_url: &str) -> Result<PgPool> {
    // Create database if it doesn't exist
    if !Postgres::database_exists(database_url).await? {
        Postgres::create_database(database_url).await?;
    }

    // Connect to the database
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(database_url)
        .await?;

    // Run migrations
    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}

#[cfg(debug_assertions)]
pub async fn init_db_pool(database_url: &str) -> Result<PgPool> {
    // Try to connect to the database with a short timeout
    let connect_result = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(1))
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
