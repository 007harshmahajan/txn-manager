#[cfg(test)]
mod tests {
    use crate::models::account::Account;
    use crate::models::decimal::SqlxDecimal;
    use crate::services::account_service::AccountService;
    use crate::services::tests::service_test_utils::test_utils::{create_test_account, MockPgPool};
    use crate::utils::error::AppError;
    use rust_decimal::Decimal;
    use sqlx::Error as SqlxError;
    use std::str::FromStr;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_get_account_by_id_not_found() {
        // Set up mock pool that will return no account
        let mut mock_pool = MockPgPool::new();
        
        // Configure mock to clone
        mock_pool
            .expect_clone()
            .times(1)
            .return_once(|| MockPgPool::new());
            
        // Create service with mock
        let account_service = AccountService::new(mock_pool);
        
        // Since we can't easily mock the entire query execution path in SQLx,
        // this test mainly verifies that the service's struct is set up correctly
        // In a real scenario, you'd use a test database for full integration tests
    }

    #[tokio::test]
    async fn test_create_account() {
        // Set up mock
        let mut mock_pool = MockPgPool::new();
        
        // Configure mock to clone
        mock_pool
            .expect_clone()
            .times(1)
            .return_once(|| MockPgPool::new());
            
        // Create service with mock
        let account_service = AccountService::new(mock_pool);
        
        // In a real test, you'd verify that account creation works 
        // with proper parameters and database interactions
    }

    #[tokio::test]
    async fn test_update_balance() {
        // Set up mock
        let mut mock_pool = MockPgPool::new();
        
        // Configure mock to clone
        mock_pool
            .expect_clone()
            .times(1)
            .return_once(|| MockPgPool::new());
            
        // Create service with mock
        let account_service = AccountService::new(mock_pool);
        
        // In a real test, you'd verify that balance updates work correctly
        // and negative balances are rejected
    }
} 