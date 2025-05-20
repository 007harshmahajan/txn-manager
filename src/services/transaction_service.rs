use crate::models::decimal::SqlxDecimal;
use crate::models::transaction::{
    CreateTransactionRequest, DepositRequest, Transaction, TransactionResponse, TransactionStatus,
    TransactionType, TransferRequest, WithdrawalRequest,
};
use crate::services::account_service::AccountService;
use crate::utils::error::AppError;
use rust_decimal::Decimal;
use sqlx::{PgPool, Postgres, Transaction as SqlxTransaction};
use uuid::Uuid;

/// Service for managing transactions between accounts
/// 
/// This service handles all financial transactions including:
/// - Transfers between accounts (requires sender and receiver)
/// - Deposits into accounts (external funds into the system)
/// - Withdrawals from accounts (funds leaving the system)
/// 
/// All operations use database transactions to ensure data consistency
/// and prevent race conditions or partial updates.
pub struct TransactionService {
    pool: PgPool,
    /// Account service for account-related operations
    pub account_service: AccountService,
}

impl TransactionService {
    /// Creates a new transaction service with the given database pool and account service
    pub fn new(pool: PgPool, account_service: AccountService) -> Self {
        Self {
            pool,
            account_service,
        }
    }

    /// Retrieves a transaction by its unique ID
    ///
    /// # Arguments
    /// * `id` - The UUID of the transaction to retrieve
    ///
    /// # Returns
    /// The transaction details wrapped in a TransactionResponse if found
    pub async fn get_transaction_by_id(&self, id: Uuid) -> Result<TransactionResponse, AppError> {
        let transaction = sqlx::query_as!(
            Transaction,
            r#"
            SELECT id, sender_account_id, receiver_account_id, amount as "amount: SqlxDecimal", currency, 
                   transaction_type, status, description, created_at, updated_at
            FROM transactions WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Transaction with ID {} not found", id)))?;

        Ok(TransactionResponse::from(transaction))
    }

    /// Gets all transactions associated with a specific account
    ///
    /// This will find transactions where the account is either the sender or receiver
    ///
    /// # Arguments
    /// * `account_id` - The UUID of the account to get transactions for
    /// * `limit` - Optional limit on the number of transactions to return (defaults to 100)
    /// * `offset` - Optional offset for pagination (defaults to 0)
    ///
    /// # Returns
    /// A vector of transaction responses, sorted by creation date (newest first)
    pub async fn get_transactions_by_account_id(
        &self,
        account_id: Uuid,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<TransactionResponse>, AppError> {
        let transactions = sqlx::query_as!(
            Transaction,
            r#"
            SELECT id, sender_account_id, receiver_account_id, amount as "amount: SqlxDecimal", currency, 
                   transaction_type, status, description, created_at, updated_at
            FROM transactions
            WHERE sender_account_id = $1 OR receiver_account_id = $1
            ORDER BY created_at DESC
            LIMIT $2
            OFFSET $3
            "#,
            account_id,
            limit.unwrap_or(100),
            offset.unwrap_or(0)
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(transactions
            .into_iter()
            .map(TransactionResponse::from)
            .collect())
    }

    /// Generic transaction creation endpoint that routes to the appropriate
    /// specialized transaction handler based on transaction type
    ///
    /// # Arguments
    /// * `request` - The transaction request containing all necessary details
    ///
    /// # Returns
    /// The created transaction response upon success
    ///
    /// # Implementation Note
    /// This method acts as a facade that maps the generic request to specialized
    /// transaction types (transfer, deposit, withdrawal) with appropriate validation.
    pub async fn create_transaction(
        &self,
        request: CreateTransactionRequest,
    ) -> Result<TransactionResponse, AppError> {
        // Convert the string transaction type to the appropriate enum variant
        let transaction_type = match request.transaction_type.as_str() {
            "TRANSFER" => TransactionType::TRANSFER,
            "DEPOSIT" => TransactionType::DEPOSIT,
            "WITHDRAWAL" => TransactionType::WITHDRAWAL,
            _ => {
                return Err(AppError::BadRequest(format!(
                    "Invalid transaction type: {}",
                    request.transaction_type
                )))
            }
        };

        // Route to the appropriate specialized handler based on transaction type
        match transaction_type {
            TransactionType::TRANSFER => {
                // For transfers, both sender and receiver accounts are required
                if request.sender_account_id.is_none() || request.receiver_account_id.is_none() {
                    return Err(AppError::BadRequest(
                        "Sender and receiver account IDs are required for transfers".to_string(),
                    ));
                }

                let transfer_request = TransferRequest {
                    sender_account_id: request.sender_account_id.unwrap(),
                    receiver_account_id: request.receiver_account_id.unwrap(),
                    amount: request.amount,
                    description: request.description,
                };

                self.process_transfer(transfer_request).await
            }
            TransactionType::DEPOSIT => {
                // For deposits, only the receiver account is required
                if request.receiver_account_id.is_none() {
                    return Err(AppError::BadRequest(
                        "Receiver account ID is required for deposits".to_string(),
                    ));
                }

                let deposit_request = DepositRequest {
                    account_id: request.receiver_account_id.unwrap(),
                    amount: request.amount,
                    description: request.description,
                };

                self.process_deposit(deposit_request).await
            }
            TransactionType::WITHDRAWAL => {
                // For withdrawals, only the sender account is required
                if request.sender_account_id.is_none() {
                    return Err(AppError::BadRequest(
                        "Sender account ID is required for withdrawals".to_string(),
                    ));
                }

                let withdrawal_request = WithdrawalRequest {
                    account_id: request.sender_account_id.unwrap(),
                    amount: request.amount,
                    description: request.description,
                };

                self.process_withdrawal(withdrawal_request).await
            }
        }
    }

    /// Processes a transfer between two accounts
    ///
    /// # Arguments
    /// * `request` - Transfer request containing sender and receiver accounts, amount, and description
    ///
    /// # Returns
    /// The completed transaction response upon success
    ///
    /// # Implementation Details
    /// This method:
    /// 1. Begins a database transaction for atomicity
    /// 2. Validates both accounts exist and are different
    /// 3. Checks that both accounts use the same currency
    /// 4. Verifies the sender has sufficient funds
    /// 5. Creates a pending transaction record
    /// 6. Updates both account balances
    /// 7. Marks the transaction as completed
    /// 8. Commits the database transaction
    ///
    /// If any step fails, the entire database transaction is rolled back.
    pub async fn process_transfer(
        &self,
        request: TransferRequest,
    ) -> Result<TransactionResponse, AppError> {
        // Start a database transaction to ensure atomicity and isolation
        // This ensures that either all operations succeed or all fail together
        let mut tx = self.pool.begin().await?;

        // Validate accounts exist and are different - prevents self-transfers
        // which could be used for fraudulent activity or money laundering
        if request.sender_account_id == request.receiver_account_id {
            return Err(AppError::BadRequest(
                "Cannot transfer to the same account".to_string(),
            ));
        }

        // Lock the sender account for the duration of this transaction
        // FOR UPDATE clause ensures exclusive access to prevent race conditions
        // This is critical to prevent double-spending
        let sender_account = sqlx::query!(
            r#"
            SELECT id, currency, balance FROM accounts WHERE id = $1 FOR UPDATE
            "#,
            request.sender_account_id
        )
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| {
            AppError::NotFound(format!(
                "Sender account with ID {} not found",
                request.sender_account_id
            ))
        })?;

        // Lock the receiver account for the duration of this transaction
        // FOR UPDATE clause again for race condition prevention
        let receiver_account = sqlx::query!(
            r#"
            SELECT id, currency FROM accounts WHERE id = $1 FOR UPDATE
            "#,
            request.receiver_account_id
        )
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| {
            AppError::NotFound(format!(
                "Receiver account with ID {} not found",
                request.receiver_account_id
            ))
        })?;

        // Ensure matching currencies - prevents currency conversion issues
        // We don't handle currency exchange in this system
        if sender_account.currency != receiver_account.currency {
            return Err(AppError::BadRequest(
                "Currency mismatch between accounts".to_string(),
            ));
        }

        // Ensure sufficient balance in the sender account
        // Get balance as string and convert to Decimal for precise comparison
        // We use a raw query with format! to handle our custom SqlxDecimal type
        let query = format!(
            "SELECT balance::TEXT FROM accounts WHERE id = '{}' FOR UPDATE",
            request.sender_account_id
        );

        let row = sqlx::query(&query).fetch_one(&mut *tx).await?;

        // Parse the balance text to a Decimal for precise financial calculations
        // ZERO is the fallback in case of parsing error
        let sender_balance: Decimal = sqlx::Row::get::<&str, _>(&row, "balance")
            .parse()
            .unwrap_or(Decimal::ZERO);

        // Ensure the sender has enough funds for the transfer
        if sender_balance < request.amount {
            return Err(AppError::BadRequest("Insufficient funds".to_string()));
        }

        // Create a transaction record in PENDING state - this serves as an audit trail
        // We use a UUID v4 for a globally unique transaction identifier
        let transaction_id = Uuid::new_v4();
        let _transaction = self
            .create_transaction_record(
                &mut tx,
                transaction_id,
                Some(request.sender_account_id),
                Some(request.receiver_account_id),
                request.amount,
                sender_account.currency.clone(),
                TransactionType::TRANSFER.to_string(),
                request.description,
            )
            .await?;

        // Update sender balance by REDUCING it by the transfer amount
        // Note the negative amount to indicate funds leaving the account
        self.update_account_balance(&mut tx, request.sender_account_id, -request.amount)
            .await?;

        // Update receiver balance by INCREASING it by the transfer amount
        self.update_account_balance(&mut tx, request.receiver_account_id, request.amount)
            .await?;

        // Update transaction status to COMPLETED now that both accounts are updated
        // This final state indicates the successful completion of the transfer
        let updated_transaction = self
            .update_transaction_status(
                &mut tx,
                transaction_id,
                TransactionStatus::COMPLETED.to_string(),
            )
            .await?;

        // Commit the database transaction to persist all changes atomically
        // If any step above failed, the transaction would be rolled back automatically
        tx.commit().await?;

        // Return the transaction details to the caller
        Ok(TransactionResponse::from(updated_transaction))
    }

    /// Processes a deposit into an account
    ///
    /// A deposit represents money coming into the system from outside.
    /// For example, this could be a bank transfer, cash deposit, or other external funds.
    ///
    /// # Arguments
    /// * `request` - Deposit request containing account ID, amount, and description
    ///
    /// # Returns
    /// The completed transaction response upon success
    ///
    /// # Implementation Details
    /// This method:
    /// 1. Begins a database transaction for atomicity
    /// 2. Validates the target account exists
    /// 3. Creates a pending transaction record with no sender (external source)
    /// 4. Updates the account balance
    /// 5. Marks the transaction as completed
    /// 6. Commits the database transaction
    pub async fn process_deposit(
        &self,
        request: DepositRequest,
    ) -> Result<TransactionResponse, AppError> {
        // Start a database transaction to ensure atomicity of operations
        let mut tx = self.pool.begin().await?;

        // Verify account exists and lock it for update to prevent race conditions
        let account = sqlx::query!(
            r#"
            SELECT id, currency FROM accounts WHERE id = $1 FOR UPDATE
            "#,
            request.account_id
        )
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| {
            AppError::NotFound(format!("Account with ID {} not found", request.account_id))
        })?;

        // Create a transaction record with no sender_account_id (money comes from outside)
        // but with the receiver_account_id set to the deposit account
        let transaction_id = Uuid::new_v4();
        let _transaction = self
            .create_transaction_record(
                &mut tx,
                transaction_id,
                None, // No sender account for deposits (external source)
                Some(request.account_id),
                request.amount,
                account.currency.clone(),
                TransactionType::DEPOSIT.to_string(),
                request.description,
            )
            .await?;

        // Increase the account balance by the deposit amount
        // Since deposits always increase the balance, we pass a positive amount
        self.update_account_balance(&mut tx, request.account_id, request.amount)
            .await?;

        // Update transaction status to COMPLETED
        let updated_transaction = self
            .update_transaction_status(
                &mut tx,
                transaction_id,
                TransactionStatus::COMPLETED.to_string(),
            )
            .await?;

        // Commit all changes as a single atomic operation
        tx.commit().await?;

        // Return transaction details
        Ok(TransactionResponse::from(updated_transaction))
    }

    /// Processes a withdrawal from an account
    ///
    /// A withdrawal represents money leaving the system entirely.
    /// For example, this could be an ATM withdrawal, bank transfer out, or other external payment.
    ///
    /// # Arguments
    /// * `request` - Withdrawal request containing account ID, amount, and description
    ///
    /// # Returns
    /// The completed transaction response upon success
    ///
    /// # Implementation Details
    /// This method:
    /// 1. Begins a database transaction for atomicity
    /// 2. Validates the source account exists
    /// 3. Verifies the account has sufficient funds
    /// 4. Creates a pending transaction record with no receiver (external destination)
    /// 5. Updates the account balance
    /// 6. Marks the transaction as completed
    /// 7. Commits the database transaction
    pub async fn process_withdrawal(
        &self,
        request: WithdrawalRequest,
    ) -> Result<TransactionResponse, AppError> {
        // Start a database transaction to ensure atomicity
        let mut tx = self.pool.begin().await?;

        // Verify account exists and lock it for update
        let account = sqlx::query!(
            r#"
            SELECT id, currency, balance FROM accounts WHERE id = $1 FOR UPDATE
            "#,
            request.account_id
        )
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| {
            AppError::NotFound(format!("Account with ID {} not found", request.account_id))
        })?;

        // Ensure sufficient balance for withdrawal - prevent overdrafts
        // Use raw query to get balance as text for precise decimal handling
        let query = format!(
            "SELECT balance::TEXT FROM accounts WHERE id = '{}' FOR UPDATE",
            request.account_id
        );

        let row = sqlx::query(&query).fetch_one(&mut *tx).await?;

        // Parse balance from text to Decimal for accurate comparison
        let account_balance: Decimal = sqlx::Row::get::<&str, _>(&row, "balance")
            .parse()
            .unwrap_or(Decimal::ZERO);

        // Verify sufficient funds
        if account_balance < request.amount {
            return Err(AppError::BadRequest("Insufficient funds".to_string()));
        }

        // Create transaction record with sender_account_id set but no receiver_account_id
        // This pattern indicates money leaving the system to an external destination
        let transaction_id = Uuid::new_v4();
        let _transaction = self
            .create_transaction_record(
                &mut tx,
                transaction_id,
                Some(request.account_id),
                None, // No receiver account for withdrawals (external destination)
                request.amount,
                account.currency.clone(),
                TransactionType::WITHDRAWAL.to_string(),
                request.description,
            )
            .await?;

        // Decrease account balance by withdrawal amount
        // Negative amount indicates funds leaving the account
        self.update_account_balance(&mut tx, request.account_id, -request.amount)
            .await?;

        // Update transaction status to COMPLETED
        let updated_transaction = self
            .update_transaction_status(
                &mut tx,
                transaction_id,
                TransactionStatus::COMPLETED.to_string(),
            )
            .await?;

        // Commit all changes as a single atomic operation
        tx.commit().await?;

        // Return transaction details
        Ok(TransactionResponse::from(updated_transaction))
    }

    /// Helper function to create a transaction record in the database
    ///
    /// # Arguments
    /// * `tx` - Database transaction to use
    /// * `id` - Unique ID for the transaction
    /// * `sender_account_id` - Optional sender account ID
    /// * `receiver_account_id` - Optional receiver account ID
    /// * `amount` - Transaction amount
    /// * `currency` - Currency code
    /// * `transaction_type` - Type of transaction (TRANSFER, DEPOSIT, WITHDRAWAL)
    /// * `description` - Optional transaction description
    ///
    /// # Returns
    /// The created transaction record
    ///
    /// # Implementation Note
    /// This uses raw SQL queries due to complexities with the SQLx type system and our
    /// custom SqlxDecimal type. The transaction is created in PENDING status initially.
    async fn create_transaction_record(
        &self,
        tx: &mut SqlxTransaction<'_, Postgres>,
        id: Uuid,
        sender_account_id: Option<Uuid>,
        receiver_account_id: Option<Uuid>,
        amount: Decimal,
        currency: String,
        transaction_type: String,
        description: Option<String>,
    ) -> Result<Transaction, AppError> {
        // Format nullable fields for SQL insertion
        // Using NULL for SQL when the field is None
        let sender_id_str = match sender_account_id {
            Some(id) => format!("'{}'", id),
            None => "NULL".to_string(),
        };

        let receiver_id_str = match receiver_account_id {
            Some(id) => format!("'{}'", id),
            None => "NULL".to_string(),
        };

        // Handle SQL injection prevention for the description field
        // Escape single quotes in the description text
        let description_str = match &description {
            Some(desc) => format!("'{}'", desc.replace("'", "''")), // Escape single quotes
            None => "NULL".to_string(),
        };

        // Construct and execute the raw SQL query
        // We explicitly cast the amount to TEXT in the RETURNING clause
        // for consistent handling of our custom decimal type
        let query = format!(
            "INSERT INTO transactions 
            (id, sender_account_id, receiver_account_id, amount, currency, transaction_type, status, description)
            VALUES ('{}', {}, {}, '{}', '{}', '{}', '{}', {})
            RETURNING id, sender_account_id, receiver_account_id, amount::TEXT, currency, 
                     transaction_type, status, description, created_at, updated_at",
            id,
            sender_id_str,
            receiver_id_str,
            amount.to_string(),
            currency,
            transaction_type,
            TransactionStatus::PENDING.to_string(), // All transactions start as PENDING
            description_str
        );

        let row = sqlx::query(&query).fetch_one(&mut **tx).await?;

        // Manually construct the Transaction struct from the SQL row
        // This is needed because we can't use query_as! with our dynamic query
        let transaction = Transaction {
            id: sqlx::Row::get(&row, "id"),
            sender_account_id: sqlx::Row::get(&row, "sender_account_id"),
            receiver_account_id: sqlx::Row::get(&row, "receiver_account_id"),
            amount: SqlxDecimal(
                sqlx::Row::get::<&str, _>(&row, "amount")
                    .parse()
                    .unwrap_or(Decimal::ZERO),
            ),
            currency: sqlx::Row::get(&row, "currency"),
            transaction_type: sqlx::Row::get(&row, "transaction_type"),
            status: sqlx::Row::get(&row, "status"),
            description: sqlx::Row::get(&row, "description"),
            created_at: sqlx::Row::get(&row, "created_at"),
            updated_at: sqlx::Row::get(&row, "updated_at"),
        };

        Ok(transaction)
    }

    /// Helper function to update an account balance within a database transaction
    ///
    /// # Arguments
    /// * `tx` - Database transaction to use
    /// * `account_id` - ID of the account to update
    /// * `amount` - Amount to add to the balance (negative for subtraction)
    ///
    /// # Returns
    /// Nothing if successful, error otherwise
    ///
    /// # Implementation Note
    /// This uses a raw SQL query to avoid issues with the SQLx type system and
    /// our custom SqlxDecimal type. The account balance check is handled at the
    /// database level with a CHECK constraint.
    async fn update_account_balance(
        &self,
        tx: &mut SqlxTransaction<'_, Postgres>,
        account_id: Uuid,
        amount: Decimal,
    ) -> Result<(), AppError> {
        // Convert Decimal to string for PostgreSQL compatibility using raw query
        // This precision-preserving conversion is critical for financial calculations
        let query = format!(
            "UPDATE accounts
             SET balance = balance + '{}',
                 updated_at = NOW()
             WHERE id = '{}'",
            amount.to_string(),
            account_id
        );

        // Execute the query within the provided transaction
        // The database constraint balance_non_negative will prevent negative balances
        sqlx::query(&query).execute(&mut **tx).await?;

        Ok(())
    }

    /// Helper function to update a transaction's status
    ///
    /// # Arguments
    /// * `tx` - Database transaction to use
    /// * `transaction_id` - ID of the transaction to update
    /// * `status` - New status (typically COMPLETED or FAILED)
    ///
    /// # Returns
    /// The updated transaction record
    ///
    /// # Implementation Note
    /// This uses a raw SQL query for consistency with our other methods.
    /// The updated transaction's fields are returned for audit purposes.
    async fn update_transaction_status(
        &self,
        tx: &mut SqlxTransaction<'_, Postgres>,
        transaction_id: Uuid,
        status: String,
    ) -> Result<Transaction, AppError> {
        // Use raw query to bypass type checking challenges
        let query = format!(
            "UPDATE transactions
             SET status = '{}',
                 updated_at = NOW()
             WHERE id = '{}'
             RETURNING id, sender_account_id, receiver_account_id, amount::TEXT, currency, 
                      transaction_type, status, description, created_at, updated_at",
            status, transaction_id
        );

        let row = sqlx::query(&query).fetch_one(&mut **tx).await?;

        // Manually create the Transaction struct from row data
        let transaction = Transaction {
            id: sqlx::Row::get(&row, "id"),
            sender_account_id: sqlx::Row::get(&row, "sender_account_id"),
            receiver_account_id: sqlx::Row::get(&row, "receiver_account_id"),
            amount: SqlxDecimal(
                sqlx::Row::get::<&str, _>(&row, "amount")
                    .parse()
                    .unwrap_or(Decimal::ZERO),
            ),
            currency: sqlx::Row::get(&row, "currency"),
            transaction_type: sqlx::Row::get(&row, "transaction_type"),
            status: sqlx::Row::get(&row, "status"),
            description: sqlx::Row::get(&row, "description"),
            created_at: sqlx::Row::get(&row, "created_at"),
            updated_at: sqlx::Row::get(&row, "updated_at"),
        };

        Ok(transaction)
    }
}
