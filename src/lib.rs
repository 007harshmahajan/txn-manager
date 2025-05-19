// Public modules that need to be accessible for integration tests
pub mod api;
pub mod config;
pub mod db;
pub mod middleware;
pub mod models;
pub mod services;
pub mod utils;

// Re-export important types
pub use api::accounts::CreateAccountRequest;
pub use config::Config;
pub use db::init_db_pool;
pub use models::account::{Account, AccountResponse};
pub use models::decimal::SqlxDecimal;
pub use models::transaction::{
    CreateTransactionRequest, DepositRequest, Transaction, TransactionResponse, TransactionStatus,
    TransactionType, TransferRequest, WithdrawalRequest,
};
pub use models::user::{CreateUserRequest, LoginRequest, LoginResponse, User, UserResponse};
pub use services::account_service::AccountService;
pub use services::transaction_service::TransactionService;
pub use services::user_service::UserService;
