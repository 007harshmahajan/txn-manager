mod api;
mod config;
mod db;
mod middleware;
mod models;
mod services;
mod utils;

use crate::api::{accounts, transactions, users};
use crate::config::Config;
use crate::db::init_db_pool;
use crate::middleware::auth::auth_middleware;
use crate::services::{
    account_service::AccountService, transaction_service::TransactionService,
    user_service::UserService,
};
use axum::{middleware::from_fn_with_state, routing::get, Router};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load configuration
    let config = Config::from_env();

    // Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Initialize database
    let pool_result = init_db_pool(&config.database_url).await;

    let pool = match pool_result {
        Ok(pool) => {
            tracing::info!("Database connection established successfully");
            pool
        }
        Err(err) => {
            if cfg!(debug_assertions) {
                tracing::warn!(
                    "Database connection failed: {}. Some features may not work.",
                    err
                );
                tracing::warn!("Starting server without database connection for development");
                tracing::warn!("Please set up a database for full functionality");
                return Err(err);
            } else {
                return Err(err);
            }
        }
    };

    // Initialize services
    let user_service = Arc::new(UserService::new(pool.clone(), config.jwt_secret.clone()));
    let account_service = Arc::new(AccountService::new(pool.clone()));
    let transaction_service = Arc::new(TransactionService::new(
        pool.clone(),
        AccountService::new(pool.clone()),
    ));

    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Create router
    let app = Router::new()
        .route("/", get(health_check))
        .nest("/api/v1/users", users::user_routes(user_service.clone()))
        .nest(
            "/api/v1/accounts",
            accounts::account_routes(account_service.clone()).route_layer(from_fn_with_state(
                config.jwt_secret.clone(),
                auth_middleware,
            )),
        )
        .nest(
            "/api/v1/transactions",
            transactions::transaction_routes(transaction_service.clone(), account_service.clone())
                .route_layer(from_fn_with_state(
                    config.jwt_secret.clone(),
                    auth_middleware,
                )),
        )
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .layer(RequestBodyLimitLayer::new(1024 * 1024)); // 1MB limit

    // Start server
    let addr = config.server_addr();
    tracing::info!("Starting server on {}", addr);

    // Bind to the address and serve the app
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> &'static str {
    "OK"
}
