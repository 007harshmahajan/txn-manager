#[cfg(test)]
mod tests {
    use crate::models::user::{CreateUserRequest, LoginRequest};
    use crate::services::user_service::UserService;
    use sqlx::postgres::PgPoolOptions;
    use std::env;
    use dotenv::dotenv;

    // This test requires a running PostgreSQL database
    // Run with: cargo test -- --ignored user_service_test
    #[tokio::test]
    #[ignore]
    async fn test_user_creation_and_login() {
        dotenv().ok();
        
        // Get database URL from environment or use a test database
        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5433/txn_manager_test".to_string());
        
        // Connect to the database
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to the database");
        
        // Create a user service
        let jwt_secret = "test_secret_key".to_string();
        let user_service = UserService::new(pool.clone(), jwt_secret);
        
        // Create a unique username for testing
        let test_username = format!("testuser_{}", chrono::Utc::now().timestamp());
        
        // Create a test user
        let create_request = CreateUserRequest {
            username: test_username.clone(),
            email: format!("{}@example.com", test_username),
            password: "securepassword123".to_string(),
            first_name: Some("Test".to_string()),
            last_name: Some("User".to_string()),
        };
        
        // Register the user
        let user_response = user_service.create_user(create_request).await.expect("Failed to create user");
        
        // Verify user data
        assert_eq!(user_response.username, test_username);
        assert_eq!(user_response.first_name.as_ref().unwrap(), "Test");
        assert_eq!(user_response.last_name.as_ref().unwrap(), "User");
        
        // Test login
        let login_request = LoginRequest {
            username: test_username,
            password: "securepassword123".to_string(),
        };
        
        // Login with created user
        let login_response = user_service.login(login_request).await.expect("Failed to login");
        
        // Verify login data
        assert_eq!(login_response.user.username, user_response.username);
        assert_eq!(login_response.user.email, user_response.email);
        assert!(login_response.token.len() > 0, "JWT token should not be empty");
        
        // Clean up - in a real test you would use transactions or a test database that gets reset
    }
    
    // Add more tests for user service functionality
} 