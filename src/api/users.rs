use crate::middleware::auth::AuthUser;
use crate::models::user::{CreateUserRequest, LoginRequest, UserResponse};
use crate::services::user_service::UserService;
use crate::utils::error::AppError;
use crate::utils::response::ApiResponse;
use axum::{
    extract::{Json, State},
    routing::{get, post, put},
    Extension, Router,
};
use std::sync::Arc;
use validator::Validate;

pub fn user_routes(user_service: Arc<UserService>) -> Router {
    Router::new()
        .route("/register", post(register_user))
        .route("/login", post(login))
        .route("/me", get(get_current_user))
        .route("/profile", put(update_profile))
        .with_state(user_service)
}

async fn register_user(
    State(user_service): State<Arc<UserService>>,
    Json(user_data): Json<CreateUserRequest>,
) -> Result<Json<ApiResponse<UserResponse>>, AppError> {
    // Validate request data
    user_data
        .validate()
        .map_err(|e| AppError::Validation(format!("Invalid user data: {}", e)))?;

    // Create user
    let user = user_service.create_user(user_data).await?;

    // Return success response
    Ok(Json(ApiResponse::success(
        "User registered successfully",
        user,
    )))
}

async fn login(
    State(user_service): State<Arc<UserService>>,
    Json(login_data): Json<LoginRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    // Validate request data
    login_data
        .validate()
        .map_err(|e| AppError::Validation(format!("Invalid login data: {}", e)))?;

    // Authenticate user
    let login_response = user_service.login(login_data).await?;

    // Return success response with token and user data
    Ok(Json(ApiResponse::success(
        "Login successful",
        serde_json::json!({
            "token": login_response.token,
            "user": login_response.user
        }),
    )))
}

async fn get_current_user(
    Extension(auth_user): Extension<AuthUser>,
    State(user_service): State<Arc<UserService>>,
) -> Result<Json<ApiResponse<UserResponse>>, AppError> {
    // Get user by ID from auth context
    let user = user_service.get_user_by_id(auth_user.user_id).await?;

    // Return success response
    Ok(Json(ApiResponse::success("User profile retrieved", user)))
}

async fn update_profile(
    Extension(auth_user): Extension<AuthUser>,
    State(user_service): State<Arc<UserService>>,
    Json(profile_data): Json<serde_json::Value>,
) -> Result<Json<ApiResponse<UserResponse>>, AppError> {
    // Extract fields from JSON data
    let first_name = profile_data
        .get("first_name")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let last_name = profile_data
        .get("last_name")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    // Update user profile
    let user = user_service
        .update_user(auth_user.user_id, first_name, last_name)
        .await?;

    // Return success response
    Ok(Json(ApiResponse::success(
        "Profile updated successfully",
        user,
    )))
}
