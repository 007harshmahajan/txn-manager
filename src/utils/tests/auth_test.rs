#[cfg(test)]
mod tests {
    use crate::utils::auth::{generate_jwt, hash_password, validate_jwt, verify_password};
    use crate::utils::error::AppError;
    use uuid::Uuid;

    #[test]
    fn test_hash_and_verify_password() {
        // Test password hashing
        let password = "secure_password";
        let hash_result = hash_password(password);
        assert!(hash_result.is_ok());
        
        let hash = hash_result.unwrap();
        
        // Verify correct password
        let verify_result = verify_password(password, &hash);
        assert!(verify_result.is_ok());
        assert!(verify_result.unwrap());
        
        // Verify incorrect password
        let verify_result = verify_password("wrong_password", &hash);
        assert!(verify_result.is_ok());
        assert!(!verify_result.unwrap());
    }

    #[test]
    fn test_jwt_generation_and_validation() {
        let user_id = Uuid::new_v4();
        let username = "testuser";
        let secret = "test_secret_key";
        
        // Generate JWT
        let jwt_result = generate_jwt(user_id, username, secret);
        assert!(jwt_result.is_ok());
        
        let token = jwt_result.unwrap();
        
        // Validate JWT
        let validate_result = validate_jwt(&token, secret);
        assert!(validate_result.is_ok());
        
        let token_data = validate_result.unwrap();
        assert_eq!(token_data.claims.sub, user_id.to_string());
        assert_eq!(token_data.claims.username, username);
        
        // Validate with wrong secret
        let validate_result = validate_jwt(&token, "wrong_secret");
        assert!(validate_result.is_err());
    }
} 