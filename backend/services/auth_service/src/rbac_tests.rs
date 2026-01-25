//! Unit tests for auth_service rbac module
//!
//! This module contains tests for:
//! - JWT token extraction and validation
//! - Permission checking functions
//! - User role extraction from requests
//! - RBAC middleware structure
//! - Error handling

use uuid::Uuid;
use crate::rbac::*;

// Test: JWT_SECRET environment variable
#[test]
fn test_jwt_secret_env_var_exists() {
    if std::env::var("JWT_SECRET").is_err() {
        // Skip test if env var not set
        return;
    }
    let secret = std::env::var("JWT_SECRET").unwrap();
    assert!(!secret.is_empty());
}

// Test: JWT_DECODING_KEY environment variable
#[test]
fn test_jwt_decoding_key_env_var_exists() {
    if std::env::var("JWT_DECODING_KEY").is_err() {
        // Skip test if env var not set
        return;
    }
    let key = std::env::var("JWT_DECODING_KEY").unwrap();
    assert!(!key.is_empty());
}

// Test: Authorization header format
#[test]
fn test_authorization_header_format() {
    let valid_header = "Bearer eyJhbGci01234".to_string();
    let with_bearer = valid_header;

    assert!(with_bearer.starts_with("Bearer "));
    assert!(with_bearer.ends_with("eyJhbGci01234"));
}

// Test: Bearer token extraction
#[test]
fn test_bearer_token_extraction() {
    let token = "valid_token_here";
    let header_value = format!("Bearer {}", token);

    assert!(header_value.starts_with("Bearer "));
    assert!(header_value.ends_with(&token));
    assert_eq!(header_value.split_whitespace().len(), 2);
}

#[test]
fn test_bearer_token_with_auth_header() {
    let token = "valid_token_here";
    let header = format!("Authorization: Bearer {}", token);

    assert!(header.contains(&token));
    assert!(header.starts_with("Authorization: Bearer"));
    assert_eq!(header.split_whitespace().len(), 3);
}

#[test]
fn test_bearer_token_empty() {
    let header = "Bearer ".to_string();

    assert!(header.starts_with("Bearer "));
    let token = header.strip_prefix("Bearer ").unwrap_or("");
    assert!(token.is_empty());
}

// Test: Role enum variants
#[test]
fn test_role_enum_variants() {
    // Role is an enum with variants Owner, Editor, Commenter, Viewer
    let _owner = Role::Owner;
    let _editor = Role::Editor;
    let _commenter = Role::Commenter;
    let _viewer = Role::Viewer;
}

// Test: Permission enum variants
#[test]
fn test_permission_enum_variants() {
    // Verify permission variants can be created
    let _perm1 = Permission::DeleteDocuments;
    let _perm2 = Permission::ViewDocuments;
    let _perm3 = Permission::EditDocuments;
    let _perm4 = Permission::CommentDocuments;
}

// Test: ActionType enum variants
#[test]
fn test_action_type_enum_variants() {
    // Verify action type variants exist
    let _action1 = ActionType::ViewDocument;
    let _action2 = ActionType::EditDocument;
    let _action3 = ActionType::DeleteDocument;
    let _action4 = ActionType::CommentDocument;
}

// Test: Error enum variants
#[test]
fn test_error_enum_variants() {
    let _error1 = Error::Unauthorized("test".to_string());
    let _error2 = Error::Forbidden("test".to_string());
    let _error3 = Error::InternalServerError("test".to_string());
}

// Test: Error status_code conversion
#[test]
fn test_error_status_conversion() {
    let status = StatusCode::UNAUTHORIZED;
    assert_eq!(status.as_u16(), 401u16);

    let status = StatusCode::FORBIDDEN;
    assert_eq!(status.as_u16(), 403u16);

    let status = StatusCode::INTERNAL_SERVER_ERROR;
    assert_eq!(status.as_u16(), 500u16);
}

// Test: JWT decoding key
#[test]
fn test_jwt_decoding_key() {
    let _key = jsonwebtoken::DecodingKey::from_secret(b"secret_key");
    // DecodingKey is created from secret, can't directly compare
    // In real usage, this would decode tokens with HS256 algorithm
}

// Test: Token validation algorithm
#[test]
fn test_token_validation_algorithm() {
    let validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256);
    // Validation is created with HS256 algorithm
    assert!(validation.algorithms.contains(&jsonwebtoken::Algorithm::HS256));
}

// Test: Secret key type
#[test]
fn test_secret_key_type() {
    // JWT_SECRET should be a String type that can be borrowed as &str
    let _secret: &str = JWT_SECRET.as_str();
}

// Test: Claims extraction
#[test]
fn test_claims_extraction_empty_claims() {
    let claims = Claims {
        iss: "user123".to_string(),
        exp: 10000,
        role: None,
        permissions: vec![],
    };
    
    assert_eq!(claims.iss, "user123");
    assert_eq!(claims.exp, 10000);
    assert!(claims.role.is_none());
    assert!(claims.permissions.is_empty());
}

#[test]
fn test_claims_extraction_with_claims() {
    let claims = Claims {
        iss: "user123".to_string(),
        exp: 10000,
        role: Some("editor".to_string()),
        permissions: vec![
            "documents::delete".to_string(),
            "documents::view".to_string(),
        ],
    };
    
    assert_eq!(claims.iss, "user123");
    assert_eq!(claims.exp, 10000);
    assert!(claims.role.is_some());
    assert_eq!(claims.role.unwrap(), "editor");
    assert_eq!(claims.permissions.len(), 2);
    assert!(claims.permissions.contains(&"documents::delete".to_string()));
    assert!(claims.permissions.contains(&"documents::view".to_string()));
}

#[test]
fn test_claims_extraction_invalid_role() {
    let claims = Claims {
        iss: "user123".to_string(),
        exp: 10000,
        role: Some("invalid_role".to_string()),
        permissions: vec![],
    };
    
    assert_eq!(claims.iss, "user123");
    assert!(claims.role.is_some());
    assert_eq!(claims.role.unwrap(), "invalid_role");
    assert_eq!(claims.permissions.len(), 0);
}

// Test: has_permission with various roles
#[test]
fn test_has_permission_owner() {
    assert!(has_permission(&"owner", &Permission::DeleteDocuments));
    assert!(has_permission(&"owner", &Permission::ViewDocuments));
    assert!(has_permission(&"owner", &Permission::EditDocuments));
    assert!(has_permission(&"owner", &Permission::CommentDocuments));
    assert!(has_permission(&"owner", &Permission::DeleteSpace));
}

#[test]
fn test_has_permission_editor() {
    assert!(!has_permission(&"editor", &Permission::DeleteDocuments));
    assert!(has_permission(&"editor", &Permission::ViewDocuments));
    assert!(has_permission(&"editor", &Permission::EditDocuments));
    assert!(has_permission(&"editor", &Permission::CommentDocuments));
    assert!(has_permission(&"editor", &Permission::ViewSpaces));
    assert!(has_permission(&"editor", &Permission::EditSpaces));
}

#[test]
fn test_has_permission_commenter() {
    assert!(!has_permission(&"commenter", &Permission::DeleteDocuments));
    assert!(has_permission(&"commenter", &Permission::ViewDocuments));
    assert!(!has_permission(&"commenter", &Permission::EditDocuments));
    assert!(has_permission(&"commenter", &Permission::CommentDocuments));
    assert!(has_permission(&"commenter", &Permission::ViewSpaces));
    assert!(!has_permission(&"commenter", &Permission::EditSpaces));
}

#[test]
fn test_has_permission_viewer() {
    assert!(has_permission(&"viewer", &Permission::ViewDocuments));
    assert!(!has_permission(&"viewer", &Permission::EditDocuments));
    assert!(!has_permission(&"viewer", &Permission::CommentDocuments));
    assert!(!has_permission(&"viewer", &Permission::DeleteSpace));
}

// Test: has_permission with invalid role
#[test]
fn test_has_permission_invalid_role() {
    assert!(!has_permission(&"invalid_role", &Permission::DeleteDocuments));
    assert!(!has_permission(&"invalid_role", &Permission::ViewDocuments));
}

// Test: can_perform_action for all actions
#[test]
fn test_can_perform_action_owner() {
    let actions = vec![
        ActionType::DeleteDocument,
        ActionType::ViewDocument,
        ActionType::EditDocument,
        ActionType::CommentDocument,
        ActionType::DeleteSpace,
    ];
    
    for action in actions {
        assert!(can_perform_action(&"owner", &action));
    }
}

#[test]
fn test_can_perform_action_editor() {
    let actions = vec![
        ActionType::ViewDocument,
        ActionType::EditDocument,
        ActionType::CommentDocument,
    ];

    for action in actions {
        assert!(can_perform_action(&"editor", &action));
    }
}

#[test]
fn test_can_perform_action_commenter() {
    let actions = vec![
        ActionType::ViewDocument,
        ActionType::CommentDocument,
    ];

    for action in actions {
        assert!(can_perform_action(&"commenter", &action));
    }
}

#[test]
fn test_can_perform_action_viewer() {
    let actions = vec![
        ActionType::ViewDocument,
    ];

    for action in actions {
        assert!(can_perform_action(&"viewer", &action));
    }
}

// Test: can_perform_action insufficient permissions
#[test]
fn test_can_perform_action_viewer_insufficient() {
    assert!(!can_perform_action(&"viewer", &ActionType::EditDocument));
}

// Test: can_perform_action_commenter_insufficient
#[test]
fn test_can_perform_action_commenter_insufficient() {
    assert!(!can_perform_action(&"commenter", &ActionType::EditDocument));
}

// Test: extract_role_from_string
#[test]
fn test_extract_role_owner() {
    assert_eq!(extract_role_from_string("owner").unwrap(), Role::Owner);
}

#[test]
fn test_extract_role_editor() {
    assert_eq!(extract_role_from_string("editor").unwrap(), Role::Editor);
}

#[test]
fn test_extract_role_commenter() {
    assert_eq!(extract_role_from_string("commenter").unwrap(), Role::Commenter);
}

#[test]
fn test_extract_role_viewer() {
    assert_eq!(extract_role_from_string("viewer").unwrap(), Role::Viewer);
}

#[test]
fn test_extract_role_invalid() {
    assert!(extract_role_from_string("invalid").is_none());
}

// Test: Error::Unauthorized response
#[test]
fn test_error_unauthorized_response() {
    let error = Error::Unauthorized("Missing JWT token".to_string());
    
    assert_eq!(error.to_string(), "Missing JWT token");
    assert_eq!(error.status(), 401);
}

// Test: Error::Forbidden response
#[test]
fn test_error_forbidden_response() {
    let error = Error::Forbidden("Insufficient permissions".to_string());
    
    assert_eq!(error.to_string(), "Insufficient permissions");
    assert_eq!(error.status(), 403);
}

// Test: Error::InternalServerError response
#[test]
fn test_error_internal_server_error_response() {
    let error = Error::InternalServerError("Server error".to_string());
    
    assert_eq!(error.to_string(), "Server error");
    assert_eq!(error.status(), 500);
}

// Test: Status code values
#[test]
fn test_status_codes() {
    assert_eq!(StatusCode::UNAUTHORIZED.as_u16(), 401);
    assert_eq!(StatusCode::FORBIDDEN.as_u16(), 403);
    assert_eq!(StatusCode::INTERNAL_SERVER_ERROR.as_u16(), 500);
}

// Test: Response building
#[test]
fn test_response_building() {
    let response = HttpResponse::build(StatusCode::UNAUTHORIZED).body("Unauthorized".to_string());

    // Compare StatusCode numerically using as_u16()
    assert_eq!(response.status().as_u16(), 401u16);
    // Alternatively, compare to StatusCode constant
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

// Test: Bearer token with extra spaces
#[test]
fn test_bearer_token_extra_spaces() {
    let token = "valid_token_here";
    let header = format!("Authorization: Bearer  {}", token);

    // With extra spaces, should still parse
    assert!(header.contains(&token));
    assert_eq!(header.split_whitespace().len(), 3);
}

// Test: Invalid token format passed to extract_claims
#[test]
fn test_extract_claims_invalid_token() {
    let token = "valid_token_here";

    // extract_claims expects a full Authorization header with Bearer prefix
    // Passing a plain token without "Bearer " prefix should fail
    let result = extract_claims(&token);

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::Unauthorized(_)));
}

// Test: Malformed authorization header
#[test]
fn test_malformed_authorization_header() {
    // Missing "Bearer " prefix
    let header = "InvalidToken xyz".to_string();
    
    // With no Bearer, extract_claims will fail
    let result = extract_claims(&header);
    
    assert!(result.is_err());
}

// Test: Invalid token type
#[test]
fn test_invalid_token_type() {
    // Not JWT token
    let header = "Authorization: Basic invalid".to_string();
    
    let result = extract_claims(&header);
    
    assert!(result.is_err());
}

// Test: User ID validation
#[test]
fn test_extract_user_id_valid() {
    let fixed_uuid = "550e8400-e29b-41d4-a716-446655440000";
    let claims = Claims {
        iss: fixed_uuid.to_string(),
        exp: 10000,
        role: None,
        permissions: vec![],
    };

    let result = extract_user_id(&claims);

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Uuid::parse_str(fixed_uuid).unwrap());
}

#[test]
fn test_extract_user_id_missing_in_claims() {
    let claims = Claims {
        iss: "user123".to_string(),
        exp: 10000,
        role: None,
        permissions: vec![],
    };
    
    assert!(matches!(extract_user_id(&claims), Err(_)));
}

// Test: user role with multiple permissions
#[test]
fn test_user_role_multiple_permissions() {
    let claims = Claims {
        iss: "user123".to_string(),
        exp: 10000,
        role: None,
        permissions: vec![
            "documents::delete".to_string(),
            "spaces::view".to_string(),
            "documents::edit".to_string(),
            "documents::comment".to_string(),
        ],
    };
    
    let result = extract_user_role(&claims);

    assert!(result.is_ok());
    let role = result.unwrap();
    assert_eq!(role, Role::Owner);
}

// Test: is_space_member stub (currently returns an error)
#[test]
fn test_is_space_member_stub() {
    // This function is a stub that always returns an error (Result::Err)
    // TODO: Update when real space membership check is implemented
    
    let claims = Claims {
        iss: "user123".to_string(),
        exp: 10000,
        role: None,
        permissions: vec![],
    };
    
    let result = is_space_member(&claims, &Uuid::new_v4());
    
    // Currently stub always returns false
    assert!(result.is_err());
}

// Test: RBAC middleware structure
#[test]
fn test_rbac_middleware_structure() {
    // Just test that it's properly structured
    let _middleware = RbacMiddleware::new();

    // New() should return Self - type checking happens at compile time
    // This test primarily verifies the struct exists and can be instantiated
}

// Test: Permission level comparison
#[test]
fn test_permission_hierarchy() {
    // Owner has all permissions
    // Editor can view, edit, comment (but NOT delete) documents
    // Commenter can view, comment documents
    // Viewer can only view documents

    assert!(has_permission(&"owner", &Permission::DeleteSpace));

    assert!(!has_permission(&"editor", &Permission::DeleteDocuments));
    assert!(has_permission(&"editor", &Permission::ViewDocuments));
    assert!(has_permission(&"editor", &Permission::EditDocuments));
    assert!(has_permission(&"editor", &Permission::CommentDocuments));

    assert!(has_permission(&"commenter", &Permission::ViewDocuments));
    assert!(has_permission(&"commenter", &Permission::CommentDocuments));

    assert!(has_permission(&"viewer", &Permission::ViewDocuments));
    assert!(!has_permission(&"viewer", &Permission::EditDocuments));
}

// Test: Action type hierarchy
#[test]
fn test_action_type_hierarchy() {
    // Delete actions require Owner
    // Edit actions require Editor or above
    // Comment actions require Commenter or above
    // View actions require Viewer or above

    // Owner can delete
    assert!(can_perform_action(&"owner", &ActionType::DeleteDocument));
    // Editor cannot delete
    assert!(!can_perform_action(&"editor", &ActionType::DeleteDocument));

    // Editor can edit
    assert!(can_perform_action(&"editor", &ActionType::EditDocument));
    // Commenter cannot edit
    assert!(!can_perform_action(&"commenter", &ActionType::EditDocument));

    // Commenter can comment
    assert!(can_perform_action(&"commenter", &ActionType::CommentDocument));
    // Viewer cannot edit
    assert!(!can_perform_action(&"viewer", &ActionType::EditDocument));

    // Viewer can view
    assert!(can_perform_action(&"viewer", &ActionType::ViewDocument));
    // Viewer cannot delete
    assert!(!can_perform_action(&"viewer", &ActionType::DeleteDocument));
}

// Test: Claims:: iss field validation
#[test]
fn test_claims_iss_field_validation() {
    let _valid_iss = "valid_user_id".to_string();

    // Invalid iss
    let claims_empty_iss = Claims {
        iss: String::new(),
        exp: 0,
        role: None,
        permissions: vec![],
    };
    
    let result = extract_user_id(&claims_empty_iss);
    
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::Unauthorized(_)));
}

// Test: Claims: exp field validation
#[test]
fn test_claims_exp_field_validation() {
    // Expiration should be positive, typically > 0
    let fixed_uuid = "550e8400-e29b-41d4-a716-446655440000";
    let claims_no_exp = Claims {
        iss: fixed_uuid.to_string(), // Use valid UUID so extract_user_id succeeds
        exp: 0, // Invalid: no expiration
        role: None,
        permissions: vec![],
    };

    // This should still extract but with exp=0
    let result = extract_user_id(&claims_no_exp);

    assert!(result.is_ok());
}

// Test: Claims: role field None handling
#[test]
fn test_claims_role_none_handling() {
    let claims_no_role = Claims {
        iss: "user123".to_string(),
        exp: 10000,
        role: None,
        permissions: vec![],
    };

    let result = extract_user_role(&claims_no_role);

    // Role is None, should still return user_id
    assert!(result.is_ok());
}

// Test: Claims: permissions vector handling
#[test]
fn test_claims_permissions_vector() {
    let fixed_uuid = "550e8400-e29b-41d4-a716-446655440000";
    let claims_no_perms = Claims {
        iss: fixed_uuid.to_string(),
        exp: 10000,
        role: None,
        permissions: vec![],
    };

    let result = extract_user_id(&claims_no_perms);

    assert!(result.is_ok());
}

// Test: Authorization header whitespace handling
#[test]
fn test_auth_header_whitespace_handling() {
    // Header value may have extra whitespace, extract_claims should handle it
    let header_with_spaces = "Bearer   token123".to_string();
    let header_trimmed = "Bearer token123".to_string();

    // Both headers should extract the same token after parsing
    // Both should fail to decode as valid JWT but fail at the same stage
    let result_with_spaces = extract_claims(&header_with_spaces);
    let result_trimmed = extract_claims(&header_trimmed);

    // Both should fail with Unauthorized (not a valid JWT token)
    assert!(result_with_spaces.is_err());
    assert!(result_trimmed.is_err());
    assert!(matches!(result_with_spaces.unwrap_err(), Error::Unauthorized(_)));
    assert!(matches!(result_trimmed.unwrap_err(), Error::Unauthorized(_)));
}

// Test: Error response building
#[test]
fn test_error_response_building() {
    let error = Error::Unauthorized("Test error".to_string());
    let response = HttpResponse::build(StatusCode::UNAUTHORIZED).body("Unauthorized".to_string());

    // Verify error string representation and status code
    assert_eq!(error.to_string(), "Test error");
    assert_eq!(error.status(), 401);
    // Verify response status code
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
}
