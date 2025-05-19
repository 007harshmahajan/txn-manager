use crate::models::account::{Account, AccountResponse};
use crate::models::decimal::SqlxDecimal;
use crate::utils::error::AppError;
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

/// Service for managing user accounts
pub struct AccountService {
    pool: PgPool,
}

impl AccountService {
    /// Creates a new account service with the given database pool
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Fetches an account by its ID
    ///
    /// # Arguments
    /// * `id` - The UUID of the account to retrieve
    ///
    /// # Returns
    /// The account details wrapped in an AccountResponse if found
    pub async fn get_account_by_id(&self, id: Uuid) -> Result<AccountResponse, AppError> {
        let account = sqlx::query_as!(
            Account,
            r#"
            SELECT id, user_id, balance as "balance: SqlxDecimal", currency, created_at, updated_at
            FROM accounts WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Account with ID {} not found", id)))?;

        Ok(AccountResponse::from(account))
    }

    /// Retrieves all accounts for a user
    ///
    /// # Arguments
    /// * `user_id` - The UUID of the user whose accounts should be retrieved
    ///
    /// # Returns
    /// A vector of account responses
    pub async fn get_accounts_by_user_id(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<AccountResponse>, AppError> {
        let accounts = sqlx::query_as!(
            Account,
            r#"
            SELECT id, user_id, balance as "balance: SqlxDecimal", currency, created_at, updated_at
            FROM accounts WHERE user_id = $1
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(accounts.into_iter().map(AccountResponse::from).collect())
    }

    pub async fn create_account(
        &self,
        user_id: Uuid,
        currency: String,
    ) -> Result<AccountResponse, AppError> {
        // Check if user exists
        let user_exists = sqlx::query!(
            r#"
            SELECT id FROM users WHERE id = $1
            "#,
            user_id
        )
        .fetch_optional(&self.pool)
        .await?;

        if user_exists.is_none() {
            return Err(AppError::NotFound(format!(
                "User with ID {} not found",
                user_id
            )));
        }

        // Create account
        let id = Uuid::new_v4();

        // For SQLx offline mode with type safety, use raw query text
        // This bypasses the SQLx type checking for our custom SqlxDecimal type
        let query = format!(
            "INSERT INTO accounts (id, user_id, balance, currency) 
             VALUES ('{}', '{}', '0', '{}') 
             RETURNING id, user_id, balance::TEXT, currency, created_at, updated_at",
            id, user_id, currency
        );

        let row = sqlx::query(&query).fetch_one(&self.pool).await?;

        // Extract fields from row using fully qualified syntax
        let account = Account {
            id: sqlx::Row::get(&row, "id"),
            user_id: sqlx::Row::get(&row, "user_id"),
            balance: SqlxDecimal(
                sqlx::Row::get::<&str, _>(&row, "balance")
                    .parse()
                    .unwrap_or(Decimal::ZERO),
            ),
            currency: sqlx::Row::get(&row, "currency"),
            created_at: sqlx::Row::get(&row, "created_at"),
            updated_at: sqlx::Row::get(&row, "updated_at"),
        };

        Ok(AccountResponse::from(account))
    }

    pub async fn update_balance(
        &self,
        id: Uuid,
        amount: Decimal,
    ) -> Result<AccountResponse, AppError> {
        // Use a transaction to update balance
        let mut tx = self.pool.begin().await?;

        // Get current account - use raw query to get balance as text
        let query = format!(
            "SELECT id, user_id, balance::TEXT, currency, created_at, updated_at 
             FROM accounts WHERE id = '{}' FOR UPDATE",
            id
        );

        let row_option = sqlx::query(&query).fetch_optional(&mut *tx).await?;

        let row = row_option
            .ok_or_else(|| AppError::NotFound(format!("Account with ID {} not found", id)))?;

        // Extract current balance and convert to Decimal
        let current_balance: Decimal = sqlx::Row::get::<&str, _>(&row, "balance")
            .parse()
            .unwrap_or(Decimal::ZERO);

        // Calculate new balance
        let new_balance = current_balance + amount;

        // Ensure balance is not negative
        if new_balance < Decimal::ZERO {
            return Err(AppError::BadRequest("Insufficient funds".to_string()));
        }

        // Update balance using a raw query
        let update_query = format!(
            "UPDATE accounts 
             SET balance = '{}', updated_at = NOW() 
             WHERE id = '{}' 
             RETURNING id, user_id, balance::TEXT, currency, created_at, updated_at",
            new_balance.to_string(),
            id
        );

        let updated_row = sqlx::query(&update_query).fetch_one(&mut *tx).await?;

        // Manually create the Account struct
        let updated_account = Account {
            id: sqlx::Row::get(&updated_row, "id"),
            user_id: sqlx::Row::get(&updated_row, "user_id"),
            balance: SqlxDecimal(
                sqlx::Row::get::<&str, _>(&updated_row, "balance")
                    .parse()
                    .unwrap_or(Decimal::ZERO),
            ),
            currency: sqlx::Row::get(&updated_row, "currency"),
            created_at: sqlx::Row::get(&updated_row, "created_at"),
            updated_at: sqlx::Row::get(&updated_row, "updated_at"),
        };

        // Commit transaction
        tx.commit().await?;

        Ok(AccountResponse::from(updated_account))
    }
}
