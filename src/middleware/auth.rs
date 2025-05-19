use crate::utils::auth::validate_jwt;
use crate::utils::error::AppError;
use axum::extract::FromRef;
use axum::http::header;
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

/// Represents an authenticated user
#[derive(Clone, Debug)]
pub struct AuthUser {
    /// The unique identifier of the user
    pub user_id: Uuid,
    /// The username of the authenticated user
    pub username: String,
}

pub async fn auth_middleware<AppState>(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError>
where
    String: FromRef<AppState>,
{
    let jwt_secret = String::from_ref(&state);

    // Extract token from Authorization header
    let token = extract_token_from_header(&request)?;

    // Validate token
    let token_data = validate_jwt(&token, &jwt_secret)?;

    // Create AuthUser from claims
    let auth_user = AuthUser {
        user_id: Uuid::parse_str(&token_data.claims.sub)
            .map_err(|_| AppError::Auth("Invalid user ID in token".to_string()))?,
        username: token_data.claims.username,
    };

    // Set auth_user as request extension
    request.extensions_mut().insert(auth_user);

    // Continue with the request
    Ok(next.run(request).await)
}

fn extract_token_from_header(request: &Request) -> Result<String, AppError> {
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .ok_or_else(|| AppError::Auth("Missing authorization header".to_string()))?
        .to_str()
        .map_err(|_| AppError::Auth("Invalid authorization header".to_string()))?;

    if !auth_header.starts_with("Bearer ") {
        return Err(AppError::Auth(
            "Authorization header must be Bearer token".to_string(),
        ));
    }

    Ok(auth_header[7..].to_string())
}
