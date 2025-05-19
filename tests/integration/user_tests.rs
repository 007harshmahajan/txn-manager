use crate::integration::setup::{create_account_service, create_user_service, setup, teardown};
use txn_manager::{CreateUserRequest, LoginRequest};

#[tokio::test]
async fn test_user_registration_and_login() {
    // Set up test environment
    let (pool, db_url) = setup().await;

    // Create user service
    let user_service = create_user_service(pool.clone());

    // Test user registration
    let user_request = CreateUserRequest {
        username: "testuser".to_string(),
        email: "test@example.com".to_string(),
        password: "securepassword".to_string(),
        first_name: Some("Test".to_string()),
        last_name: Some("User".to_string()),
    };

    let user_result = user_service.create_user(user_request).await;
    assert!(
        user_result.is_ok(),
        "User creation failed: {:?}",
        user_result.err()
    );

    let user = user_result.unwrap();
    assert_eq!(user.username, "testuser");
    assert_eq!(user.email, "test@example.com");
    assert_eq!(user.first_name.as_ref().unwrap(), "Test");
    assert_eq!(user.last_name.as_ref().unwrap(), "User");

    // Test login
    let login_request = LoginRequest {
        username: "testuser".to_string(),
        password: "securepassword".to_string(),
    };

    let login_result = user_service.login(login_request).await;
    assert!(
        login_result.is_ok(),
        "Login failed: {:?}",
        login_result.err()
    );

    let login_response = login_result.unwrap();
    assert_eq!(login_response.user.username, "testuser");
    assert_eq!(login_response.user.email, "test@example.com");
    assert!(!login_response.token.is_empty(), "JWT token is empty");

    // Test login with incorrect password
    let login_request = LoginRequest {
        username: "testuser".to_string(),
        password: "wrongpassword".to_string(),
    };

    let login_result = user_service.login(login_request).await;
    assert!(
        login_result.is_err(),
        "Login should fail with incorrect password"
    );

    // Clean up test environment
    teardown(&db_url).await;
}

#[tokio::test]
async fn test_user_creation() {
    // Set up test environment
    let (pool, db_url) = setup().await;

    // Create user service
    let user_service = create_user_service(pool.clone());

    // Create a test user
    let user_request = CreateUserRequest {
        username: "testuser1".to_string(),
        email: "test1@example.com".to_string(),
        password: "securepassword".to_string(),
        first_name: Some("Test".to_string()),
        last_name: Some("User".to_string()),
    };

    let user_result = user_service.create_user(user_request).await;
    assert!(user_result.is_ok(), "User creation should succeed");

    let user = user_result.unwrap();
    assert_eq!(user.username, "testuser1");
    assert_eq!(user.email, "test1@example.com");
    assert_eq!(user.first_name, Some("Test".to_string()));
    assert_eq!(user.last_name, Some("User".to_string()));

    // Verify that an account service can see the default account
    let account_service = create_account_service(pool.clone());
    let accounts = account_service
        .get_accounts_by_user_id(user.id)
        .await
        .unwrap();
    assert_eq!(
        accounts.len(),
        1,
        "New user should have one default account"
    );

    // Clean up test environment
    teardown(&db_url).await;
}

#[tokio::test]
async fn test_duplicate_user() {
    // Set up test environment
    let (pool, db_url) = setup().await;

    // Create user service
    let user_service = create_user_service(pool.clone());

    // Create first user
    let user_request = CreateUserRequest {
        username: "duplicatetest".to_string(),
        email: "duplicate@example.com".to_string(),
        password: "securepassword".to_string(),
        first_name: Some("Duplicate".to_string()),
        last_name: Some("Test".to_string()),
    };

    let first_result = user_service.create_user(user_request.clone()).await;
    assert!(first_result.is_ok(), "First user creation should succeed");

    // Try to create duplicate user
    let duplicate_result = user_service.create_user(user_request).await;
    assert!(
        duplicate_result.is_err(),
        "Duplicate user creation should fail"
    );

    // Clean up test environment
    teardown(&db_url).await;
}

#[tokio::test]
async fn test_user_login() {
    // Set up test environment
    let (pool, db_url) = setup().await;

    // Create user service
    let user_service = create_user_service(pool.clone());

    // Create a test user
    let user_request = CreateUserRequest {
        username: "logintest".to_string(),
        email: "login@example.com".to_string(),
        password: "securepassword".to_string(),
        first_name: Some("Login".to_string()),
        last_name: Some("Test".to_string()),
    };

    let user = user_service.create_user(user_request).await.unwrap();

    // Test successful login
    let login_request = LoginRequest {
        username: "logintest".to_string(),
        password: "securepassword".to_string(),
    };

    let login_result = user_service.login(login_request).await;
    assert!(
        login_result.is_ok(),
        "Login should succeed with correct credentials"
    );

    let login_response = login_result.unwrap();
    assert!(
        !login_response.token.is_empty(),
        "JWT token should be returned"
    );
    assert_eq!(login_response.user.id, user.id);
    assert_eq!(login_response.user.username, "logintest");

    // Test failed login with incorrect password
    let failed_login_request = LoginRequest {
        username: "logintest".to_string(),
        password: "wrongpassword".to_string(),
    };

    let failed_login_result = user_service.login(failed_login_request).await;
    assert!(
        failed_login_result.is_err(),
        "Login should fail with incorrect password"
    );

    // Test failed login with non-existent user
    let nonexistent_login_request = LoginRequest {
        username: "nonexistentuser".to_string(),
        password: "anypassword".to_string(),
    };

    let nonexistent_login_result = user_service.login(nonexistent_login_request).await;
    assert!(
        nonexistent_login_result.is_err(),
        "Login should fail with non-existent user"
    );

    // Clean up test environment
    teardown(&db_url).await;
}

#[tokio::test]
async fn test_get_user_profile() {
    // Set up test environment
    let (pool, db_url) = setup().await;

    // Create user service
    let user_service = create_user_service(pool.clone());

    // Create a test user
    let user_request = CreateUserRequest {
        username: "profiletest".to_string(),
        email: "profile@example.com".to_string(),
        password: "securepassword".to_string(),
        first_name: Some("Profile".to_string()),
        last_name: Some("Test".to_string()),
    };

    let created_user = user_service.create_user(user_request).await.unwrap();

    // Retrieve user profile
    let retrieved_user = user_service.get_user_by_id(created_user.id).await.unwrap();

    // Verify profile data
    assert_eq!(retrieved_user.id, created_user.id);
    assert_eq!(retrieved_user.username, "profiletest");
    assert_eq!(retrieved_user.email, "profile@example.com");
    assert_eq!(retrieved_user.first_name, Some("Profile".to_string()));
    assert_eq!(retrieved_user.last_name, Some("Test".to_string()));

    // Try to retrieve non-existent user
    let random_id = uuid::Uuid::new_v4();
    let not_found_result = user_service.get_user_by_id(random_id).await;
    assert!(
        not_found_result.is_err(),
        "Should return error for non-existent user"
    );

    // Clean up test environment
    teardown(&db_url).await;
}
