#[cfg(test)]
pub mod test_utils {
    use crate::models::account::Account;
    use crate::models::decimal::SqlxDecimal;
    use crate::models::transaction::Transaction;
    use crate::models::user::User;
    use chrono::{DateTime, Utc};
    use mockall::predicate::*;
    use mockall::*;
    use rust_decimal::Decimal;
    use sqlx::{Error as SqlxError, Pool, Postgres, Row};
    use std::str::FromStr;
    use uuid::Uuid;

    mock! {
        pub PgPool {}
        impl Clone for PgPool {
            fn clone(&self) -> Self;
        }
    }

    /// Creates a test user with predefined values
    pub fn create_test_user() -> User {
        User {
            id: Uuid::new_v4(),
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password_hash: "$2y$12$...".to_string(), // Dummy hash
            first_name: Some("Test".to_string()),
            last_name: Some("User".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// Creates a test account with the given balance
    pub fn create_test_account(user_id: Uuid, balance: Decimal, currency: &str) -> Account {
        Account {
            id: Uuid::new_v4(),
            user_id,
            balance: SqlxDecimal(balance),
            currency: currency.to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// Creates a test transaction
    pub fn create_test_transaction(
        sender_id: Option<Uuid>,
        receiver_id: Option<Uuid>,
        amount: Decimal,
        transaction_type: &str,
        status: &str,
    ) -> Transaction {
        Transaction {
            id: Uuid::new_v4(),
            sender_account_id: sender_id,
            receiver_account_id: receiver_id,
            amount: SqlxDecimal(amount),
            currency: "USD".to_string(),
            transaction_type: transaction_type.to_string(),
            status: status.to_string(),
            description: Some("Test transaction".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
} 