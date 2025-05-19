use crate::middleware::auth::AuthUser;
use crate::models::transaction::{
    CreateTransactionRequest, DepositRequest, TransactionResponse, TransferRequest,
    WithdrawalRequest,
};
use crate::services::{account_service::AccountService, transaction_service::TransactionService};
use crate::utils::error::AppError;
use crate::utils::response::ApiResponse;
use axum::{
    extract::{Json, Path, Query, State},
    routing::{get, post},
    Extension, Router,
};
use serde::{Deserialize};
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

pub fn transaction_routes(
    transaction_service: Arc<TransactionService>,
    account_service: Arc<AccountService>,
) -> Router {
    Router::new()
        .route("/", post(create_transaction))
        .route("/:id", get(get_transaction))
        .route("/transfer", post(transfer))
        .route("/deposit", post(deposit))
        .route("/withdrawal", post(withdrawal))
        .route("/account/:id", get(get_account_transactions))
        .with_state((transaction_service, account_service))
}

#[derive(Debug, Deserialize)]
pub struct TransactionQueryParams {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

async fn get_transaction(
    Extension(auth_user): Extension<AuthUser>,
    State((transaction_service, account_service)): State<(
        Arc<TransactionService>,
        Arc<AccountService>,
    )>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<TransactionResponse>>, AppError> {
    // Get the transaction
    let transaction = transaction_service.get_transaction_by_id(id).await?;

    // Verify the transaction involves an account owned by the authenticated user
    if let Some(sender_id) = transaction.sender_account_id {
        let sender_account = account_service.get_account_by_id(sender_id).await?;
        if sender_account.user_id == auth_user.user_id {
            return Ok(Json(ApiResponse::success(
                "Transaction retrieved successfully",
                transaction,
            )));
        }
    }

    if let Some(receiver_id) = transaction.receiver_account_id {
        let receiver_account = account_service.get_account_by_id(receiver_id).await?;
        if receiver_account.user_id == auth_user.user_id {
            return Ok(Json(ApiResponse::success(
                "Transaction retrieved successfully",
                transaction,
            )));
        }
    }

    // If we get here, the user doesn't own any accounts involved in the transaction
    Err(AppError::Forbidden(
        "You don't have permission to access this transaction".to_string(),
    ))
}

async fn create_transaction(
    Extension(auth_user): Extension<AuthUser>,
    State((transaction_service, account_service)): State<(
        Arc<TransactionService>,
        Arc<AccountService>,
    )>,
    Json(request): Json<CreateTransactionRequest>,
) -> Result<Json<ApiResponse<TransactionResponse>>, AppError> {
    // Validate request data
    request
        .validate()
        .map_err(|e| AppError::Validation(format!("Invalid transaction data: {}", e)))?;

    // Verify account ownership for sender or receiver
    if let Some(sender_id) = request.sender_account_id {
        let sender_account = account_service.get_account_by_id(sender_id).await?;
        if sender_account.user_id != auth_user.user_id {
            return Err(AppError::Forbidden(
                "You don't have permission to use this sender account".to_string(),
            ));
        }
    }

    if let Some(receiver_id) = request.receiver_account_id {
        let receiver_account = account_service.get_account_by_id(receiver_id).await?;
        if receiver_account.user_id != auth_user.user_id {
            return Err(AppError::Forbidden(
                "You don't have permission to use this receiver account".to_string(),
            ));
        }
    }

    // Create the transaction
    let transaction = transaction_service.create_transaction(request).await?;

    // Return success response
    Ok(Json(ApiResponse::success(
        "Transaction created successfully",
        transaction,
    )))
}

async fn transfer(
    Extension(auth_user): Extension<AuthUser>,
    State((transaction_service, account_service)): State<(
        Arc<TransactionService>,
        Arc<AccountService>,
    )>,
    Json(request): Json<TransferRequest>,
) -> Result<Json<ApiResponse<TransactionResponse>>, AppError> {
    // Validate request data
    request
        .validate()
        .map_err(|e| AppError::Validation(format!("Invalid transfer data: {}", e)))?;

    // Verify sender account ownership
    let sender_account = account_service
        .get_account_by_id(request.sender_account_id)
        .await?;
    if sender_account.user_id != auth_user.user_id {
        return Err(AppError::Forbidden(
            "You don't have permission to use this sender account".to_string(),
        ));
    }

    // Process transfer
    let transaction = transaction_service.process_transfer(request).await?;

    // Return success response
    Ok(Json(ApiResponse::success(
        "Transfer successful",
        transaction,
    )))
}

async fn deposit(
    Extension(auth_user): Extension<AuthUser>,
    State((transaction_service, account_service)): State<(
        Arc<TransactionService>,
        Arc<AccountService>,
    )>,
    Json(request): Json<DepositRequest>,
) -> Result<Json<ApiResponse<TransactionResponse>>, AppError> {
    // Validate request data
    request
        .validate()
        .map_err(|e| AppError::Validation(format!("Invalid deposit data: {}", e)))?;

    // Verify account ownership
    let account = account_service
        .get_account_by_id(request.account_id)
        .await?;
    if account.user_id != auth_user.user_id {
        return Err(AppError::Forbidden(
            "You don't have permission to use this account".to_string(),
        ));
    }

    // Process deposit
    let transaction = transaction_service.process_deposit(request).await?;

    // Return success response
    Ok(Json(ApiResponse::success(
        "Deposit successful",
        transaction,
    )))
}

async fn withdrawal(
    Extension(auth_user): Extension<AuthUser>,
    State((transaction_service, account_service)): State<(
        Arc<TransactionService>,
        Arc<AccountService>,
    )>,
    Json(request): Json<WithdrawalRequest>,
) -> Result<Json<ApiResponse<TransactionResponse>>, AppError> {
    // Validate request data
    request
        .validate()
        .map_err(|e| AppError::Validation(format!("Invalid withdrawal data: {}", e)))?;

    // Verify account ownership
    let account = account_service
        .get_account_by_id(request.account_id)
        .await?;
    if account.user_id != auth_user.user_id {
        return Err(AppError::Forbidden(
            "You don't have permission to use this account".to_string(),
        ));
    }

    // Process withdrawal
    let transaction = transaction_service.process_withdrawal(request).await?;

    // Return success response
    Ok(Json(ApiResponse::success(
        "Withdrawal successful",
        transaction,
    )))
}

async fn get_account_transactions(
    Extension(auth_user): Extension<AuthUser>,
    State((transaction_service, account_service)): State<(
        Arc<TransactionService>,
        Arc<AccountService>,
    )>,
    Path(id): Path<Uuid>,
    Query(params): Query<TransactionQueryParams>,
) -> Result<Json<ApiResponse<Vec<TransactionResponse>>>, AppError> {
    // Verify account ownership
    let account = account_service.get_account_by_id(id).await?;
    if account.user_id != auth_user.user_id {
        return Err(AppError::Forbidden(
            "You don't have permission to access this account".to_string(),
        ));
    }

    // Get transactions for this account
    let transactions = transaction_service
        .get_transactions_by_account_id(id, params.limit, params.offset)
        .await?;

    // Return success response
    Ok(Json(ApiResponse::success(
        "Transactions retrieved successfully",
        transactions,
    )))
}
