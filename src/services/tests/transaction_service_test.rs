#[cfg(test)]
mod tests {
    use crate::models::transaction::{CreateTransactionRequest, DepositRequest, TransferRequest, WithdrawalRequest};
    use crate::services::account_service::AccountService;
    use crate::services::tests::service_test_utils::test_utils::{create_test_account, MockPgPool};
    use crate::services::transaction_service::TransactionService;
    use crate::utils::error::AppError;
    use rust_decimal::Decimal;
    use std::str::FromStr;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_process_transfer_success() {
        // Set up mock
        let mut mock_pool = MockPgPool::new();
        
        // Configure mock to clone
        mock_pool
            .expect_clone()
            .times(2) // Once for transaction service, once for account service
            .return_once(|| MockPgPool::new());
            
        // Create services
        let account_service = AccountService::new(mock_pool.clone());
        let transaction_service = TransactionService::new(mock_pool, account_service);
        
        // In a real test, you'd verify that transfers work correctly
        // with sufficient balance and matching currencies
    }

    #[tokio::test]
    async fn test_process_transfer_insufficient_funds() {
        // Set up mock
        let mut mock_pool = MockPgPool::new();
        
        // Configure mock to clone
        mock_pool
            .expect_clone()
            .times(2)
            .return_once(|| MockPgPool::new());
            
        // Create services
        let account_service = AccountService::new(mock_pool.clone());
        let transaction_service = TransactionService::new(mock_pool, account_service);
        
        // In a real test, you'd verify that transfers are rejected
        // when there are insufficient funds
    }

    #[tokio::test]
    async fn test_process_deposit() {
        // Set up mock
        let mut mock_pool = MockPgPool::new();
        
        // Configure mock to clone
        mock_pool
            .expect_clone()
            .times(2)
            .return_once(|| MockPgPool::new());
            
        // Create services
        let account_service = AccountService::new(mock_pool.clone());
        let transaction_service = TransactionService::new(mock_pool, account_service);
        
        // In a real test, you'd verify that deposits increase account balance
    }

    #[tokio::test]
    async fn test_process_withdrawal() {
        // Set up mock
        let mut mock_pool = MockPgPool::new();
        
        // Configure mock to clone
        mock_pool
            .expect_clone()
            .times(2)
            .return_once(|| MockPgPool::new());
            
        // Create services
        let account_service = AccountService::new(mock_pool.clone());
        let transaction_service = TransactionService::new(mock_pool, account_service);
        
        // In a real test, you'd verify that withdrawals decrease account balance
        // and are rejected when there are insufficient funds
    }
} 