use crate::models::account::{Account, AccountResponse};
use crate::models::decimal::SqlxDecimal;
use crate::utils::error::AppError;
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

/// Service for managing user accounts
/// 
/// This service handles all account-related operations including:
/// - Creating new accounts for users
/// - Retrieving account information
/// - Updating account balances
/// 
/// A core component of the financial system, the AccountService ensures that
/// all balance operations maintain consistency and prevent negative balances.
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

    /// Creates a new account for a user with a specified currency
    ///
    /// # Arguments
    /// * `user_id` - The UUID of the user who will own this account
    /// * `currency` - The three-letter currency code (e.g., "USD", "EUR")
    ///
    /// # Returns
    /// The newly created account wrapped in an AccountResponse
    ///
    /// # Implementation Details
    /// This method:
    /// 1. Verifies the user exists
    /// 2. Creates a new account with zero initial balance
    /// 3. Associates the account with the user
    /// 
    /// New accounts always start with a zero balance. The balance can only
    /// be modified through proper transaction operations.
    pub async fn create_account(
        &self,
        user_id: Uuid,
        currency: String,
    ) -> Result<AccountResponse, AppError> {
        // Check if user exists - we don't want orphaned accounts
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

        // Create account with a new UUID and initial zero balance
        let id = Uuid::new_v4();

        // For SQLx offline mode with type safety, use raw query text
        // This bypasses the SQLx type checking for our custom SqlxDecimal type
        // We explicitly use a raw query to handle the custom decimal type properly
        let query = format!(
            "INSERT INTO accounts (id, user_id, balance, currency) 
             VALUES ('{}', '{}', '0', '{}') 
             RETURNING id, user_id, balance::TEXT, currency, created_at, updated_at",
            id, user_id, currency
        );

        let row = sqlx::query(&query).fetch_one(&self.pool).await?;

        // Extract fields from row using fully qualified syntax
        // This manual construction is needed because we can't use query_as! with a dynamic query
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

    /// Updates an account's balance by adding or subtracting the specified amount
    ///
    /// # Arguments
    /// * `id` - The UUID of the account to update
    /// * `amount` - The amount to add to the balance (use negative for subtraction)
    ///
    /// # Returns
    /// The updated account with the new balance
    ///
    /// # Implementation Details
    /// This method:
    /// 1. Begins a database transaction for atomicity
    /// 2. Locks the account row to prevent concurrent modifications
    /// 3. Retrieves the current balance
    /// 4. Calculates the new balance and ensures it won't be negative
    /// 5. Updates the account with the new balance
    /// 6. Commits the transaction
    ///
    /// # Financial Safety Measures
    /// - Uses a database transaction for atomicity
    /// - Locks the row with FOR UPDATE to prevent race conditions
    /// - Performs explicit negative balance check
    /// - Additionally, the database schema has a CHECK constraint for non-negative balances
    pub async fn update_balance(
        &self,
        id: Uuid,
        amount: Decimal,
    ) -> Result<AccountResponse, AppError> {
        // Use a database transaction to ensure atomicity and consistency
        // This is crucial for financial operations to prevent partial updates
        let mut tx = self.pool.begin().await?;

        // Get current account with an exclusive lock (FOR UPDATE)
        // This prevents concurrent updates to the same account, avoiding race conditions
        // that could lead to inconsistencies like double-spending or incorrect balances
        let query = format!(
            "SELECT id, user_id, balance::TEXT, currency, created_at, updated_at 
             FROM accounts WHERE id = '{}' FOR UPDATE",
            id
        );

        let row_option = sqlx::query(&query).fetch_optional(&mut *tx).await?;

        // Verify account exists
        let row = row_option
            .ok_or_else(|| AppError::NotFound(format!("Account with ID {} not found", id)))?;

        // Extract current balance as Decimal for precise calculation
        // We parse from text to maintain full decimal precision
        let current_balance: Decimal = sqlx::Row::get::<&str, _>(&row, "balance")
            .parse()
            .unwrap_or(Decimal::ZERO);

        // Calculate new balance - the core financial operation
        let new_balance = current_balance + amount;

        // Explicit check to ensure balance won't go negative
        // This is a critical financial safeguard
        if new_balance < Decimal::ZERO {
            return Err(AppError::BadRequest("Insufficient funds".to_string()));
        }

        // Update balance using a raw query 
        // We use string formatting for the balance to maintain precision
        let update_query = format!(
            "UPDATE accounts 
             SET balance = '{}', updated_at = NOW() 
             WHERE id = '{}' 
             RETURNING id, user_id, balance::TEXT, currency, created_at, updated_at",
            new_balance.to_string(),
            id
        );

        let updated_row = sqlx::query(&update_query).fetch_one(&mut *tx).await?;

        // Manually create the Account struct with updated balance
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

        // Commit the transaction to make the balance update permanent
        // If any error occurred before this point, the transaction would be rolled back
        tx.commit().await?;

        // Return the updated account information
        Ok(AccountResponse::from(updated_account))
    }
}
