#[cfg(test)]
mod refresh_token_test {
    use auth_service::jwt::{JwtService, JwtConfig};

    #[test]
    fn test_refresh_token_generation() {
        let config = JwtConfig {
            secret: "test_secret_key".to_string(),
            access_expiry: 3600,
            refresh_expiry: 86400,
        };

        let jwt_service = JwtService::new(config);

        let user_id = "user_id_123";
        let refresh_token = jwt_service.generate_refresh_token(user_id);

        assert!(refresh_token.is_ok());
        assert!(!refresh_token.unwrap().is_empty());
    }

    #[test]
    fn test_refresh_token_rotation() {
        let config = JwtConfig {
            secret: "test_secret_key".to_string(),
            access_expiry: 3600,
            refresh_expiry: 86400,
        };

        let jwt_service = JwtService::new(config);

        let user_id = "user_id_123";

        let first_token = jwt_service.generate_refresh_token(user_id).unwrap();

        let claims = jwt_service.validate_token(&first_token).unwrap();
        let access_token_1 = jwt_service.generate_access_token(
            &claims.user_id,
            &claims.email,
            &claims.role,
        ).unwrap();

        let second_token = jwt_service.generate_refresh_token(user_id).unwrap();

        assert_ne!(first_token, second_token);
        assert_ne!(access_token_1, second_token);
    }

    #[test]
    fn test_refresh_token_expiration() {
        let config = JwtConfig {
            secret: "test_secret_key".to_string(),
            access_expiry: 3600,
            refresh_expiry: 86400,
        };

        let jwt_service = JwtService::new(config);

        let token = jwt_service
            .generate_refresh_token("user_id_123")
            .unwrap();

        let claims = jwt_service.validate_token(&token).unwrap();

        use chrono::{Utc, Duration};
        let now = Utc::now();
        let expected_expiry = now + Duration::seconds(86400);
        let actual_expiry = chrono::DateTime::<Utc>::from_timestamp(claims.exp.try_into().unwrap(), 0).unwrap();

        let diff = (actual_expiry - expected_expiry).num_seconds().abs();
        assert!(diff < 10);
    }

    #[test]
    fn test_refresh_token_different_from_access_token() {
        let config = JwtConfig {
            secret: "test_secret_key".to_string(),
            access_expiry: 3600,
            refresh_expiry: 86400,
        };

        let jwt_service = JwtService::new(config);

        let user_id = "user_id_123";
        let email = "test@example.com";

        let access_token = jwt_service
            .generate_access_token(user_id, email, "user")
            .unwrap();

        let refresh_token = jwt_service.generate_refresh_token(user_id).unwrap();

        let _access_claims = jwt_service.validate_token(&access_token).unwrap();
        let refresh_claims = jwt_service.validate_token(&refresh_token).unwrap();

        assert!(refresh_claims.email.is_empty());
        assert!(refresh_claims.role.is_empty());
    }
}
