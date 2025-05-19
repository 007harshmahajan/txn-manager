use crate::integration::setup::{
    create_account_service, create_transaction_service, create_user_service, setup, teardown,
};
use rust_decimal::Decimal;
use txn_manager::{CreateUserRequest, DepositRequest, TransferRequest, WithdrawalRequest};

#[tokio::test]
async fn test_deposit_transaction() {
    // Set up test environment
    let (pool, db_url) = setup().await;

    // Create services
    let user_service = create_user_service(pool.clone());
    let account_service = create_account_service(pool.clone());
    let transaction_service = create_transaction_service(pool.clone());

    // Create a test user
    let user_request = CreateUserRequest {
        username: "txnuser1".to_string(),
        email: "txn1@example.com".to_string(),
        password: "securepassword".to_string(),
        first_name: Some("Txn".to_string()),
        last_name: Some("User".to_string()),
    };

    let user = user_service.create_user(user_request).await.unwrap();

    // Get default account
    let accounts = account_service
        .get_accounts_by_user_id(user.id)
        .await
        .unwrap();
    let account = &accounts[0];

    // Test deposit transaction
    let deposit_request = DepositRequest {
        account_id: account.id,
        amount: Decimal::from(100),
        description: Some("Test deposit".to_string()),
    };

    let deposit_result = transaction_service.process_deposit(deposit_request).await;
    assert!(
        deposit_result.is_ok(),
        "Deposit failed: {:?}",
        deposit_result.err()
    );

    let deposit_response = deposit_result.unwrap();
    assert_eq!(deposit_response.receiver_account_id, Some(account.id));
    assert_eq!(deposit_response.amount, Decimal::from(100));
    assert_eq!(deposit_response.transaction_type, "DEPOSIT");
    assert_eq!(deposit_response.status, "COMPLETED");

    // Verify account balance was updated
    let updated_account = account_service.get_account_by_id(account.id).await.unwrap();
    assert_eq!(updated_account.balance, Decimal::from(100));

    // Clean up test environment
    teardown(&db_url).await;
}

#[tokio::test]
async fn test_withdrawal_transaction() {
    // Set up test environment
    let (pool, db_url) = setup().await;

    // Create services
    let user_service = create_user_service(pool.clone());
    let account_service = create_account_service(pool.clone());
    let transaction_service = create_transaction_service(pool.clone());

    // Create a test user
    let user_request = CreateUserRequest {
        username: "txnuser2".to_string(),
        email: "txn2@example.com".to_string(),
        password: "securepassword".to_string(),
        first_name: Some("Txn".to_string()),
        last_name: Some("User".to_string()),
    };

    let user = user_service.create_user(user_request).await.unwrap();

    // Get default account
    let accounts = account_service
        .get_accounts_by_user_id(user.id)
        .await
        .unwrap();
    let account = &accounts[0];

    // First deposit some money
    let deposit_request = DepositRequest {
        account_id: account.id,
        amount: Decimal::from(200),
        description: Some("Initial deposit".to_string()),
    };

    transaction_service
        .process_deposit(deposit_request)
        .await
        .unwrap();

    // Test withdrawal transaction
    let withdrawal_request = WithdrawalRequest {
        account_id: account.id,
        amount: Decimal::from(50),
        description: Some("Test withdrawal".to_string()),
    };

    let withdrawal_result = transaction_service
        .process_withdrawal(withdrawal_request)
        .await;
    assert!(
        withdrawal_result.is_ok(),
        "Withdrawal failed: {:?}",
        withdrawal_result.err()
    );

    let withdrawal_response = withdrawal_result.unwrap();
    assert_eq!(withdrawal_response.sender_account_id, Some(account.id));
    assert_eq!(withdrawal_response.amount, Decimal::from(50));
    assert_eq!(withdrawal_response.transaction_type, "WITHDRAWAL");
    assert_eq!(withdrawal_response.status, "COMPLETED");

    // Verify account balance was updated
    let updated_account = account_service.get_account_by_id(account.id).await.unwrap();
    assert_eq!(updated_account.balance, Decimal::from(150)); // 200 - 50

    // Test withdrawal with insufficient funds
    let withdrawal_request = WithdrawalRequest {
        account_id: account.id,
        amount: Decimal::from(1000),
        description: Some("Test excessive withdrawal".to_string()),
    };

    let withdrawal_result = transaction_service
        .process_withdrawal(withdrawal_request)
        .await;
    assert!(
        withdrawal_result.is_err(),
        "Withdrawal with insufficient funds should fail"
    );

    // Clean up test environment
    teardown(&db_url).await;
}

#[tokio::test]
async fn test_transfer_transaction() {
    // Set up test environment
    let (pool, db_url) = setup().await;

    // Create services
    let user_service = create_user_service(pool.clone());
    let account_service = create_account_service(pool.clone());
    let transaction_service = create_transaction_service(pool.clone());

    // Create sender user
    let sender_request = CreateUserRequest {
        username: "sender".to_string(),
        email: "sender@example.com".to_string(),
        password: "securepassword".to_string(),
        first_name: Some("Sender".to_string()),
        last_name: Some("User".to_string()),
    };

    let sender = user_service.create_user(sender_request).await.unwrap();

    // Create receiver user
    let receiver_request = CreateUserRequest {
        username: "receiver".to_string(),
        email: "receiver@example.com".to_string(),
        password: "securepassword".to_string(),
        first_name: Some("Receiver".to_string()),
        last_name: Some("User".to_string()),
    };

    let receiver = user_service.create_user(receiver_request).await.unwrap();

    // Get sender and receiver accounts
    let sender_accounts = account_service
        .get_accounts_by_user_id(sender.id)
        .await
        .unwrap();
    let sender_account = &sender_accounts[0];

    let receiver_accounts = account_service
        .get_accounts_by_user_id(receiver.id)
        .await
        .unwrap();
    let receiver_account = &receiver_accounts[0];

    // Fund sender account
    let deposit_request = DepositRequest {
        account_id: sender_account.id,
        amount: Decimal::from(500),
        description: Some("Initial funding".to_string()),
    };

    transaction_service
        .process_deposit(deposit_request)
        .await
        .unwrap();

    // Test transfer transaction
    let transfer_request = TransferRequest {
        sender_account_id: sender_account.id,
        receiver_account_id: receiver_account.id,
        amount: Decimal::from(200),
        description: Some("Test transfer".to_string()),
    };

    let transfer_result = transaction_service.process_transfer(transfer_request).await;
    assert!(
        transfer_result.is_ok(),
        "Transfer failed: {:?}",
        transfer_result.err()
    );

    let transfer_response = transfer_result.unwrap();
    assert_eq!(transfer_response.sender_account_id, Some(sender_account.id));
    assert_eq!(
        transfer_response.receiver_account_id,
        Some(receiver_account.id)
    );
    assert_eq!(transfer_response.amount, Decimal::from(200));
    assert_eq!(transfer_response.transaction_type, "TRANSFER");
    assert_eq!(transfer_response.status, "COMPLETED");

    // Verify account balances were updated
    let updated_sender = account_service
        .get_account_by_id(sender_account.id)
        .await
        .unwrap();
    let updated_receiver = account_service
        .get_account_by_id(receiver_account.id)
        .await
        .unwrap();

    assert_eq!(updated_sender.balance, Decimal::from(300)); // 500 - 200
    assert_eq!(updated_receiver.balance, Decimal::from(200));

    // Test transfer with insufficient funds
    let transfer_request = TransferRequest {
        sender_account_id: sender_account.id,
        receiver_account_id: receiver_account.id,
        amount: Decimal::from(1000),
        description: Some("Test excessive transfer".to_string()),
    };

    let transfer_result = transaction_service.process_transfer(transfer_request).await;
    assert!(
        transfer_result.is_err(),
        "Transfer with insufficient funds should fail"
    );

    // Clean up test environment
    teardown(&db_url).await;
}
