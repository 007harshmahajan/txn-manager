use crate::middleware::auth::AuthUser;
use crate::models::account::AccountResponse;
use crate::services::account_service::AccountService;
use crate::utils::error::AppError;
use crate::utils::response::ApiResponse;
use axum::{
    extract::{Json, Path, State},
    routing::{get, post},
    Extension, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

pub fn account_routes(account_service: Arc<AccountService>) -> Router {
    Router::new()
        .route("/", get(get_user_accounts))
        .route("/", post(create_account))
        .route("/:id", get(get_account))
        .with_state(account_service)
}

#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct CreateAccountRequest {
    #[validate(length(min = 3, max = 3, message = "Currency must be a 3-letter code"))]
    pub currency: String,
}

async fn get_user_accounts(
    Extension(auth_user): Extension<AuthUser>,
    State(account_service): State<Arc<AccountService>>,
) -> Result<Json<ApiResponse<Vec<AccountResponse>>>, AppError> {
    // Get all accounts for the authenticated user
    let accounts = account_service
        .get_accounts_by_user_id(auth_user.user_id)
        .await?;

    // Return success response
    Ok(Json(ApiResponse::success(
        "Accounts retrieved successfully",
        accounts,
    )))
}

async fn get_account(
    Extension(auth_user): Extension<AuthUser>,
    State(account_service): State<Arc<AccountService>>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<AccountResponse>>, AppError> {
    // Get the account
    let account = account_service.get_account_by_id(id).await?;

    // Verify the account belongs to the authenticated user
    if account.user_id != auth_user.user_id {
        return Err(AppError::Forbidden(
            "You don't have permission to access this account".to_string(),
        ));
    }

    // Return success response
    Ok(Json(ApiResponse::success(
        "Account retrieved successfully",
        account,
    )))
}

async fn create_account(
    Extension(auth_user): Extension<AuthUser>,
    State(account_service): State<Arc<AccountService>>,
    Json(request): Json<CreateAccountRequest>,
) -> Result<Json<ApiResponse<AccountResponse>>, AppError> {
    // Validate request data
    request
        .validate()
        .map_err(|e| AppError::Validation(format!("Invalid account data: {}", e)))?;

    // Create new account for the authenticated user
    let account = account_service
        .create_account(auth_user.user_id, request.currency)
        .await?;

    // Return success response
    Ok(Json(ApiResponse::success(
        "Account created successfully",
        account,
    )))
}
