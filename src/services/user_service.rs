use crate::models::user::{CreateUserRequest, LoginRequest, LoginResponse, User, UserResponse};
use crate::utils::auth::{generate_jwt, hash_password, verify_password};
use crate::utils::error::AppError;
use sqlx::PgPool;
use uuid::Uuid;

pub struct UserService {
    pool: PgPool,
    jwt_secret: String,
}

impl UserService {
    pub fn new(pool: PgPool, jwt_secret: String) -> Self {
        Self { pool, jwt_secret }
    }

    pub async fn create_user(
        &self,
        user_data: CreateUserRequest,
    ) -> Result<UserResponse, AppError> {
        // Check if user exists
        let existing_user = sqlx::query!(
            r#"
            SELECT id FROM users WHERE username = $1 OR email = $2
            "#,
            user_data.username,
            user_data.email
        )
        .fetch_optional(&self.pool)
        .await?;

        if existing_user.is_some() {
            return Err(AppError::Conflict(
                "Username or email already exists".to_string(),
            ));
        }

        // Hash password
        let password_hash = hash_password(&user_data.password)?;

        // Generate UUID
        let id = Uuid::new_v4();

        // Insert user
        let user = sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (id, username, email, password_hash, first_name, last_name)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, username, email, password_hash, first_name, last_name, created_at, updated_at
            "#,
            id,
            user_data.username,
            user_data.email,
            password_hash,
            user_data.first_name,
            user_data.last_name
        )
        .fetch_one(&self.pool)
        .await?;

        // Create default account for user
        let account_id = Uuid::new_v4();
        sqlx::query!(
            r#"
            INSERT INTO accounts (id, user_id, balance, currency)
            VALUES ($1, $2, 0, 'USD')
            "#,
            account_id,
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(UserResponse::from(user))
    }

    pub async fn login(&self, login_data: LoginRequest) -> Result<LoginResponse, AppError> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT id, username, email, password_hash, first_name, last_name, created_at, updated_at
            FROM users WHERE username = $1
            "#,
            login_data.username
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AppError::Auth("Invalid username or password".to_string()))?;

        // Verify password
        let is_valid = verify_password(&login_data.password, &user.password_hash)?;
        if !is_valid {
            return Err(AppError::Auth("Invalid username or password".to_string()));
        }

        // Generate JWT
        let token = generate_jwt(user.id, &user.username, &self.jwt_secret)?;

        Ok(LoginResponse {
            token,
            user: UserResponse::from(user),
        })
    }

    pub async fn get_user_by_id(&self, id: Uuid) -> Result<UserResponse, AppError> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT id, username, email, password_hash, first_name, last_name, created_at, updated_at
            FROM users WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("User with ID {} not found", id)))?;

        Ok(UserResponse::from(user))
    }

    pub async fn update_user(
        &self,
        id: Uuid,
        first_name: Option<String>,
        last_name: Option<String>,
    ) -> Result<UserResponse, AppError> {
        // Check if user exists
        let existing_user = sqlx::query!(
            r#"
            SELECT id FROM users WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        if existing_user.is_none() {
            return Err(AppError::NotFound(format!("User with ID {} not found", id)));
        }

        // Update user
        let user = sqlx::query_as!(
            User,
            r#"
            UPDATE users
            SET first_name = COALESCE($2, first_name),
                last_name = COALESCE($3, last_name),
                updated_at = NOW()
            WHERE id = $1
            RETURNING id, username, email, password_hash, first_name, last_name, created_at, updated_at
            "#,
            id,
            first_name,
            last_name
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(UserResponse::from(user))
    }
}
