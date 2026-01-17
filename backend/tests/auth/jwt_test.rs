#[cfg(test)]
mod jwt_test {
    use auth_service::jwt::{JwtService, JwtConfig};

    #[test]
    fn test_jwt_service_generate_access_token() {
        let config = JwtConfig {
            secret: "test_secret_key".to_string(),
            access_expiry: 3600,
            refresh_expiry: 86400,
        };

        let jwt_service = JwtService::new(config);

        let result = jwt_service.generate_access_token(
            "user_id_123",
            "test@example.com",
            "user",
        );

        assert!(result.is_ok());

        let token = result.unwrap();
        assert!(!token.is_empty());
    }

    #[test]
    fn test_jwt_service_generate_refresh_token() {
        let config = JwtConfig {
            secret: "test_secret_key".to_string(),
            access_expiry: 3600,
            refresh_expiry: 86400,
        };

        let jwt_service = JwtService::new(config);

        let result = jwt_service.generate_refresh_token("user_id_123");

        assert!(result.is_ok());

        let token = result.unwrap();
        assert!(!token.is_empty());
    }

    #[test]
    fn test_jwt_service_validate_token_valid() {
        let config = JwtConfig {
            secret: "test_secret_key".to_string(),
            access_expiry: 3600,
            refresh_expiry: 86400,
        };

        let jwt_service = JwtService::new(config);

        let token = jwt_service
            .generate_access_token("user_id_123", "test@example.com", "user")
            .unwrap();

        let result = jwt_service.validate_token(&token);

        assert!(result.is_ok());

        let claims = result.unwrap();
        assert_eq!(claims.user_id, "user_id_123");
        assert_eq!(claims.email, "test@example.com");
        assert_eq!(claims.role, "user");
    }

    #[test]
    fn test_jwt_service_validate_token_invalid() {
        let config = JwtConfig {
            secret: "test_secret_key".to_string(),
            access_expiry: 3600,
            refresh_expiry: 86400,
        };

        let jwt_service = JwtService::new(config);

        let result = jwt_service.validate_token("invalid.token.here");

        assert!(result.is_err());
    }

    #[test]
    fn test_jwt_service_validate_token_wrong_secret() {
        let config = JwtConfig {
            secret: "correct_secret".to_string(),
            access_expiry: 3600,
            refresh_expiry: 86400,
        };

        let jwt_service = JwtService::new(config);

        let token = jwt_service
            .generate_access_token("user_id_123", "test@example.com", "user")
            .unwrap();

        let wrong_config = JwtConfig {
            secret: "wrong_secret".to_string(),
            access_expiry: 3600,
            refresh_expiry: 86400,
        };

        let wrong_jwt_service = JwtService::new(wrong_config);

        let result = wrong_jwt_service.validate_token(&token);

        assert!(result.is_err());
    }

    #[test]
    fn test_extract_token_from_header() {
        let valid_header = "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6ImVzZXJuYW1lIjoidXwiZW1haWwiOiJ0OTk5fQ.Sfl";
        let invalid_header = "Invalid token";

        assert!(JwtService::extract_token_from_header(valid_header).is_some());
        assert!(JwtService::extract_token_from_header(invalid_header).is_none());
    }
}
