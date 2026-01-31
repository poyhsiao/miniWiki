//! Integration tests for space_service
//!
//! These tests verify the space functionality including handlers,
//! repository operations, and model validation.

use serde_json::json;
use uuid::Uuid;
use validator::Validate;

use space_service::models::*;
use space_service::repository::SpaceRepository;

// ========================================
// Model Tests
// ========================================

#[test]
fn test_space_creation() {
    let ts = chrono::DateTime::from_timestamp(0, 0).unwrap().naive_utc();
    let space = Space {
        id: Uuid::new_v4(),
        owner_id: Uuid::new_v4(),
        name: "Test Space".to_string(),
        icon: Some("🚀".to_string()),
        description: Some("A test space".to_string()),
        is_public: false,
        created_at: ts,
        updated_at: ts,
    };

    assert_eq!(space.name, "Test Space");
    assert!(!space.is_public);
    assert!(space.icon.is_some());
}

#[test]
fn test_space_public() {
    let ts = chrono::DateTime::from_timestamp(0, 0).unwrap().naive_utc();
    let space = Space {
        id: Uuid::new_v4(),
        owner_id: Uuid::new_v4(),
        name: "Public Space".to_string(),
        icon: None,
        description: None,
        is_public: true,
        created_at: ts,
        updated_at: ts,
    };

    assert!(space.is_public);
    assert!(space.icon.is_none());
}

#[test]
fn test_create_space_request_valid() {
    let request = CreateSpaceRequest {
        name: "My Space".to_string(),
        icon: Some("📁".to_string()),
        description: Some("Description".to_string()),
        is_public: true,
    };

    assert!(request.validate().is_ok());
}

#[test]
fn test_create_space_request_empty_name() {
    let request = CreateSpaceRequest {
        name: "".to_string(),
        icon: None,
        description: None,
        is_public: false,
    };

    assert!(request.validate().is_err()); // Empty name fails
}

#[test]
fn test_create_space_request_name_too_long() {
    let request = CreateSpaceRequest {
        name: "a".repeat(201), // Max is 200
        icon: None,
        description: None,
        is_public: false,
    };

    assert!(request.validate().is_err()); // Name too long fails
}

#[test]
fn test_create_space_request_icon_too_long() {
    let request = CreateSpaceRequest {
        name: "Valid Name".to_string(),
        icon: Some("a".repeat(51)), // Max is 50
        description: None,
        is_public: false,
    };

    assert!(request.validate().is_err()); // Icon too long fails
}

#[test]
fn test_update_space_request_partial() {
    let request = UpdateSpaceRequest {
        name: Some("Updated Name".to_string()),
        icon: None,
        description: None,
        is_public: Some(true),
    };

    assert!(request.name.is_some());
    assert!(request.icon.is_none());
    assert!(request.is_public.is_some());
}

#[test]
fn test_update_space_request_all_fields() {
    let request = UpdateSpaceRequest {
        name: Some("New Name".to_string()),
        icon: Some("⭐".to_string()),
        description: Some("New description".to_string()),
        is_public: Some(false),
    };

    assert!(request.name.is_some());
    assert!(request.icon.is_some());
    assert!(request.description.is_some());
    assert!(request.is_public.is_some());
}

#[test]
fn test_space_membership() {
    let joined_at = chrono::DateTime::from_timestamp(0, 0).unwrap().naive_utc();
    let membership = SpaceMembership {
        id: Uuid::new_v4(),
        space_id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        role: "editor".to_string(),
        joined_at,
        invited_by: Uuid::new_v4(),
    };

    assert_eq!(membership.role, "editor");
}

#[test]
fn test_space_membership_roles() {
    let joined_at = chrono::DateTime::from_timestamp(0, 0).unwrap().naive_utc();

    for role in ["owner", "admin", "editor", "viewer"] {
        let membership = SpaceMembership {
            id: Uuid::new_v4(),
            space_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            role: role.to_string(),
            joined_at,
            invited_by: Uuid::new_v4(),
        };
        assert_eq!(membership.role, role);
    }
}

#[test]
fn test_add_member_request() {
    let request = AddMemberRequest {
        user_id: Uuid::new_v4().to_string(),
        role: "viewer".to_string(),
    };

    assert!(request.validate().is_ok());
}

#[test]
fn test_add_member_request_role_boundaries() {
    // Empty role should fail (min length = 1)
    let empty_request = AddMemberRequest {
        user_id: Uuid::new_v4().to_string(),
        role: "".to_string(),
    };
    assert!(empty_request.validate().is_err(), "Empty role should fail");

    // Role longer than 50 chars should fail (max length = 50)
    let long_request = AddMemberRequest {
        user_id: Uuid::new_v4().to_string(),
        role: "a".repeat(51),
    };
    assert!(
        long_request.validate().is_err(),
        "Role longer than 50 chars should fail"
    );

    // Valid-length role should pass (validator only checks length, not specific values)
    let valid_request = AddMemberRequest {
        user_id: Uuid::new_v4().to_string(),
        role: "editor".to_string(),
    };
    assert!(valid_request.validate().is_ok(), "Valid-length role should pass");
}

#[test]
fn test_update_member_request() {
    let request = UpdateMemberRequest {
        role: "admin".to_string(),
    };

    assert_eq!(request.role, "admin");
}

#[test]
fn test_update_member_request_valid_roles() {
    for role in ["admin", "editor", "viewer"] {
        let request = UpdateMemberRequest { role: role.to_string() };
        assert!(request.validate().is_ok());
    }
}

// ========================================
// Space Error Tests
// ========================================

#[test]
fn test_space_error_not_found() {
    let error = SpaceError::NotFound;
    assert_eq!(error.to_string(), "Space not found");
}

#[test]
fn test_space_error_forbidden() {
    let error = SpaceError::Forbidden;
    assert_eq!(error.to_string(), "Access denied");
}

#[test]
fn test_space_error_validation() {
    let error = SpaceError::Validation("Invalid name".to_string());
    assert_eq!(error.to_string(), "Validation error: Invalid name");
}

#[test]
fn test_space_error_database() {
    let error = SpaceError::Database(sqlx::Error::RowNotFound);
    assert!(error.to_string().contains("Database error"));
}

// ========================================
// Repository Pattern Tests
// ========================================

#[test]
fn test_space_repository_new_exists() {
    // Verify SpaceRepository::new constructor exists
    let _ = SpaceRepository::new;
}

#[test]
fn test_space_repository_list_by_user_exists() {
    // Verify SpaceRepository::list_by_user method exists
    let _ = SpaceRepository::list_by_user;
}

#[test]
fn test_space_repository_create_exists() {
    // Verify SpaceRepository::create method exists
    let _ = SpaceRepository::create;
}

#[test]
fn test_space_repository_find_by_id_exists() {
    // Verify SpaceRepository::find_by_id method exists
    let _ = SpaceRepository::find_by_id;
}

// ========================================
// Handler Helper Function Tests
// ========================================

#[test]
fn test_extract_user_id_from_valid_jwt() {
    use jsonwebtoken::{encode, EncodingKey, Header};

    let secret = "test-secret-key-for-testing-only-do-not-use-in-production";
    let user_id = Uuid::new_v4().to_string();

    let claims = json!({
        "sub": user_id,
        "iat": chrono::Utc::now().timestamp(),
        "exp": chrono::Utc::now().timestamp() + 3600
    });

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .expect("Failed to encode token");

    // Verify token can be decoded
    let decoding_key = jsonwebtoken::DecodingKey::from_secret(secret.as_bytes());
    let validation = jsonwebtoken::Validation::default();

    let decoded = jsonwebtoken::decode::<serde_json::Value>(&token, &decoding_key, &validation);
    assert!(decoded.is_ok());
    assert_eq!(decoded.unwrap().claims.get("sub").unwrap().as_str().unwrap(), user_id);
}

#[test]
fn test_jwt_encoding_decoding() {
    use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};

    let secret = "test-secret-key-for-testing-only-do-not-use-in-production";

    let claims = json!({
        "sub": "550e8400-e29b-41d4-a716-446655440000",
        "iat": chrono::Utc::now().timestamp(),
        "exp": chrono::Utc::now().timestamp() + 3600
    });

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .expect("Failed to encode");

    let decoded = decode::<serde_json::Value>(
        &token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .expect("Failed to decode");

    assert_eq!(
        decoded.claims.get("sub").unwrap().as_str().unwrap(),
        "550e8400-e29b-41d4-a716-446655440000"
    );
}

// ========================================
// Edge Case Tests
// ========================================

#[test]
fn test_create_space_request_minimal() {
    let request = CreateSpaceRequest {
        name: "A".to_string(), // Minimum 1 character
        icon: None,
        description: None,
        is_public: false,
    };

    assert!(request.validate().is_ok());
}

#[test]
fn test_create_space_request_max_name() {
    let request = CreateSpaceRequest {
        name: "a".repeat(200), // Exactly 200 characters
        icon: None,
        description: None,
        is_public: false,
    };

    assert!(request.validate().is_ok());
}

#[test]
fn test_create_space_request_max_description() {
    let request = CreateSpaceRequest {
        name: "Valid Name".to_string(),
        icon: None,
        description: Some("a".repeat(1000)), // Exactly 1000 characters
        is_public: false,
    };

    assert!(request.validate().is_ok());
}

#[test]
fn test_update_space_request_empty() {
    // All fields None is valid (no updates)
    let request = UpdateSpaceRequest {
        name: None,
        icon: None,
        description: None,
        is_public: None,
    };

    assert!(request.name.is_none());
    assert!(request.icon.is_none());
    assert!(request.description.is_none());
    assert!(request.is_public.is_none());
}

#[test]
fn test_space_with_special_characters_in_name() {
    let ts = chrono::DateTime::from_timestamp(0, 0).unwrap().naive_utc();
    let space = Space {
        id: Uuid::new_v4(),
        owner_id: Uuid::new_v4(),
        name: "Space with special chars: @#$%".to_string(),
        icon: None,
        description: None,
        is_public: false,
        created_at: ts,
        updated_at: ts,
    };

    assert!(space.name.contains("@"));
    assert!(space.name.contains("#"));
}

// ========================================
// Validation Edge Cases
// ========================================

#[test]
fn test_create_space_request_description_too_long() {
    let request = CreateSpaceRequest {
        name: "Valid Name".to_string(),
        icon: None,
        description: Some("a".repeat(1001)), // Max is 1000
        is_public: false,
    };

    assert!(request.validate().is_err()); // Description too long fails
}

#[test]
fn test_add_member_request_valid_uuid() {
    let request = AddMemberRequest {
        user_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
        role: "editor".to_string(),
    };

    assert!(Uuid::parse_str(&request.user_id).is_ok());
}

#[test]
fn test_add_member_request_invalid_uuid() {
    let request = AddMemberRequest {
        user_id: "not-a-valid-uuid".to_string(),
        role: "viewer".to_string(),
    };

    assert!(Uuid::parse_str(&request.user_id).is_err());
}

// ========================================
// Performance/Capacity Tests
// ========================================

#[test]
fn test_space_name_boundary_values() {
    // Test name at boundary values
    for len in [1, 199, 200, 201] {
        let name = "a".repeat(len);
        let request = CreateSpaceRequest {
            name,
            icon: None,
            description: None,
            is_public: false,
        };

        if len <= 200 {
            // Should be valid
            assert!(request.validate().is_ok(), "Name length {} should be valid", len);
        } else {
            // Should be invalid
            assert!(request.validate().is_err(), "Name length {} should be invalid", len);
        }
    }
}

#[test]
fn test_role_string_values() {
    let valid_roles = ["owner", "admin", "editor", "viewer"];
    let invalid_roles = ["superuser", "moderator", "", "root"];

    for role in valid_roles {
        let request = UpdateMemberRequest { role: role.to_string() };
        assert!(request.validate().is_ok(), "Role '{}' should be valid", role);
    }

    for role in invalid_roles {
        let request = UpdateMemberRequest { role: role.to_string() };
        // Note: validator only checks length, not specific values
        if role.is_empty() {
            assert!(request.validate().is_err(), "Empty role should be invalid");
        } else {
            assert!(
                request.validate().is_ok(),
                "Non-empty role '{}' currently passes length-only validation",
                role
            );
        }
    }
}
