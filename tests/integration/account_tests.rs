use crate::integration::setup::{create_account_service, create_user_service, setup, teardown};
use rust_decimal::Decimal;
use txn_manager::CreateUserRequest;
use uuid::Uuid;

#[tokio::test]
async fn test_account_creation_and_retrieval() {
    // Set up test environment
    let (pool, db_url) = setup().await;

    // Create services
    let user_service = create_user_service(pool.clone());
    let account_service = create_account_service(pool.clone());

    // Create a test user
    let user_request = CreateUserRequest {
        username: "accountuser".to_string(),
        email: "account@example.com".to_string(),
        password: "securepassword".to_string(),
        first_name: Some("Account".to_string()),
        last_name: Some("User".to_string()),
    };

    let user_result = user_service.create_user(user_request).await;
    assert!(
        user_result.is_ok(),
        "User creation failed: {:?}",
        user_result.err()
    );

    let user = user_result.unwrap();

    // Test account creation
    let account_result = account_service
        .create_account(user.id, "EUR".to_string())
        .await;
    assert!(
        account_result.is_ok(),
        "Account creation failed: {:?}",
        account_result.err()
    );

    let account = account_result.unwrap();
    assert_eq!(account.user_id, user.id);
    assert_eq!(account.currency, "EUR");
    assert_eq!(account.balance, Decimal::ZERO);

    // Test get account by ID
    let get_account_result = account_service.get_account_by_id(account.id).await;
    assert!(
        get_account_result.is_ok(),
        "Get account failed: {:?}",
        get_account_result.err()
    );

    let retrieved_account = get_account_result.unwrap();
    assert_eq!(retrieved_account.id, account.id);
    assert_eq!(retrieved_account.user_id, user.id);
    assert_eq!(retrieved_account.currency, "EUR");

    // Test get accounts by user ID
    let get_accounts_result = account_service.get_accounts_by_user_id(user.id).await;
    assert!(
        get_accounts_result.is_ok(),
        "Get accounts failed: {:?}",
        get_accounts_result.err()
    );

    let accounts = get_accounts_result.unwrap();
    assert_eq!(accounts.len(), 2); // One from user creation (default) and one we just created

    // Test non-existent account
    let get_bad_account_result = account_service.get_account_by_id(Uuid::new_v4()).await;
    assert!(
        get_bad_account_result.is_err(),
        "Get non-existent account should fail"
    );

    // Clean up test environment
    teardown(&db_url).await;
}

#[tokio::test]
async fn test_account_balance_update_positive() {
    // Set up test environment
    let (pool, db_url) = setup().await;

    // Create services
    let user_service = create_user_service(pool.clone());
    let account_service = create_account_service(pool.clone());

    // Create a test user
    let user_request = CreateUserRequest {
        username: "balanceuser".to_string(),
        email: "balance@example.com".to_string(),
        password: "securepassword".to_string(),
        first_name: Some("Balance".to_string()),
        last_name: Some("User".to_string()),
    };

    let user = user_service.create_user(user_request).await.unwrap();

    // Get default account
    let accounts = account_service
        .get_accounts_by_user_id(user.id)
        .await
        .unwrap();
    let account = &accounts[0];

    // Test update balance (positive)
    let deposit_amount = Decimal::from(100);
    let update_result = account_service
        .update_balance(account.id, deposit_amount)
        .await;
    assert!(
        update_result.is_ok(),
        "Balance update failed: {:?}",
        update_result.err()
    );

    let updated_account = update_result.unwrap();
    assert_eq!(updated_account.balance, deposit_amount);

    // Test update balance (negative with sufficient funds)
    let withdrawal_amount = Decimal::from(50);
    let update_result = account_service
        .update_balance(account.id, -withdrawal_amount)
        .await;
    assert!(
        update_result.is_ok(),
        "Balance update failed: {:?}",
        update_result.err()
    );

    let updated_account = update_result.unwrap();
    assert_eq!(updated_account.balance, deposit_amount - withdrawal_amount);

    // Test update balance (negative with insufficient funds)
    let big_withdrawal = Decimal::from(1000);
    let update_result = account_service
        .update_balance(account.id, -big_withdrawal)
        .await;
    assert!(
        update_result.is_err(),
        "Balance update should fail with insufficient funds"
    );

    // Clean up test environment
    teardown(&db_url).await;
}

#[tokio::test]
async fn test_account_creation() {
    // Set up test environment
    let (pool, db_url) = setup().await;

    // Create services
    let user_service = create_user_service(pool.clone());
    let account_service = create_account_service(pool.clone());

    // Create a test user
    let user_request = CreateUserRequest {
        username: "accuser1".to_string(),
        email: "acc1@example.com".to_string(),
        password: "securepassword".to_string(),
        first_name: Some("Account".to_string()),
        last_name: Some("User".to_string()),
    };

    let user = user_service.create_user(user_request).await.unwrap();

    // Verify default account was created
    let accounts = account_service
        .get_accounts_by_user_id(user.id)
        .await
        .unwrap();
    assert_eq!(accounts.len(), 1, "User should have one default account");

    let default_account = &accounts[0];
    assert_eq!(default_account.user_id, user.id);
    assert_eq!(default_account.balance, Decimal::from(0));
    assert_eq!(default_account.currency, "USD");

    // Create a new account for the user with a different currency
    let new_account = account_service
        .create_account(user.id, "EUR".to_string())
        .await
        .unwrap();

    // Verify the new account was created
    assert_eq!(new_account.user_id, user.id);
    assert_eq!(new_account.balance, Decimal::from(0));
    assert_eq!(new_account.currency, "EUR");

    // Check both accounts are returned
    let updated_accounts = account_service
        .get_accounts_by_user_id(user.id)
        .await
        .unwrap();
    assert_eq!(
        updated_accounts.len(),
        2,
        "User should now have two accounts"
    );

    // Clean up test environment
    teardown(&db_url).await;
}

#[tokio::test]
async fn test_account_balance_operations() {
    // Set up test environment
    let (pool, db_url) = setup().await;

    // Create services
    let user_service = create_user_service(pool.clone());
    let account_service = create_account_service(pool.clone());

    // Create a test user
    let user_request = CreateUserRequest {
        username: "accuser2".to_string(),
        email: "acc2@example.com".to_string(),
        password: "securepassword".to_string(),
        first_name: Some("Account".to_string()),
        last_name: Some("User".to_string()),
    };

    let user = user_service.create_user(user_request).await.unwrap();

    // Get default account
    let accounts = account_service
        .get_accounts_by_user_id(user.id)
        .await
        .unwrap();
    let account = &accounts[0];

    // Update balance positively
    let updated_account = account_service
        .update_balance(account.id, Decimal::from(100))
        .await
        .unwrap();
    assert_eq!(updated_account.balance, Decimal::from(100));

    // Verify the balance is persisted
    let retrieved_account = account_service.get_account_by_id(account.id).await.unwrap();
    assert_eq!(retrieved_account.balance, Decimal::from(100));

    // Update balance again
    let updated_account = account_service
        .update_balance(account.id, Decimal::from(50))
        .await
        .unwrap();
    assert_eq!(updated_account.balance, Decimal::from(150)); // 100 + 50

    // Try to withdraw more than the balance
    let excess_withdrawal = account_service
        .update_balance(account.id, Decimal::from(-200))
        .await;
    assert!(
        excess_withdrawal.is_err(),
        "Should not allow negative balance"
    );

    // Withdraw an allowable amount
    let updated_account = account_service
        .update_balance(account.id, Decimal::from(-75))
        .await
        .unwrap();
    assert_eq!(updated_account.balance, Decimal::from(75)); // 150 - 75

    // Clean up test environment
    teardown(&db_url).await;
}

#[tokio::test]
async fn test_retrieve_non_existent_account() {
    // Set up test environment
    let (pool, db_url) = setup().await;

    // Create account service
    let account_service = create_account_service(pool.clone());

    // Try to retrieve non-existent account
    let random_id = uuid::Uuid::new_v4();
    let result = account_service.get_account_by_id(random_id).await;

    // Verify the error
    assert!(
        result.is_err(),
        "Should return error for non-existent account"
    );

    // Clean up test environment
    teardown(&db_url).await;
}
