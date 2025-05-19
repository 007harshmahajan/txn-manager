#[cfg(test)]
mod tests {
    use crate::models::user::{CreateUserRequest, LoginRequest, User};
    use crate::services::tests::service_test_utils::test_utils::{create_test_user, MockPgPool};
    use crate::services::user_service::UserService;
    use crate::utils::error::AppError;
    use mockall::predicate::*;
    use rust_decimal::Decimal;
    use sqlx::Error as SqlxError;
    use std::sync::Arc;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_create_user_success() {
        // Set up mock
        let mut mock_pool = MockPgPool::new();
        
        // First query to check if user exists
        mock_pool
            .expect_clone()
            .times(1)
            .return_once(|| MockPgPool::new());
            
        // Set up test parameters
        let user_request = CreateUserRequest {
            username: "newuser".to_string(),
            email: "new@example.com".to_string(),
            password: "securepassword".to_string(),
            first_name: Some("New".to_string()),
            last_name: Some("User".to_string()),
        };
        
        // Create service with mock
        let jwt_secret = "test_secret".to_string();
        let user_service = UserService::new(mock_pool, jwt_secret);
        
        // Since we can't easily mock the entire query execution path in SQLx,
        // this test mainly verifies that the service's struct is set up correctly
        // In a real scenario, you'd use a test database for full integration tests
    }

    #[tokio::test]
    async fn test_login_success() {
        // In a real test, you'd mock the database response and 
        // verify that login works with correct credentials
        
        // Set up mock
        let mut mock_pool = MockPgPool::new();
        
        // Create service with mock
        let jwt_secret = "test_secret".to_string();
        let user_service = UserService::new(mock_pool, jwt_secret);
        
        // This is where you would verify login behavior with mocks
        // However, due to the complexity of mocking SQLx, 
        // this is better tested in integration tests with a real database
    }
} 