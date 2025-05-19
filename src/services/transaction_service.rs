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

    pub async fn create_transaction(
        &self,
        request: CreateTransactionRequest,
    ) -> Result<TransactionResponse, AppError> {
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

        match transaction_type {
            TransactionType::TRANSFER => {
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

    pub async fn process_transfer(
        &self,
        request: TransferRequest,
    ) -> Result<TransactionResponse, AppError> {
        // Start a database transaction
        let mut tx = self.pool.begin().await?;

        // Validate accounts exist and are different
        if request.sender_account_id == request.receiver_account_id {
            return Err(AppError::BadRequest(
                "Cannot transfer to the same account".to_string(),
            ));
        }

        // Get sender account for currency check and validation
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

        // Get receiver account for currency validation
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

        // Ensure matching currencies
        if sender_account.currency != receiver_account.currency {
            return Err(AppError::BadRequest(
                "Currency mismatch between accounts".to_string(),
            ));
        }

        // Ensure sufficient balance
        // Get balance as string and convert to Decimal
        let query = format!(
            "SELECT balance::TEXT FROM accounts WHERE id = '{}' FOR UPDATE",
            request.sender_account_id
        );

        let row = sqlx::query(&query).fetch_one(&mut *tx).await?;

        let sender_balance: Decimal = sqlx::Row::get::<&str, _>(&row, "balance")
            .parse()
            .unwrap_or(Decimal::ZERO);

        if sender_balance < request.amount {
            return Err(AppError::BadRequest("Insufficient funds".to_string()));
        }

        // Create transaction record
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

        // Update sender balance
        self.update_account_balance(&mut tx, request.sender_account_id, -request.amount)
            .await?;

        // Update receiver balance
        self.update_account_balance(&mut tx, request.receiver_account_id, request.amount)
            .await?;

        // Update transaction status
        let updated_transaction = self
            .update_transaction_status(
                &mut tx,
                transaction_id,
                TransactionStatus::COMPLETED.to_string(),
            )
            .await?;

        // Commit transaction
        tx.commit().await?;

        Ok(TransactionResponse::from(updated_transaction))
    }

    pub async fn process_deposit(
        &self,
        request: DepositRequest,
    ) -> Result<TransactionResponse, AppError> {
        // Start a database transaction
        let mut tx = self.pool.begin().await?;

        // Get account
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

        // Create transaction record
        let transaction_id = Uuid::new_v4();
        let _transaction = self
            .create_transaction_record(
                &mut tx,
                transaction_id,
                None,
                Some(request.account_id),
                request.amount,
                account.currency.clone(),
                TransactionType::DEPOSIT.to_string(),
                request.description,
            )
            .await?;

        // Update account balance
        self.update_account_balance(&mut tx, request.account_id, request.amount)
            .await?;

        // Update transaction status
        let updated_transaction = self
            .update_transaction_status(
                &mut tx,
                transaction_id,
                TransactionStatus::COMPLETED.to_string(),
            )
            .await?;

        // Commit transaction
        tx.commit().await?;

        Ok(TransactionResponse::from(updated_transaction))
    }

    pub async fn process_withdrawal(
        &self,
        request: WithdrawalRequest,
    ) -> Result<TransactionResponse, AppError> {
        // Start a database transaction
        let mut tx = self.pool.begin().await?;

        // Get account
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

        // Ensure sufficient balance
        // Get balance as string and convert to Decimal
        let query = format!(
            "SELECT balance::TEXT FROM accounts WHERE id = '{}' FOR UPDATE",
            request.account_id
        );

        let row = sqlx::query(&query).fetch_one(&mut *tx).await?;

        let account_balance: Decimal = sqlx::Row::get::<&str, _>(&row, "balance")
            .parse()
            .unwrap_or(Decimal::ZERO);

        if account_balance < request.amount {
            return Err(AppError::BadRequest("Insufficient funds".to_string()));
        }

        // Create transaction record
        let transaction_id = Uuid::new_v4();
        let _transaction = self
            .create_transaction_record(
                &mut tx,
                transaction_id,
                Some(request.account_id),
                None,
                request.amount,
                account.currency.clone(),
                TransactionType::WITHDRAWAL.to_string(),
                request.description,
            )
            .await?;

        // Update account balance
        self.update_account_balance(&mut tx, request.account_id, -request.amount)
            .await?;

        // Update transaction status
        let updated_transaction = self
            .update_transaction_status(
                &mut tx,
                transaction_id,
                TransactionStatus::COMPLETED.to_string(),
            )
            .await?;

        // Commit transaction
        tx.commit().await?;

        Ok(TransactionResponse::from(updated_transaction))
    }

    // Helper function to create a transaction record
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
        // Use raw query to bypass type checking
        let sender_id_str = match sender_account_id {
            Some(id) => format!("'{}'", id),
            None => "NULL".to_string(),
        };

        let receiver_id_str = match receiver_account_id {
            Some(id) => format!("'{}'", id),
            None => "NULL".to_string(),
        };

        let description_str = match &description {
            Some(desc) => format!("'{}'", desc.replace("'", "''")), // Escape single quotes
            None => "NULL".to_string(),
        };

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
            TransactionStatus::PENDING.to_string(),
            description_str
        );

        let row = sqlx::query(&query).fetch_one(&mut **tx).await?;

        // Manually create the Transaction struct
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

    // Helper function to update account balance
    async fn update_account_balance(
        &self,
        tx: &mut SqlxTransaction<'_, Postgres>,
        account_id: Uuid,
        amount: Decimal,
    ) -> Result<(), AppError> {
        // Convert Decimal to string for PostgreSQL compatibility using raw query
        let query = format!(
            "UPDATE accounts
             SET balance = balance + '{}',
                 updated_at = NOW()
             WHERE id = '{}'",
            amount.to_string(),
            account_id
        );

        sqlx::query(&query).execute(&mut **tx).await?;

        Ok(())
    }

    // Helper function to update transaction status
    async fn update_transaction_status(
        &self,
        tx: &mut SqlxTransaction<'_, Postgres>,
        transaction_id: Uuid,
        status: String,
    ) -> Result<Transaction, AppError> {
        // Use raw query to bypass type checking
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

        // Manually create the Transaction struct
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
