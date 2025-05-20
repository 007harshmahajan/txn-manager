use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::{Validate, ValidationError};

use crate::models::decimal::SqlxDecimal;

/// Enum representing the different types of transactions supported by the system
///
/// - TRANSFER: Movement of funds between two accounts within the system
/// - DEPOSIT: External funds coming into an account in the system
/// - WITHDRAWAL: Funds leaving an account to an external destination
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum TransactionType {
    TRANSFER,
    DEPOSIT,
    WITHDRAWAL,
}

impl std::fmt::Display for TransactionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionType::TRANSFER => write!(f, "TRANSFER"),
            TransactionType::DEPOSIT => write!(f, "DEPOSIT"),
            TransactionType::WITHDRAWAL => write!(f, "WITHDRAWAL"),
        }
    }
}

/// Enum representing the possible states of a transaction
///
/// - PENDING: Transaction has been created but not fully processed
/// - COMPLETED: Transaction was successfully processed
/// - FAILED: Transaction processing failed and any partial changes were rolled back
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum TransactionStatus {
    PENDING,
    COMPLETED,
    FAILED,
}

impl std::fmt::Display for TransactionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionStatus::PENDING => write!(f, "PENDING"),
            TransactionStatus::COMPLETED => write!(f, "COMPLETED"),
            TransactionStatus::FAILED => write!(f, "FAILED"),
        }
    }
}

/// The core transaction entity as stored in the database
///
/// This represents a financial transaction in the system with complete metadata.
/// Sender and receiver account IDs are optional depending on transaction type:
/// - TRANSFER: Both sender and receiver are required
/// - DEPOSIT: Only receiver is required (sender is NULL)
/// - WITHDRAWAL: Only sender is required (receiver is NULL)
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Transaction {
    /// Unique identifier for the transaction
    pub id: Uuid,
    /// Account ID of the sender (NULL for deposits)
    pub sender_account_id: Option<Uuid>,
    /// Account ID of the receiver (NULL for withdrawals)
    pub receiver_account_id: Option<Uuid>,
    /// Transaction amount with high precision using our custom decimal type
    pub amount: SqlxDecimal,
    /// Three-letter currency code (e.g., "USD", "EUR")
    pub currency: String,
    /// Type of transaction as a string (TRANSFER, DEPOSIT, WITHDRAWAL)
    pub transaction_type: String,
    /// Current status as a string (PENDING, COMPLETED, FAILED)
    pub status: String,
    /// Optional transaction description or notes
    pub description: Option<String>,
    /// When the transaction was created
    pub created_at: DateTime<Utc>,
    /// When the transaction was last updated
    pub updated_at: DateTime<Utc>,
}

/// Data transfer object for transaction responses
///
/// This is the public-facing representation of a transaction,
/// exposed through the API. It omits updated_at for simplicity.
#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionResponse {
    /// Unique identifier for the transaction
    pub id: Uuid,
    /// Account ID of the sender (NULL for deposits)
    pub sender_account_id: Option<Uuid>,
    /// Account ID of the receiver (NULL for withdrawals)
    pub receiver_account_id: Option<Uuid>,
    /// Transaction amount as a Decimal
    pub amount: Decimal,
    /// Three-letter currency code (e.g., "USD", "EUR")
    pub currency: String,
    /// Type of transaction as a string (TRANSFER, DEPOSIT, WITHDRAWAL)
    pub transaction_type: String,
    /// Current status as a string (PENDING, COMPLETED, FAILED)
    pub status: String,
    /// Optional transaction description or notes
    pub description: Option<String>,
    /// When the transaction was created
    pub created_at: DateTime<Utc>,
}

impl From<Transaction> for TransactionResponse {
    fn from(tx: Transaction) -> Self {
        Self {
            id: tx.id,
            sender_account_id: tx.sender_account_id,
            receiver_account_id: tx.receiver_account_id,
            amount: tx.amount.into(),
            currency: tx.currency,
            transaction_type: tx.transaction_type,
            status: tx.status,
            description: tx.description,
            created_at: tx.created_at,
        }
    }
}

/// Request object for creating a generic transaction
///
/// This is a flexible request format that can represent any type of transaction.
/// Based on the transaction_type, different fields are required.
#[derive(Debug, Deserialize, Serialize, Validate, Clone)]
pub struct CreateTransactionRequest {
    /// Type of transaction as a string: "TRANSFER", "DEPOSIT", or "WITHDRAWAL"
    pub transaction_type: String,

    /// Account ID of the sender (required for TRANSFER and WITHDRAWAL)
    pub sender_account_id: Option<Uuid>,
    /// Account ID of the receiver (required for TRANSFER and DEPOSIT)
    pub receiver_account_id: Option<Uuid>,

    /// Transaction amount (must be positive)
    #[validate(custom = "validate_positive_amount")]
    pub amount: Decimal,

    /// Three-letter currency code
    #[validate(length(min = 3, max = 3, message = "Currency must be a 3-letter code"))]
    pub currency: String,

    /// Optional transaction description or notes
    pub description: Option<String>,
}

/// Request object specifically for transfers between accounts
///
/// Used when explicitly creating a transfer between two accounts.
#[derive(Debug, Deserialize, Serialize, Validate, Clone)]
pub struct TransferRequest {
    /// Account ID to transfer money from
    pub sender_account_id: Uuid,
    /// Account ID to transfer money to
    pub receiver_account_id: Uuid,

    /// Transfer amount (must be positive)
    #[validate(custom = "validate_positive_amount")]
    pub amount: Decimal,

    /// Optional transfer description or notes
    pub description: Option<String>,
}

/// Request object specifically for deposits into an account
///
/// Used when adding funds to an account from an external source.
#[derive(Debug, Deserialize, Serialize, Validate, Clone)]
pub struct DepositRequest {
    /// Account ID to deposit money into
    pub account_id: Uuid,

    /// Deposit amount (must be positive)
    #[validate(custom = "validate_positive_amount")]
    pub amount: Decimal,

    /// Optional deposit description or notes
    pub description: Option<String>,
}

/// Request object specifically for withdrawals from an account
///
/// Used when removing funds from an account to an external destination.
#[derive(Debug, Deserialize, Serialize, Validate, Clone)]
pub struct WithdrawalRequest {
    /// Account ID to withdraw money from
    pub account_id: Uuid,

    /// Withdrawal amount (must be positive)
    #[validate(custom = "validate_positive_amount")]
    pub amount: Decimal,

    /// Optional withdrawal description or notes
    pub description: Option<String>,
}

/// Custom validator function to ensure all transaction amounts are positive
/// 
/// Financial transactions cannot have zero or negative amounts.
/// This validator ensures all amount fields across transaction types
/// have a value greater than zero.
fn validate_positive_amount(amount: &Decimal) -> Result<(), ValidationError> {
    if *amount <= Decimal::ZERO {
        let mut err = ValidationError::new("amount_positive");
        err.message = Some("Amount must be positive".into());
        return Err(err);
    }
    Ok(())
}
