use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::{Validate, ValidationError};

use crate::models::decimal::SqlxDecimal;

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

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Transaction {
    pub id: Uuid,
    pub sender_account_id: Option<Uuid>,
    pub receiver_account_id: Option<Uuid>,
    pub amount: SqlxDecimal,
    pub currency: String,
    pub transaction_type: String,
    pub status: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionResponse {
    pub id: Uuid,
    pub sender_account_id: Option<Uuid>,
    pub receiver_account_id: Option<Uuid>,
    pub amount: Decimal,
    pub currency: String,
    pub transaction_type: String,
    pub status: String,
    pub description: Option<String>,
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

#[derive(Debug, Deserialize, Serialize, Validate, Clone)]
pub struct CreateTransactionRequest {
    pub transaction_type: String,

    pub sender_account_id: Option<Uuid>,
    pub receiver_account_id: Option<Uuid>,

    #[validate(custom = "validate_positive_amount")]
    pub amount: Decimal,

    #[validate(length(min = 3, max = 3, message = "Currency must be a 3-letter code"))]
    pub currency: String,

    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Validate, Clone)]
pub struct TransferRequest {
    pub sender_account_id: Uuid,
    pub receiver_account_id: Uuid,

    #[validate(custom = "validate_positive_amount")]
    pub amount: Decimal,

    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Validate, Clone)]
pub struct DepositRequest {
    pub account_id: Uuid,

    #[validate(custom = "validate_positive_amount")]
    pub amount: Decimal,

    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Validate, Clone)]
pub struct WithdrawalRequest {
    pub account_id: Uuid,

    #[validate(custom = "validate_positive_amount")]
    pub amount: Decimal,

    pub description: Option<String>,
}

// Custom validator function
fn validate_positive_amount(amount: &Decimal) -> Result<(), ValidationError> {
    if *amount <= Decimal::ZERO {
        let mut err = ValidationError::new("amount_positive");
        err.message = Some("Amount must be positive".into());
        return Err(err);
    }
    Ok(())
}
